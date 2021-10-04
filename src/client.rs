mod consts;
use tokio_tungstenite::{client_async, tungstenite::protocol::Message};
use kube::api::{Api, WatchEvent, ListParams};
use k8s_openapi::api::core::v1::Pod;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection failure: {0}")]
    ConnectionError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn get_pod_stream<'a>(
    pods: Api<Pod>,
) -> Result<tokio_tungstenite::WebSocketStream<(impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin)>>
{
    use mojaloop_api::clients::k8s;

    let voodoo_doll_stream = k8s::port_forward_stream(
        &None,
        &None,
        Some(pods),
        // TODO: we sort of can't really know how the pod is going to be identified,
        // therefore this information should be exposed by the voodoo-doll lib, one way or
        // another. Perhaps voodoo-doll lib should have a function that accepts a pod list
        // (our &pods above) and returns the correct pod, if it is present. And creates it,
        // if not?
        k8s::KubernetesParams {
            container_name: "app",
            label: "app.kubernetes.io/name=voodoo-doll",
            port: k8s::Port::Number(3030),
        }
    ).await.map_err(|e| Error::ConnectionError(e.to_string()))?;

    // TODO: we sort of can't really know what endpoint to call, therefore this information
    // should be exposed by the voodoo-doll lib, one way or another.
    // let uri = "/voodoo".parse::<http::Uri>().unwrap();
    let (ws_stream, _) = client_async("ws://host.ignored/voodoo", voodoo_doll_stream)
        .await.map_err(|e| Error::ConnectionError(e.to_string()))?;

    Ok(ws_stream)
}

pub async fn create(pods: Api<Pod>) -> Result<()> {
    // TODO: here we fail if the pod exists or is being created/deleted- need to handle
    // this better.
    pods.create(
        &kube::api::PostParams::default(),
        consts::POD,
        // serde_json::from_value(consts::POD_JSON).unwrap(),
    ).await?;

    // Wait until the pod is running
    let lp = ListParams::default()
        .fields(format!("metadata.name={}", &consts::POD_NAME).as_str())
        .timeout(30);
    let mut stream = pods.watch(&lp, "0").await?.boxed();
    while let Some(status) = stream.try_next().await? {
        match status {
            WatchEvent::Added(o) => {
                println!("Added {}", o.name());
            }
            WatchEvent::Modified(o) => {
                let s = o.status.as_ref().expect("status exists on pod");
                if s.phase.clone().unwrap_or_default() == "Running" {
                    break;
                }
            }
            _ => {}
        }
    }
}

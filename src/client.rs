mod consts;
use tokio_tungstenite::client_async;
use kube::api::{Api, WatchEvent, ListParams, PostParams, DeleteParams};
use k8s_openapi::api::core::v1::Pod;
use thiserror::Error;

pub use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection failure: {0}")]
    ConnectionError(String),
    #[error("Error creating k8s resource: {0}")]
    KubernetesResourceCreation(String),
    #[error("Error connecting to pod: {0}")]
    PodConnect(String),
    #[error("Failed to fetch k8s pods: {0}")]
    PodList(String),
    #[error("Error deleting pod: {0}")]
    PodDelete(String),
    #[error("Unable to create k8s client: {0}")]
    KubeClientCreation(String),
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

pub async fn create(
    client: Option<kube::client::Client>,
    namespace: &Option<String>,
) -> Result<()> {
    use futures::TryStreamExt;
    use kube::ResourceExt;
    use k8s_openapi::api::core::v1::ServiceAccount;
    use k8s_openapi::api::rbac::v1::{ClusterRole, ClusterRoleBinding};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    fn get_api<T: k8s_openapi::Metadata<Ty = ObjectMeta>>(
        client: kube::Client, ns: &Option<String>
    ) -> Api<T> {
        match ns {
            Some(n) => Api::namespaced(client, n.as_str()),
            None => Api::default_namespaced(client),
        }
    }

    async fn create_resource<T>(
        client: kube::Client,
        namespace: &Option<String>,
        res_json: serde_json::Value,
    ) -> Result<()>
        where
            T: k8s_openapi::Metadata<Ty = ObjectMeta> + Clone + serde::de::DeserializeOwned + std::fmt::Debug + serde::Serialize
    {
        let api = get_api::<T>(client, namespace);
        let res = serde_json::from_value(res_json.clone()).unwrap();
        api.create(
            &PostParams::default(),
            &res,
        // ).await.map_err(|e| Error::KubernetesResourceCreation(e.to_string())).and(Ok(()))
        ).await.map_err(|e| Error::KubernetesResourceCreation(format!("Attempting to create:\n{:?}\n{:?}", res_json, e.to_string()))).and(Ok(()))
    }

    // TODO: here we fail if the pod exists or is being created/deleted- need to handle
    // this better.
    let client = match client {
        Some(c) => c,
        None => kube::Client::try_default().await
            .map_err(|e| Error::KubeClientCreation(e.to_string()))?
    };

    // Order matters here: we must create the service account before the pod.

    tokio::try_join!(
        create_resource::<ServiceAccount>(
            client.clone(),
            namespace,
            consts::SERVICEACCOUNT_JSON.clone(),
        ),
        create_resource::<ClusterRole>(
            client.clone(),
            namespace,
            consts::CLUSTERROLE_JSON.clone(),
        ),
        create_resource::<ClusterRoleBinding>(
            client.clone(),
            namespace,
            consts::CLUSTERROLEBINDING_JSON.clone(),
        ),
    )?;

    create_resource::<Pod>(
        client.clone(),
        namespace,
        consts::POD_JSON.clone(),
    ).await?;

    let pod_api = get_api::<Pod>(client.clone(), namespace);

    // Wait until the pod is running
    let lp = ListParams::default()
        .fields(format!("metadata.name={}", &consts::POD_NAME).as_str())
        .timeout(30);
    let mut stream = Box::pin(pod_api.watch(&lp, "0").await.map_err(|e| Error::PodConnect(e.to_string()))?);
    while let Some(status) = stream.try_next().await.map_err(|e| Error::PodConnect(e.to_string()))? {
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
    Ok(())
}

// TODO: should probably take kube::Client instead of kube::Api
pub async fn destroy(pods: Option<Api<Pod>>) -> Result<()> {
    let pods = match pods {
        None => fspiox_api::clients::k8s::get_pods(&None, &None).await.map_err(|e| Error::PodList(e.to_string()))?,
        Some(pods) => pods,
    };
    pods.delete(consts::POD_NAME, &DeleteParams::default()).await
        .and(Ok(()))
        .map_err(|e| Error::PodDelete(e.to_string()))
}

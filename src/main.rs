// Retrieve our hostname from the container environment?
// https://kubernetes.io/docs/concepts/containers/container-environment/
// It's possible to use hostname -i, so probably possible from code

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use thiserror::Error;

use fspiox_api::*;
mod protocol;

#[derive(Error, Debug)]
enum VoodooError {
    #[error("Couldn't convert from http::request to reqwest::Request")]
    RequestConversionError,
    #[error("Received a non-string websocket message from client")]
    NonStringWebsocketMessageReceived,
    #[error("Unrecognised message received from client")]
    WebsocketMessageDeserializeFailed,
    #[error("Typed an invalid URL in the code. Need a unit test for this..")]
    InvalidUrl,
    #[error("HOST_IP environment variable not set")]
    HostIpNotFound,
    #[error("Failed to set participant endpoint: {0}")]
    FailedToSetParticipantEndpoint(String),
}

type Result<T> = std::result::Result<T, VoodooError>;

static CLIENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

type ClientId = usize;
type Clients = Arc<RwLock<HashMap<ClientId, mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>>>;

#[derive(Hash, PartialEq, Eq, Debug)]
enum FspiopMessageId {
    TransferId(transfer::TransferId),
}

/// FSPIOP messages that we've sent that we're expecting a callback for, mapped to the client that
/// requested them.
type InFlightFspiopMessages = Arc<RwLock<HashMap<FspiopMessageId, ClientId>>>;

// TODO: implement this properly to return an actual FSPIOP error. Or perhaps add warp-specific
// return value implementations to the fspiox-api FspiopError.
#[derive(Error, Debug)]
enum FspiopError {
    #[error("Request body couldn't be deserialized: {0}")]
    MalformedRequestBody(String),
}
impl warp::reject::Reject for FspiopError {}

// We implement a json body handler here because warp::body::json() also enforces the content-type
// header to equal application/json. This doesn't work for us.
fn json_body<T: serde::de::DeserializeOwned + Send>() -> impl warp::Filter<Extract = (T,), Error = warp::Rejection> + Copy {
    warp::body::bytes().and_then(|buf: hyper::body::Bytes| async move {
        serde_json::from_slice::<T>(&buf)
            // .map_err(|e| warp::reject::known(warp::filters::body::BodyDeserializeError { cause: e }))
            .map_err(|e| warp::reject::custom(FspiopError::MalformedRequestBody(e.to_string())))
    })
}

async fn handle_rejection(err: warp::reject::Rejection) -> std::result::Result<impl warp::reply::Reply, std::convert::Infallible> {
    use warp::http::StatusCode;
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(e) = err.find::<FspiopError>() {
        match e {
            FspiopError::MalformedRequestBody(s) => {
                code = StatusCode::BAD_REQUEST;
                message = s;
            }
        }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // TODO: we only _really_ want to handle FspiopError here, and nothing else. Can we fall back to
        // the default rejection handler for everything that's not an FspiopError?
        println!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    #[derive(serde::Serialize)]
    struct ErrorMessage {
        code: u16,
        message: String,
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

#[tokio::main]
async fn main() {
    let clients = Clients::default();
    let in_flight_msgs = InFlightFspiopMessages::default();

    // GET /voodoo -> websocket upgrade
    let voodoo = warp::path("voodoo")
        .and(warp::ws())
        // TODO: is there a tidier way to do this?
        .and(warp::any().map({ let clients = clients.clone(); move || clients.clone()}))
        .and(warp::any().map({ let in_flight_msgs = in_flight_msgs.clone(); move || in_flight_msgs.clone() }))
        .map(|ws: warp::ws::Ws, clients, in_flight_msgs| {
            ws.on_upgrade(move |socket| ws_connection_handler(socket, clients, in_flight_msgs))
        });

    // PUT /transfers
    let put_transfers = warp::put()
        .and(warp::path("transfers"))
        .and(warp::path::param::<transfer::TransferId>())
        .and(warp::path::end())
        .and(json_body())
        .and_then({
            let clients = clients.clone();
            let in_flight_msgs = in_flight_msgs.clone();
            move |transfer_id, transfer_fulfil: transfer::TransferFulfilRequestBody|
                handle_put_transfers(transfer_id, transfer_fulfil, in_flight_msgs.clone(), clients.clone())
        });

    // POST /transfers
    let post_transfers = warp::post()
        .and(warp::path("transfers"))
        .and(warp::path::end())
        .and(json_body())
        // TODO: does this create a new client per-request? I guess so? Avoid this..
        .and(warp::any().map(|| reqwest::Client::new()))
        .and_then({
            let clients = clients.clone();
            let in_flight_msgs = in_flight_msgs.clone();
            move |transfer_prepare: transfer::TransferPrepareRequestBody, http_client|
                handle_post_transfers(transfer_prepare, http_client, in_flight_msgs.clone(), clients.clone())
        });

    // PUT /transfers/{id}/error
    let put_transfers_error = warp::put()
        .and(warp::path("transfers"))
        .and(warp::path::param::<transfer::TransferId>())
        .and(warp::path("error"))
        .and(warp::path::end())
        .and(json_body())
        .and_then({
            let clients = clients.clone();
            let in_flight_msgs = in_flight_msgs.clone();
            move |transfer_id: transfer::TransferId, transfer_error: fspiox_api::common::ErrorResponse|
                handle_put_transfers_error(transfer_id, transfer_error, in_flight_msgs.clone(), clients.clone())
        });

    let routes = voodoo
        .or(put_transfers)
        .or(post_transfers)
        .or(put_transfers_error)
        .recover(handle_rejection);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn handle_put_transfers(
    transfer_id: transfer::TransferId,
    transfer_fulfil: transfer::TransferFulfilRequestBody,
    in_flight_msgs: InFlightFspiopMessages,
    clients: Clients,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {
    match in_flight_msgs.read().await.get(&FspiopMessageId::TransferId(transfer_id)) {
        // TODO: assert that the transfer ID in the URI matches the transfer ID in the payload?
        Some(client_id) => {
            println!("Transfer fulfil received | {:?}", transfer_fulfil);
            if let Some(client_ws_tx) = clients.write().await.get_mut(client_id) {
                let msg_text = serde_json::to_string(&protocol::ServerMessage::TransferComplete(
                    protocol::TransferCompleteMessage {
                        id: transfer_id,
                    }
                )).unwrap();
                if let Err(_disconnected) = client_ws_tx.send(Ok(Message::text(msg_text))) {
                    // Disconnect handled elsewhere
                    println!("Client disconnected, failed to send");
                }
            } else {
                println!("No client found for transfer");
            }
        }
        None => println!("Received unrecognised FSPIOP transfer prepare message")
    }
    Ok("")
}

async fn handle_put_transfers_error(
    transfer_id: transfer::TransferId,
    transfer_error: fspiox_api::common::ErrorResponse,
    in_flight_msgs: InFlightFspiopMessages,
    clients: Clients,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {
    match in_flight_msgs.read().await.get(&FspiopMessageId::TransferId(transfer_id)) {
        // TODO: assert that the transfer ID in the URI matches the transfer ID in the payload?
        Some(client_id) => {
            println!("Transfer error received {} | {:?}", transfer_id, transfer_error);
            if let Some(client_ws_tx) = clients.write().await.get_mut(client_id) {
                let msg_text = serde_json::to_string(&protocol::ServerMessage::TransferError(
                    protocol::TransferErrorMessage {
                        id: transfer_id,
                        response: transfer_error,
                    }
                )).unwrap();
                if let Err(_disconnected) = client_ws_tx.send(Ok(Message::text(msg_text))) {
                    // Disconnect handled elsewhere
                    println!("Client disconnected, failed to send");
                }
            } else {
                println!("No client found for transfer");
            }
        }
        None => println!(
            "Received unrecognised FSPIOP transfer error message. ID: {:?}",
            transfer_id,
        )
    }
    Ok("")
}

async fn handle_post_transfers(
    transfer_prepare: transfer::TransferPrepareRequestBody,
    http_client: reqwest::Client,
    in_flight_msgs: InFlightFspiopMessages,
    clients: Clients,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {
    use std::convert::TryFrom;
    let req_post_transfer = to_http_request(
        build_transfer_fulfil(
            transfer_prepare.payer_fsp,
            transfer_prepare.payee_fsp,
            transfer_prepare.transfer_id,
        ),
        // TODO: more robust mechanism for finding the "ml api adapter service" service
        "http://ml-api-adapter-service",
    ).unwrap();
    let request = reqwest::Request::try_from(req_post_transfer).unwrap();
    http_client.execute(request).await.unwrap();
    Ok("")
}

async fn ws_connection_handler(ws: WebSocket, clients: Clients, in_flight_msgs: InFlightFspiopMessages) {
    let client_id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    println!("voodoo client connected: {}", client_id);

    let (client_ws_tx, mut client_ws_rx) = ws.split();

    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(rx.forward(client_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));

    clients.write().await.insert(client_id, tx);

    let http_client = reqwest::Client::new();

    // Handle client messages
    while let Some(result) = client_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", client_id, e);
                break;
            }
        };
        if let Err(e) = client_message(client_id, &http_client, msg, &in_flight_msgs).await {
            println!("Uh oh: {}", e);
        };
    }

    // The handling above will block until the client disconnects.
    eprintln!("client disconnected: {}", client_id);
    clients.write().await.remove(&client_id);
}

async fn client_message(
    client_id: usize,
    http_client: &reqwest::Client,
    msg: Message,
    in_flight_msgs: &InFlightFspiopMessages
) -> Result<()> {
    // TODO: consider replying with non-string messages with "go away"
    let msg = msg.to_str().map_err(|_| VoodooError::NonStringWebsocketMessageReceived)?;
    println!("Message from client: {}", msg);
    let msg_de: protocol::ClientMessage = serde_json::from_str(msg)
        .map_err(|_| VoodooError::WebsocketMessageDeserializeFailed)?;

    use mojaloop_api::central_ledger::participants;

    // TODO: Get this at startup and panic if it fails. If it fails once, it will always fail.
    // TODO: Assert hostname is valid URI using url::Uri::parse?
    // TODO: We use status.podIP, but we might better use status.podIPs (or _maybe_, but probably
    //       not, status.hostIP)
    let my_ip = std::env::var("HOST_IP").map_err(|_| VoodooError::HostIpNotFound)?;
    let my_address = format!("http://{}:{}", my_ip, 3030);

    match msg_de {
        protocol::ClientMessage::Transfer(transfer_message) => {
            use std::convert::TryFrom;

            // TODO: check all transfer preconditions (optionally)? I.e.:
            //       - hub has correct currency accounts
            //       - 
            //       - sender exists, has correct currency accounts, has sufficient liquidity
            //       - recipient exists, has correct currency accounts

            // TODO: a lot of scope for deduplication here. The best thing to do might be to
            // temporarily intercept all FSPIOP messages for the relevant participants, and restore
            // them afterward.

            // Take over the sender's PUT /transfers endpoint
            let req_set_sender_transfer_fulfil = participants::to_request(
                participants::PostCallbackUrl {
                    name: transfer_message.msg_sender.clone(),
                    callback_type: participants::FspiopCallbackType::FspiopCallbackUrlTransferPut,
                    hostname: my_address.clone(),
                },
                // TODO: more robust mechanism for finding the "central ledger service" service
                "http://centralledger-service",
            ).map_err(|_| VoodooError::InvalidUrl)?;
            let request = reqwest::Request::try_from(req_set_sender_transfer_fulfil)
                .map_err(|_| VoodooError::RequestConversionError)?;
            http_client.execute(request).await
                .map_err(|e| VoodooError::FailedToSetParticipantEndpoint(e.to_string()))?;

            // Take over the recipient's POST /transfers endpoint
            let req_set_recipient_transfer_fulfil = participants::to_request(
                participants::PostCallbackUrl {
                    name: transfer_message.msg_recipient.clone(),
                    callback_type: participants::FspiopCallbackType::FspiopCallbackUrlTransferPost,
                    hostname: my_address.clone(),
                },
                // TODO: more robust mechanism for finding the "central ledger service" service
                "http://centralledger-service",
            ).map_err(|_| VoodooError::InvalidUrl)?;
            let request = reqwest::Request::try_from(req_set_recipient_transfer_fulfil)
                .map_err(|_| VoodooError::RequestConversionError)?;
            http_client.execute(request).await
                .map_err(|e| VoodooError::FailedToSetParticipantEndpoint(e.to_string()))?;

            // Take over both the sender's and recipient's transfers error endpoint
            let req_set_sender_transfer_error = participants::to_request(
                participants::PostCallbackUrl {
                    name: transfer_message.msg_sender.clone(),
                    callback_type: participants::FspiopCallbackType::FspiopCallbackUrlTransferError,
                    hostname: my_address.clone(),
                },
                // TODO: more robust mechanism for finding the "central ledger service" service
                "http://centralledger-service",
            ).map_err(|_| VoodooError::InvalidUrl)?;
            let request = reqwest::Request::try_from(req_set_sender_transfer_error)
                .map_err(|_| VoodooError::RequestConversionError)?;
            http_client.execute(request).await
                .map_err(|e| VoodooError::FailedToSetParticipantEndpoint(e.to_string()))?;

            let req_set_recipient_transfer_error = participants::to_request(
                participants::PostCallbackUrl {
                    name: transfer_message.msg_recipient.clone(),
                    callback_type: participants::FspiopCallbackType::FspiopCallbackUrlTransferError,
                    hostname: my_address.clone(),
                },
                // TODO: more robust mechanism for finding the "central ledger service" service
                "http://centralledger-service",
            ).map_err(|_| VoodooError::InvalidUrl)?;
            let request = reqwest::Request::try_from(req_set_recipient_transfer_error)
                .map_err(|_| VoodooError::RequestConversionError)?;
            http_client.execute(request).await
                .map_err(|e| VoodooError::FailedToSetParticipantEndpoint(e.to_string()))?;

            // Send the transfer prepare, we'll receive it on our POST /transfers soon enough..
            let req_post_transfer = to_http_request(
                build_transfer_prepare(
                    transfer_message.msg_sender,
                    transfer_message.msg_recipient,
                    transfer_message.amount,
                    transfer_message.currency,
                    Some(transfer_message.transfer_id),
                ),
                // TODO: more robust mechanism for finding the "ml api adapter service" service
                "http://ml-api-adapter-service",
            ).map_err(|_| VoodooError::InvalidUrl)?;
            let request = reqwest::Request::try_from(req_post_transfer)
                .map_err(|_| VoodooError::RequestConversionError)?;
            http_client.execute(request).await
                .map_err(|e| VoodooError::FailedToSetParticipantEndpoint(e.to_string()))?;

            in_flight_msgs.write().await.insert(
                FspiopMessageId::TransferId(transfer_message.transfer_id),
                client_id
            );

            // 1. hijack the appropriate participants
            //    - participants might not exist- we should require they exist, to begin with
            // 2. send the transfer prepare
            //    - there might be some manner of connectivity error, or our request could receive
            //      a sync response for being malformed (I _think_)
            // 3. exit this function, _but_, in the POST /transfer handler....
            // 4. return a PUT /transfer
            //    - there might be some manner of connectivity error, or our request could receive
            //      a sync response for being malformed (I _think_)
            // 5. notify this caller
            //
            // - this could all take forever, for whatever reason, there should probably be a
            //   timeout, nonconfigurable at first
        }
    }

    Ok(())
}

// Retrieve our hostname from the container environment?
// https://kubernetes.io/docs/concepts/containers/container-environment/
// It's possible to use hostname -i, so probably possible from code

use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{FutureExt, StreamExt};
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws;
use warp::Filter;
use thiserror::Error;
use regex::Regex;
use lazy_static::lazy_static;

use fspiox_api::{FspId, transfer};
mod protocol;
mod control_plane;
mod fspiop_handlers;
use crate::fspiop_handlers::*;

#[derive(Error, Debug)]
enum VoodooError {
    #[error("Received a non-string websocket message from client")]
    NonStringWebsocketMessageReceived,
    #[error("Unrecognised message received from client. Error: {0}. Message: {1}")]
    WebsocketMessageDeserializeFailed(String, String),
    #[error("HOST_IP environment variable not set")]
    HostIpNotFound,
    #[error("Switch client error: {0}")]
    ClientError(String),
}

impl From<mojaloop_api::clients::Error> for VoodooError {
    fn from(client_error: mojaloop_api::clients::Error) -> VoodooError {
        VoodooError::ClientError(client_error.to_string())
    }
}

static CLIENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

type Result<T> = std::result::Result<T, VoodooError>;
impl warp::reject::Reject for control_plane::FspiopError {}

// We implement a json body handler here because warp::body::json() also enforces the content-type
// header to equal application/json. This doesn't work for us.
fn json_body<T: serde::de::DeserializeOwned + Send>() -> impl warp::Filter<Extract = (T,), Error = warp::Rejection> + Copy {
    warp::body::bytes().and_then(|buf: hyper::body::Bytes| async move {
        serde_json::from_slice::<T>(&buf)
            // .map_err(|e| warp::reject::known(warp::filters::body::BodyDeserializeError { cause: e }))
            .map_err(|e| warp::reject::custom(control_plane::FspiopError::MalformedRequestBody(e.to_string())))
    })
}

async fn handle_rejection(err: warp::reject::Rejection) -> std::result::Result<impl warp::reply::Reply, std::convert::Infallible> {
    use warp::http::StatusCode;
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(e) = err.find::<control_plane::FspiopError>() {
        match e {
            control_plane::FspiopError::MalformedRequestBody(s) => {
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
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use mojaloop_api::clients::FspiopClient;
    let clients = control_plane::Clients::default();
    let in_flight_msgs = control_plane::InFlightFspiopMessages::default();
    let k8s_client = match kube::Client::try_default().await {
        Ok(cli) => cli,
        Err(e) => panic!("Couldn't infer k8s config from environment: {}", e),
    };

    // GET /voodoo -> websocket upgrade
    let voodoo = warp::path("voodoo")
        .and(warp::ws())
        // TODO: is there a tidier way to do this?
        .and(warp::any().map({ let clients = clients.clone(); move || clients.clone()}))
        .and(warp::any().map({ let in_flight_msgs = in_flight_msgs.clone(); move || in_flight_msgs.clone() }))
        .and(warp::any().map({ let k8s_client = k8s_client.clone(); move || k8s_client.clone() }))
        .map(|ws: warp::ws::Ws, clients, in_flight_msgs, k8s_client| {
            ws.on_upgrade(move |socket| ws_connection_handler(socket, clients, in_flight_msgs, k8s_client))
        });

    // PUT /transfers
    let put_transfers = warp::put()
        .and(warp::path("transfers"))
        // TODO: .and(warp::path!("transfers" / transfer::TransferId))
        .and(warp::path::param::<transfer::TransferId>())
        .and(warp::path::end())
        .and(json_body())
        .and_then({
            let clients = clients.clone();
            let in_flight_msgs = in_flight_msgs.clone();
            move |transfer_id, transfer_fulfil: transfer::TransferFulfilRequestBody|
                handle_put_transfers(transfer_id, transfer_fulfil, in_flight_msgs.clone(), clients.clone())
        });

    let transfer_client = Arc::new(Mutex::new(
        mojaloop_api::clients::transfer::Client::from_k8s_defaults().await.unwrap()
    ));
    // POST /transfers
    let post_transfers = warp::post()
        .and(warp::path("transfers"))
        .and(warp::path::end())
        .and(json_body())
        // .and_then({
        //     move |transfer_prepare: transfer::TransferPrepareRequestBody|
        //         handle_post_transfers(transfer_prepare, transfer_client)
        // });
        .and(warp::any().map({ let client = transfer_client.clone(); move || client.clone()}))
        .and_then({
            move |transfer_prepare: transfer::TransferPrepareRequestBody, client: Arc<Mutex<mojaloop_api::clients::transfer::Client>>|
                handle_post_transfers(transfer_prepare, client)
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
            move |transfer_id: transfer::TransferId, transfer_error: fspiox_api::ErrorResponse|
                handle_put_transfers_error(transfer_id, transfer_error, in_flight_msgs.clone(), clients.clone())
        });

    let routes = voodoo
        .or(put_transfers)
        .or(post_transfers)
        .or(put_transfers_error)
        .recover(handle_rejection);

    println!("Voodoo Doll {} starting on port 3030", env!("CARGO_PKG_VERSION"));

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

async fn ws_connection_handler(
    ws: ws::WebSocket,
    clients: control_plane::Clients,
    in_flight_msgs: control_plane::InFlightFspiopMessages,
    k8s_client: kube::Client,
) {
    let client_id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    println!("voodoo client connected: {}", client_id);

    let (client_ws_tx, mut client_ws_rx) = ws.split();

    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(rx.forward(client_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("Websocket send error: {}", e);
        }
    }));

    // TODO: what is the default namespace? If we're running inside a cluster and we use defaults,
    // will the default namespace be "default" (or whatever is set as the default cluster
    // namespace), or will it be the namespace we're running in?
    let mut moja_clients = match mojaloop_api::clients::k8s::get_all_from_k8s(Some(k8s_client), &None).await {
        Err(e) => {
            panic!("Couldn't connect to switch for websocket client: {}", e);
        },
        Ok(cs) => cs
    };

    clients.write().await.insert(client_id, control_plane::ClientData { chan: tx, participants: Vec::new() });

    // Handle client messages
    while let Some(result) = client_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Websocket error(uid={}): {}", client_id, e);
                break;
            }
        };
        if let Err(e) = client_message(client_id, &mut moja_clients, msg, &in_flight_msgs, &clients).await {
            // TODO: let the client know they sent us something we couldn't handle
            println!("Uh oh: {}", e);
        };
    }

    // The handling above will block until the client disconnects.
    eprintln!("client disconnected: {}", client_id);
    clients.write().await.remove(&client_id);
}

async fn client_message(
    client_id: usize,
    moja_clients: &mut mojaloop_api::clients::k8s::Clients,
    msg: ws::Message,
    in_flight_msgs: &control_plane::InFlightFspiopMessages,
    clients: &control_plane::Clients,
) -> Result<()> {
    use mojaloop_api::central_ledger::participants;
    use mojaloop_api::settlement::{settlement, settlement_windows};

    // TODO: consider replying with non-string messages with "go away"
    let msg = msg.to_str().map_err(|_| VoodooError::NonStringWebsocketMessageReceived)?;
    println!("Raw message from client: {}", msg);

    let msg_de: protocol::ClientMessage = serde_json::from_str(msg)
        .map_err(|e| VoodooError::WebsocketMessageDeserializeFailed(e.to_string(), msg.to_string()))?;
    println!("Deserialized message from client: {:?}", msg_de);

    // TODO: Get this at startup and panic if it fails. If it fails once, it will always fail.
    // TODO: Assert hostname is valid URI using url::Uri::parse?
    // TODO: We use status.podIP, but we might better use status.podIPs (or _maybe_, but probably
    //       not, status.hostIP)
    let my_ip = std::env::var("HOST_IP").map_err(|_| VoodooError::HostIpNotFound)?;
    let my_address = format!("http://{}:{}", my_ip, 3030);

    #[derive(serde::Deserialize, Debug)]
    #[serde(untagged)]
    enum MlApiResponse<T> {
        Err(fspiox_api::ErrorResponse),
        Response(T),
    }
    #[derive(serde::Deserialize, Debug)]
    struct Empty {}

    fn handle_failed_match<T: std::fmt::Debug>(res: mojaloop_api::clients::Result<T>) {
        match res {
            Ok(m) => {
                println!("Whoopsie. TODO: let client know there was a problem. The problem: {:?}", m);
            }
            Err(mojaloop_api::clients::Error::MojaloopApiError(e)) => {
                println!("Whoopsie. TODO: let client know there was a problem. The problem: {:?}", e);
            }
            Err(e) => {
                println!("Whoopsie. TODO: let client know there was a problem. The problem: {:?}", e);
            }
        }
    }

    match msg_de {
        protocol::ClientMessage::CreateSettlement(new_settlement) => {
            if let Some(client_data) = clients.write().await.get_mut(&client_id) {
                let create_settlement_req = settlement::PostSettlement { new_settlement };
                println!("Create settlement request: {:?}", create_settlement_req);
                let response = moja_clients.settlement.send(create_settlement_req).await?.des().await?;
                client_data.send(&protocol::ServerMessage::NewSettlementCreated(response));
            } else {
                // TODO: we should return something to the client indicating an error
                println!("No client data found for connection!");
            }
        }

        protocol::ClientMessage::GetSettlements(query_params) => {
            if let Some(client_data) = clients.write().await.get_mut(&client_id) {
                let result = moja_clients.settlement.send(query_params).await?.des().await?;
                client_data.send(&protocol::ServerMessage::Settlements(result));
            } else {
                // TODO: we should return something to the client indicating an error
                println!("No client data found for connection!");
            }
        }

        protocol::ClientMessage::GetSettlementWindows(query_params) => {
            if let Some(client_data) = clients.write().await.get_mut(&client_id) {
                let result = moja_clients.settlement.send(query_params).await?.des().await?;
                client_data.send(&protocol::ServerMessage::SettlementWindows(result));
            } else {
                // TODO: we should return something to the client indicating an error
                println!("No client data found for connection!");
            }
        }

        protocol::ClientMessage::CloseSettlementWindow(close_msg) => {
            if let Some(client_data) = clients.write().await.get_mut(&client_id) {
                let close_req = settlement_windows::CloseSettlementWindow {
                    id: close_msg.id,
                    payload: settlement_windows::SettlementWindowClosurePayload {
                        state: settlement_windows::SettlementWindowCloseState::Closed,
                        reason: close_msg.reason,
                    }
                };
                let close_result = match moja_clients.settlement.send(close_req).await {
                    Ok(_) => {
                        // Settlement window closure happens asynchronously for no reason that I
                        // can discern. So we need to poll central settlement for the window state
                        // until it's closed.
                        // TODO: ask _why_ window closure happens asynchronously
                        println!("Begin polling for settlement window {} closure", close_msg.id);
                        let mut n = 1;
                        loop {
                            let window_req = settlement_windows::GetSettlementWindow {
                                id: close_msg.id,
                            };
                            let window = moja_clients.settlement.send(window_req).await?.des().await?;
                            println!("Polling attempt #{} to retrieve settlement window {} state. State: {:?}", n, close_msg.id, window);
                            if window.state == settlement_windows::SettlementWindowState::Closed {
                                println!(
                                    "Finished polling for settlement window {} closure. Window closed.",
                                    close_msg.id,
                                );
                                break Ok(
                                    protocol::ServerMessage::SettlementWindowClosed(close_msg.id)
                                );
                            }
                            if n == 10 {
                                break Err(
                                    format!(
                                        "Polling for settlement window {} closure failed after {} attempts",
                                        close_msg.id,
                                        n,
                                    )
                                );
                            }
                            sleep(Duration::from_secs(1)).await;
                            n = n + 1;
                        }
                    }
                    Err(mojaloop_api::clients::Error::MojaloopApiError(ml_err)) => {
                        Ok(
                            protocol::ServerMessage::SettlementWindowCloseFailed(
                                protocol::SettlementWindowCloseFailedMessage {
                                    id: close_msg.id,
                                    response: ml_err,
                                }
                            )
                        )
                    }
                    x => {
                        Err(format!("Unhandled error closing settlement window: {:?}", x))
                    }
                };
                match close_result {
                    Ok(msg) => {
                        println!("Sending message to client: {:?}", msg);
                        client_data.send(&msg);
                    }
                    Err(msg) => {
                        println!("Whoopsie. TODO: let client know there was a problem. The problem: {:?}", msg);
                    }
                }
            } else {
                // TODO: we should return something to the client indicating an error
                println!("No client data found for connection!");
            }
        }

        protocol::ClientMessage::CreateHubAccounts(accounts) => {
            if let Some(client_data) = clients.write().await.get_mut(&client_id) {
                for account in &accounts {
                    let create_account_req = participants::PostHubAccount {
                        // TODO: need to get hub name from switch; older versions
                        // of ML use "hub" instead of "Hub".
                        name: FspId::from("Hub").unwrap(),
                        account: *account,
                    };
                    // TODO: we don't actually care about the json response here if the
                    // response code was a 2xx. If it wasn't, we might be interested in the
                    // following response:
                    //   ErrorResponse {
                    //     error_information: ErrorInformation {
                    //       error_code: AddPartyInfoError,
                    //       error_description: "Add Party information error - Hub account has already been registered."
                    //     }
                    //   }
                    if let Err(e) = moja_clients.central_ledger.send(create_account_req).await {
                        println!("Whoopsie. TODO: let client know there was a problem. The problem: {:?}", e);
                    }
                }
                client_data.send(&protocol::ServerMessage::HubAccountsCreated(accounts));
            } else {
                // TODO: we should return something to the client indicating an error
                println!("No client data found for connection!");
            }
        }

        protocol::ClientMessage::CreateParticipants(create_participants_message) => {
            // TODO: it's pretty obvious that the client will want the hub RECONCILIATION accounts
            // and settlement model created. We could probably just do that? If anyone ever doesn't
            // want the hub accounts created first, we'll get a PR.
            // TODO: optionally accept participant names. If the participant already exists, check
            // if it's in our participant pool. If it is, check if it's been "issued" to another
            // client. If it hasn't, "issue" it to the requester.
            // TODO: ensure we return participants in the same order they're sent in. E.g. if the
            // client requests an MMK participant and an SEK participant, in that order, the result
            // should be an array containing participants with those properties, in that order.
            if let Some(client_data) = clients.write().await.get_mut(&client_id) {
                let mut new_participants: Vec<protocol::ClientParticipant> = Vec::new();

                // TODO: this in parallel, with try_join!
                for account_init in create_participants_message.iter() {
                    use std::iter;
                    use rand::{SeedableRng, rngs::StdRng, Rng};
                    use rand::distributions::Alphanumeric;

                    // TODO: here, we simply assume that the participant does not already exist.
                    // There's a possibility that they actually already do. We could either
                    // maintain a pool of participants, or, probably better, if our attempt to
                    // create said participant fails with "participant exists", try again with a
                    // different name.
                    let mut rng: StdRng = SeedableRng::from_entropy(); // TODO: move this outside, if possible
                    let name_suffix: String = iter::repeat(())
                        .map(|()| rng.sample(Alphanumeric))
                        .map(char::from)
                        .take(24)
                        .collect();
                    let name = FspId::from(format!("voodoo{}", name_suffix).as_str()).unwrap();

                    let new_participant_req = participants::PostParticipant {
                        participant: participants::NewParticipant {
                            currency: account_init.currency,
                            name,
                        },
                    };
                    let new_participant = moja_clients.central_ledger.send(new_participant_req).await?.des().await?;
                    let participant_init_req = participants::PostInitialPositionAndLimits {
                        initial_position_and_limits: participants::InitialPositionAndLimits {
                            currency: account_init.currency,
                            limit: participants::Limit {
                                r#type: participants::LimitType::NetDebitCap,
                                value: account_init.ndc,
                            },
                            initial_position: account_init.initial_position,
                        },
                        name,
                    };
                    moja_clients.central_ledger.send(participant_init_req).await?;

                    println!("Created participant {} for client", name);
                    // TODO: Need to disable these participants when we're done with them
                    client_data.participants.push(new_participant.name.clone());
                    new_participants.push(protocol::ClientParticipant {
                        name: new_participant.name,
                        account: *account_init,
                    });
                }

                client_data.send(&protocol::ServerMessage::AssignParticipants(new_participants));
            } else {
                // TODO: we should return something to the client indicating an error
                println!("No client data found for connection!");
            }
        }

        protocol::ClientMessage::Transfers(transfers_message) => {
            for transfer in transfers_message.iter() {
                // TODO: check all transfer preconditions (optionally)? I.e.:
                //       - hub has correct currency accounts
                //       - sender and recipient are active
                //       - sender exists, has correct, active currency accounts, has sufficient liquidity
                //       - recipient exists, has correct, active currency accounts

                // TODO: restore FSP endpoints afterward
                for participant in [&transfer.msg_recipient, &transfer.msg_sender].iter() {
                    println!("Overriding endpoints for {}", participant);
                    use strum::IntoEnumIterator;
                    for callback_type in participants::FspiopCallbackType::iter() {
                        let request = participants::PostCallbackUrl {
                            name: **participant,
                            callback_type,
                            // TODO: strip trailing slash
                            hostname: my_address.clone(),
                        };
                        moja_clients.central_ledger.send(request).await?;
                        println!("Updated {:?} endpoint to {}.", callback_type, my_address.clone());
                    }
                }

                // Send the transfer prepare, we'll receive it on our POST /transfers soon enough..
                let req_post_transfer = transfer::TransferPrepareRequest::new(
                        transfer.msg_sender.clone(),
                        transfer.msg_recipient.clone(),
                        transfer.amount,
                        transfer.currency,
                        Some(transfer.transfer_id),
                );
                moja_clients.transfer.send(req_post_transfer).await?;

                println!("Storing in-flight message {}", transfer.transfer_id);
                in_flight_msgs.write().await.insert(
                    control_plane::FspiopMessageId::TransferId(transfer.transfer_id),
                    client_id
                );

            }

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

        protocol::ClientMessage::CreateSettlementModel(settlement_model) => {
            if let Some(client_data) = clients.write().await.get_mut(&client_id) {
                use mojaloop_api::central_ledger::settlement_models;

                let success_response =
                    protocol::ServerMessage::SettlementModelCreated(
                        protocol::SettlementModelCreatedMessage {
                            settlement_model: settlement_model.clone()
                        }
                    );

                let settlement_model_create_req = settlement_models::PostSettlementModel {
                    settlement_model
                };

                match moja_clients.central_ledger.send(settlement_model_create_req).await {
                    Ok(_) => {
                        client_data.send(&success_response);
                    },
                    Err(fspiox_api::clients::Error::MojaloopApiError(ml_err)) => {
                        lazy_static! {
                            static ref RE: Regex = Regex::new(r".*Settlement model.*already exists.*").unwrap();
                        }
                        if ml_err.error_information.error_code == fspiox_api::MojaloopApiError::InternalServerError &&
                            RE.is_match(&ml_err.error_information.error_description) {
                            // TODO: this is actually an error that the client should be
                            // informed of. The switch has simply said "there has been a name
                            // clash- you asked for a settlement model name that already
                            // existed". At the time of writing, ignoring this case is
                            // desirable, however this is certainly not the general case, and
                            // this should be handled, and an error returned to the client.
                            client_data.send(&success_response);
                        } else {
                            println!("Whoopsie. TODO: let client know there was a problem. The problem: {:?}", ml_err);
                        }
                    },
                    x => handle_failed_match(x),
                }
            } else {
                // TODO: we should return something to the client indicating an error
                println!("No client data found for connection!");
            }
        }
    }

    Ok(())
}

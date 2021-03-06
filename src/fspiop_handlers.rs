use crate::control_plane;
use fspiox_api::transfer;
use crate::protocol;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) async fn handle_put_transfers(
    transfer_id: transfer::TransferId,
    transfer_fulfil: transfer::TransferFulfilRequestBody,
    in_flight_msgs: control_plane::InFlightFspiopMessages,
    clients: control_plane::Clients,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {
    match in_flight_msgs.read().await.get(&control_plane::FspiopMessageId::TransferId(transfer_id)) {
        // TODO: assert that the transfer ID in the URI matches the transfer ID in the payload?
        Some((client_id, msg_id)) => {
            println!("Transfer fulfil received | {:?}", transfer_fulfil);
            if let Some(client_data) = clients.write().await.get_mut(client_id) {
                client_data.send(
                    Some(*msg_id),
                    protocol::Notification::TransferComplete(
                        protocol::TransferCompleteMessage {
                            id: transfer_id,
                        }
                    )
                );
            } else {
                println!("No client found for transfer");
            }
        }
        None => println!("Received unrecognised FSPIOP transfer prepare message")
    }
    Ok("")
}

pub(crate) async fn handle_put_transfers_error(
    transfer_id: transfer::TransferId,
    transfer_error: fspiox_api::ErrorResponse,
    in_flight_msgs: control_plane::InFlightFspiopMessages,
    clients: control_plane::Clients,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {
    match in_flight_msgs.read().await.get(&control_plane::FspiopMessageId::TransferId(transfer_id)) {
        // TODO: assert that the transfer ID in the URI matches the transfer ID in the payload?
        Some((client_id, msg_id)) => {
            println!("Transfer error received {} | {:?}", transfer_id, transfer_error);
            if let Some(client_data) = clients.write().await.get_mut(client_id) {
                client_data.send(
                    Some(*msg_id),
                    protocol::Notification::TransferError(
                        protocol::TransferErrorMessage {
                            id: transfer_id,
                            response: transfer_error,
                        }
                    )
                );
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

pub(crate) async fn handle_post_transfers(
    transfer_prepare: transfer::TransferPrepareRequestBody,
    transfer_client: Arc<Mutex<mojaloop_api::clients::transfer::Client>>,
    in_flight_msgs: control_plane::InFlightFspiopMessages,
    clients: control_plane::Clients,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {
    println!("Received POST /transfer {:?}", transfer_prepare);
    // TODO: we need to identify whether one of our clients owns the participant for which this
    // transfer prepare is intended. If so, we should send a corresponding notification to this
    // client.
    match in_flight_msgs.read().await.get(&control_plane::FspiopMessageId::TransferId(transfer_prepare.transfer_id)) {
        // TODO: assert that the transfer ID in the URI matches the transfer ID in the payload?
        Some((client_id, msg_id)) => {
            if let Some(client_data) = clients.write().await.get_mut(client_id) {
                client_data.send(
                    Some(*msg_id),
                    protocol::Notification::TransferPrepare(transfer_prepare),
                );
            } else {
                println!("No client found for transfer");
            }
        }
        None => println!("Received FSPIOP transfer prepare message not initiated by us")
    }
    let req_put_transfer = transfer::TransferFulfilRequest::new(
        transfer_prepare.payer_fsp,
        transfer_prepare.payee_fsp,
        transfer_prepare.transfer_id,
    );
    println!("Sending PUT /transfer {:?}", req_put_transfer);
    match transfer_client.lock().await.send(req_put_transfer).await {
        Ok(_) => {
            println!("Sent PUT /transfer with ID {}", transfer_prepare.transfer_id);
        }
        e => {
            println!("Failed to send PUT /transfer with ID {:?}. Error:", e);
        }
    }
    Ok("")
}

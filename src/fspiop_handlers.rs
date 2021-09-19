use crate::control_plane;
use fspiox_api::{transfer, to_http_request, build_transfer_fulfil};
use crate::protocol;

pub(crate) async fn handle_put_transfers(
    transfer_id: transfer::TransferId,
    transfer_fulfil: transfer::TransferFulfilRequestBody,
    in_flight_msgs: control_plane::InFlightFspiopMessages,
    clients: control_plane::Clients,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {
    match in_flight_msgs.read().await.get(&control_plane::FspiopMessageId::TransferId(transfer_id)) {
        // TODO: assert that the transfer ID in the URI matches the transfer ID in the payload?
        Some(client_id) => {
            println!("Transfer fulfil received | {:?}", transfer_fulfil);
            if let Some(client_data) = clients.write().await.get_mut(client_id) {
                let msg = &protocol::ServerMessage::TransferComplete(
                    protocol::TransferCompleteMessage {
                        id: transfer_id,
                    }
                );
                client_data.send(&msg);
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
    transfer_error: fspiox_api::common::ErrorResponse,
    in_flight_msgs: control_plane::InFlightFspiopMessages,
    clients: control_plane::Clients,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {
    match in_flight_msgs.read().await.get(&control_plane::FspiopMessageId::TransferId(transfer_id)) {
        // TODO: assert that the transfer ID in the URI matches the transfer ID in the payload?
        Some(client_id) => {
            println!("Transfer error received {} | {:?}", transfer_id, transfer_error);
            if let Some(client_data) = clients.write().await.get_mut(client_id) {
                let msg = protocol::ServerMessage::TransferError(
                    protocol::TransferErrorMessage {
                        id: transfer_id,
                        response: transfer_error,
                    }
                );
                client_data.send(&msg);
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
    http_client: reqwest::Client,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {
    use std::convert::TryFrom;
    println!("Received POST /transfer {:?}", transfer_prepare);
    let req_put_transfer = to_http_request(
        build_transfer_fulfil(
            transfer_prepare.payer_fsp,
            transfer_prepare.payee_fsp,
            transfer_prepare.transfer_id,
        ),
        // TODO: more robust mechanism for finding the "ml api adapter service" service
        "http://ml-api-adapter-service",
    ).unwrap();
    println!("Sending PUT /transfer {:?}", req_put_transfer);
    let request = reqwest::Request::try_from(req_put_transfer).unwrap();
    http_client.execute(request).await.unwrap();
    println!("Sent PUT /transfer with ID {}", transfer_prepare.transfer_id);
    Ok("")
}

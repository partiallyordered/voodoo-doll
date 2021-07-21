use fspiox_api::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferMessage {
    pub msg_sender: common::FspId,
    pub msg_recipient: common::FspId,
    pub currency: common::Currency,
    pub amount: common::Amount,
    // If we have the caller provide the transfer ID, we can use this as a unique message reference
    // for this message sequence with this caller.
    pub transfer_id: transfer::TransferId,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Transfer(TransferMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferCompleteMessage {
    pub id: transfer::TransferId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferErrorMessage {
    pub id: transfer::TransferId,
    pub response: common::ErrorResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    TransferComplete(TransferCompleteMessage),
    TransferError(TransferErrorMessage),
}

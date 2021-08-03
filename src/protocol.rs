use fspiox_api::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript_types")]
use ts_rs::{TS, export};
#[cfg(feature = "typescript_types")]
use std::any::TypeId;

#[cfg_attr(feature = "typescript_types", derive(TS))]
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

#[cfg_attr(feature = "typescript_types", derive(TS))]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct AccountInitialization {
    pub currency: common::Currency,
    pub initial_position: common::Amount,
    pub ndc: u32,
}

#[cfg_attr(feature = "typescript_types", derive(TS))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ClientMessage {
    /// Run end-to-end transfers
    Transfers(Vec<TransferMessage>),
    /// Create hub settlement and reconciliation accounts
    CreateHubAccounts(Vec<common::Currency>),
    // TODO: this _could_ be a vector of vectors of accounts. Each 0th-level vector would represent
    // a participant, and each 1st-level vector would contain desired accounts.
    /// Create a set of participants. Will be disabled when the socket disconnects.
    CreateParticipants(Vec<AccountInitialization>),
}

#[cfg_attr(feature = "typescript_types", derive(TS))]
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferCompleteMessage {
    pub id: transfer::TransferId,
}

#[cfg_attr(feature = "typescript_types", derive(TS))]
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferErrorMessage {
    pub id: transfer::TransferId,
    pub response: common::ErrorResponse,
}

#[cfg_attr(feature = "typescript_types", derive(TS))]
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientParticipant {
    pub name: common::FspId,
    pub account: AccountInitialization,
}

#[cfg_attr(feature = "typescript_types", derive(TS))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ServerMessage {
    TransferComplete(TransferCompleteMessage),
    TransferError(TransferErrorMessage),
    AssignParticipants(Vec<ClientParticipant>),
    HubAccountsCreated(Vec<common::Currency>),
}

#[cfg(feature = "typescript_types")]
export! {
    TransferCompleteMessage,
    TransferErrorMessage,
    AccountInitialization,
    ClientParticipant,
    TransferMessage,
    common::DateTime,
    common::Amount,
    common::Currency,
    common::ErrorResponse,
    common::ErrorInformation,
    common::MojaloopApiError,
    ServerMessage,
    ClientMessage => "clients/typescript/src/lib/protocol.ts"
}

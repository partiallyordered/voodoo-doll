use fspiox_api::*;
use mojaloop_api::central_ledger::{participants, settlement_models};
use mojaloop_api::settlement::settlement_windows;
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript_types")]
use ts_rs::{TS, export};

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
    CreateHubAccounts(Vec<participants::HubAccount>),
    // TODO: this _could_ be a vector of vectors of accounts. Each 0th-level vector would represent
    // a participant, and each 1st-level vector would contain desired accounts.
    // TODO: disable the participants on socket closure
    /// Create a set of participants. Will be disabled when the socket disconnects.
    CreateParticipants(Vec<AccountInitialization>),
    /// Create a settlement model
    CreateSettlementModel(settlement_models::SettlementModel),
    /// Generate some closed settlement windows with the given transfers. Will close the currently
    /// open settlement window if that contains any existing transfers.
    CreateSettlementWindows(Vec<Vec<TransferMessage>>),
}

#[cfg_attr(feature = "typescript_types", derive(TS))]
#[derive(Debug, Serialize, Deserialize)]
pub struct SettlementModelCreatedMessage {
    pub settlement_model: settlement_models::SettlementModel,
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
    HubAccountsCreated(Vec<participants::HubAccount>),
    SettlementModelCreated(SettlementModelCreatedMessage),
    SettlementWindowsCreated(Vec<settlement_windows::SettlementWindowId>),
}

#[cfg(feature = "typescript_types")]
export! {
    TransferCompleteMessage,
    TransferErrorMessage,
    AccountInitialization,
    ClientParticipant,
    TransferMessage,
    SettlementModelCreatedMessage,
    mojaloop_api::settlement::settlement::SettlementId,
    settlement_windows::SettlementWindowState,
    settlement_windows::SettlementWindowContent,
    settlement_windows::SettlementWindow,
    settlement_windows::SettlementWindowId,
    settlement_windows::SettlementWindowContentId,
    settlement_models::SettlementAccountType,
    settlement_models::SettlementDelay,
    settlement_models::SettlementGranularity,
    settlement_models::SettlementInterchange,
    settlement_models::LedgerAccountType,
    settlement_models::SettlementModel,
    participants::HubAccount,
    participants::HubAccountType,
    common::DateTime,
    common::Amount,
    common::Currency,
    common::ErrorResponse,
    common::ErrorInformation,
    common::MojaloopApiError,
    ServerMessage,
    ClientMessage => "clients/typescript/src/lib/protocol.ts"
}

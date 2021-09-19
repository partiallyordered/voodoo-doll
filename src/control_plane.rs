use std::collections::HashMap;
use warp::ws::Message;
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;
use fspiox_api::{common, transfer};
use thiserror::Error;
use crate::protocol;

pub(crate) struct ClientData {
    pub chan: mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>,
    pub participants: Vec<common::FspId>,
}

impl ClientData {
    pub fn send(&self, msg: &protocol::ServerMessage) {
        let msg_text = serde_json::to_string(msg).unwrap();
        println!("Sending message to client: {}", msg_text);
        if let Err(e) = self.chan.send(Ok(Message::text(msg_text))) {
            // The actual disconnect will be handled elsewhere
            println!("Error sending message. Client may have disconnected. Error: {:?}.", e);
        }
    }
}

pub(crate) type ClientId = usize;
pub(crate) type Clients = Arc<RwLock<HashMap<ClientId, ClientData>>>;

#[derive(Hash, PartialEq, Eq, Debug)]
pub(crate) enum FspiopMessageId {
    TransferId(transfer::TransferId),
}

/// FSPIOP messages that we've sent that we're expecting a callback for, mapped to the client that
/// requested them.
pub(crate) type InFlightFspiopMessages = Arc<RwLock<HashMap<FspiopMessageId, ClientId>>>;

// TODO: implement this properly to return an actual FSPIOP error. Or perhaps add warp-specific
// return value implementations to the fspiox-api FspiopError.
#[derive(Error, Debug)]
pub(crate) enum FspiopError {
    #[error("Request body couldn't be deserialized: {0}")]
    MalformedRequestBody(String),
}

use crate::types::basic::{Fee, SyncMode};
use crate::types::signature::Signature;
use serde::Serialize;

/// tx in transfer transaction
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Tx<M: Serialize> {
    /// messages to be executed
    #[serde(rename = "msg")]
    pub messages: Vec<M>,
    /// fee to be paid
    pub fee: Fee,
    /// extra metadata payload
    pub memo: String,
    /// signatures for each message
    pub signatures: Vec<Signature>,
}

/// transfer transaction
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Transaction<M: Serialize> {
    /// transaction to be broadcasted
    pub tx: Tx<M>,
    /// broadcasting mode
    pub mode: SyncMode,
}

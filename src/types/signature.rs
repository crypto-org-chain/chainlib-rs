use crate::error::Error;
use crate::types::basic::Fee;
use crate::types::key::PublicKeyWrap;
use crate::utils::codec::serde_to_str;
use serde::Serialize;

/// Signature used in Tx
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// signature payload
    pub signature: String,
    /// public key
    pub pub_key: PublicKeyWrap,
    /// global nonce
    pub account_number: u64,
    /// local nonce
    pub sequence: u64,
}

/// signing payload
#[derive(Serialize, Debug, Clone)]
pub struct SignDoc<M: Serialize> {
    /// global nonce
    #[serde(serialize_with = "serde_to_str")]
    pub account_number: u64,
    /// local nonce
    #[serde(serialize_with = "serde_to_str")]
    pub sequence: u64,
    /// network identifier
    pub chain_id: String,
    /// extra metadata payload
    pub memo: String,
    /// fee to be paid
    pub fee: Fee,
    /// messages to be executed
    pub msgs: Vec<M>,
}

impl<M: Serialize> SignDoc<M> {
    /// encode to amino-json
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let value = serde_json::to_value(self).map_err(|e| Error::SerializeError(e.to_string()))?;
        let sign_str = sorted_json::to_json(&value)
            .replace("\n", "")
            .replace(" ", "");
        Ok(sign_str.into_bytes())
    }
}

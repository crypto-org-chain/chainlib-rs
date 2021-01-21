/// ledger device signing
#[cfg(feature = "ledger")]
pub mod ledger_service;
/// in-mem signing -- for testing
pub mod private_key_service;

use crate::error::Error;
use crate::types::key::PublicKey;
use async_trait::async_trait;
use stdtx::Address;

/// signing backend trait
#[async_trait]
pub trait KeyService {
    /// return the public key
    fn public_key(&self) -> Result<PublicKey, Error>;

    /// Address returns a Bitcoin style account addresses: RIPEMD160(SHA256(pubkey))
    fn address(&self) -> Result<Address, Error>;

    /// sign a message, return base64 encoded string
    async fn sign(&self, msg: &[u8]) -> Result<String, Error>;
}

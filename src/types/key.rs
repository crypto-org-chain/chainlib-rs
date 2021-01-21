use crate::error::Error;
use hdwallet::secp256k1;
use hdwallet::ExtendedPrivKey;
use secp256k1::{All, Secp256k1};
use secp256k1::{Error as SecpError, PublicKey as InnerPublicKey, SecretKey};
use serde::Serialize;
use std::string::ToString;

/// Private key
#[derive(Debug, Clone)]
pub struct PrivateKey(SecretKey);

/// public key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(InnerPublicKey);

/// public key format used in transfer transaction
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct PublicKeyWrap {
    #[serde(rename = "type")]
    p_type: String,
    value: String,
}

impl From<PublicKey> for PublicKeyWrap {
    fn from(pubkey: PublicKey) -> PublicKeyWrap {
        Self {
            p_type: "tendermint/PubKeySecp256k1".to_string(),
            value: pubkey.to_string(),
        }
    }
}

impl AsRef<SecretKey> for PrivateKey {
    fn as_ref(&self) -> &SecretKey {
        &self.0
    }
}

impl PrivateKey {
    pub fn from_slice(slice: &[u8]) -> Result<Self, SecpError> {
        let secret_key = SecretKey::from_slice(slice)?;
        Ok(Self(secret_key))
    }
}

impl PublicKey {
    pub fn from_base64_str(pubkey_str: &str) -> Result<Self, Error> {
        let raw = base64::decode(pubkey_str)
            .map_err(|_e| Error::InputError("invalid publickey input".to_string()))?;
        let inner = InnerPublicKey::from_slice(&raw)?;
        Ok(Self(inner))
    }
}

impl From<&PrivateKey> for PublicKey {
    fn from(private_key: &PrivateKey) -> Self {
        let secp = Secp256k1::<All>::new();
        let public_key_inner = InnerPublicKey::from_secret_key(&secp, &private_key.0);
        Self(public_key_inner)
    }
}

impl From<ExtendedPrivKey> for PrivateKey {
    fn from(extended_priv_key: ExtendedPrivKey) -> Self {
        let secret_key = extended_priv_key.private_key;
        Self(secret_key)
    }
}

impl ToString for PublicKey {
    fn to_string(&self) -> String {
        let raw = self.0.serialize();
        base64::encode(&raw)
    }
}

impl AsRef<InnerPublicKey> for PublicKey {
    fn as_ref(&self) -> &InnerPublicKey {
        &self.0
    }
}

impl From<InnerPublicKey> for PublicKey {
    fn from(inner_pubkey: InnerPublicKey) -> Self {
        Self(inner_pubkey)
    }
}

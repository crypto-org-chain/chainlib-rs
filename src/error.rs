use crate::hd_wallet::mnemonic::MnemonicError;
use hdwallet::secp256k1;
use thiserror::Error;

/// Common errors
#[derive(Error, Debug)]
pub enum Error {
    /// wrong word counts etc.
    #[error("mnemonic error")]
    MnemonicError(#[from] MnemonicError),

    /// user input error
    #[error("invalid input: {0}")]
    InputError(String),

    /// cryptographic error
    #[error("secp error")]
    SecpError(#[from] secp256k1::Error),

    /// serialization failed
    #[error("serialize error: {0}")]
    SerializeError(String),

    /// device error
    #[error("ledger error: {0}")]
    LedgerError(String),

    /// grpc error
    #[cfg(feature = "grpc")]
    #[error("prost encode error")]
    ProstEncodeError(#[from] prost::EncodeError),

    /// json-rpc error
    #[error("client request error")]
    RequestError(#[from] reqwest::Error),

    /// general API error
    #[error("client error: {0}")]
    ClientError(String),
}

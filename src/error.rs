use crate::hd_wallet::mnemonic::MnemonicError;
use hdwallet::secp256k1;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("mnemonic error")]
    MnemonicError(#[from] MnemonicError),

    #[error("invalid input: {0}")]
    InputError(String),

    #[error("secp error")]
    SecpError(#[from] secp256k1::Error),

    #[error("serialize error: {0}")]
    SerializeError(String),

    #[error("ledger error: {0}")]
    LedgerError(String),

    #[cfg(feature = "grpc")]
    #[error("prost encode error")]
    ProstEncodeError(#[from] prost::EncodeError),

    #[error("client request error")]
    RequestError(#[from] reqwest::Error),

    #[error("client error: {0}")]
    ClientError(String),
}

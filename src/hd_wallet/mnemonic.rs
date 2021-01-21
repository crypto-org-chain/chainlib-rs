use crate::types::key::PrivateKey;

use crate::constant::FUNDRAISER_PATH;
use bip39::{Language, MnemonicType, Seed};
use hdwallet::{ChainPath, KeyChain};
use hdwallet::{DefaultKeyChain, ExtendedPrivKey};

/// BIP39 mnemonic code
pub struct Mnemonic {
    inner_mnemonic: bip39::Mnemonic,
    password: Option<String>,
}

/// errors from processing mnemonic
#[derive(thiserror::Error, Debug)]
pub enum MnemonicError {
    /// invalid word count
    #[error("input error: {0}")]
    InputError(String),

    /// unknown words etc.
    #[error("mnemonic error")]
    MnemonicError(#[from] anyhow::Error),

    /// invalid path etc.
    #[error("hdwallet error")]
    HdWalletError(hdwallet::error::Error),
}

impl From<hdwallet::error::Error> for MnemonicError {
    fn from(err: hdwallet::error::Error) -> MnemonicError {
        MnemonicError::HdWalletError(err)
    }
}

impl Mnemonic {
    /// create new Mnemonic
    pub fn new(word_count: u32, password: Option<String>) -> Result<Self, MnemonicError> {
        let flag = match word_count {
            12 => MnemonicType::Words12,
            15 => MnemonicType::Words15,
            18 => MnemonicType::Words18,
            21 => MnemonicType::Words21,
            24 => MnemonicType::Words24,
            _ => return Err(MnemonicError::InputError("invalid words count".to_string())),
        };
        let mnemonic = bip39::Mnemonic::new(flag, Language::English);
        let m = Mnemonic {
            inner_mnemonic: mnemonic,
            password,
        };
        Ok(m)
    }

    /// Create Mnemonic from words in string literal
    pub fn from_str(words: &str, password: Option<String>) -> Result<Self, MnemonicError> {
        let mnemonic = bip39::Mnemonic::from_phrase(words, Language::English)?;
        let m = Mnemonic {
            inner_mnemonic: mnemonic,
            password,
        };
        Ok(m)
    }

    /// Generates private key
    pub fn private_key(&self) -> Result<PrivateKey, MnemonicError> {
        let chain_path = ChainPath::from(FUNDRAISER_PATH);
        let password = self.password.clone().unwrap_or_default();
        let seed = Seed::new(&self.inner_mnemonic, &password)
            .as_bytes()
            .to_vec();
        let master_key = ExtendedPrivKey::with_seed(&seed)?;
        let key_chain = DefaultKeyChain::new(master_key);

        let (extended_private_key, _) = key_chain.derive_private_key(chain_path)?;
        Ok(extended_private_key.into())
    }
}

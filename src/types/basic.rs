use crate::constant::CRO;
use crate::utils::codec::serde_to_str;
use serde::Serialize;

/// sync mode when send the transaction
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    /// synchronous
    Sync,
    /// asynchronous
    Async,
    /// block-commit -- only for development
    Block,
}

/// denomination: 1Cro = 100_000_000 Basecro
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Denom {
    /// base unit
    Basecro,
    /// human unit
    Cro,
}

/// Amount
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Amount {
    denom: Denom,
    #[serde(serialize_with = "serde_to_str")]
    amount: u64,
}

impl Amount {
    /// create a new amount, whatever input will be a Benom::Basecro denomination result
    pub fn new(amount: u64, denom: Denom) -> Self {
        let amount = match denom {
            Denom::Basecro => amount,
            Denom::Cro => amount * CRO,
        };
        Self {
            denom: Denom::Basecro,
            amount,
        }
    }
}

/// transaction fee
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Fee {
    /// gas limit
    #[serde(serialize_with = "serde_to_str")]
    pub gas: u64,
    /// fee to be paid
    pub amount: Vec<Amount>,
}

impl Default for Fee {
    fn default() -> Self {
        Self {
            gas: 2000000,
            amount: vec![],
        }
    }
}

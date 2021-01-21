use crate::constant::ACCOUNT_ADDRESS_PREFIX;
use crate::types::basic::Amount;
use serde::Serialize;
use stdtx::Address;

/// the message in Tx
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Message<V: Serialize> {
    #[serde(rename = "type")]
    /// msg identifier
    pub transfer_type: String,
    /// msg payload
    pub value: V,
}

/// the value in Transfer
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TransferValue {
    /// sender
    pub from_address: String,
    /// receiver
    pub to_address: String,
    /// coints to send
    pub amount: Vec<Amount>,
}

impl TransferValue {
    /// create a new TransferValue
    pub fn new(from_address: Address, to_address: Address, amount: Amount) -> Self {
        Self {
            from_address: from_address.to_bech32(ACCOUNT_ADDRESS_PREFIX),
            to_address: to_address.to_bech32(ACCOUNT_ADDRESS_PREFIX),
            amount: vec![amount],
        }
    }
}

/// the message in transfer tx
pub type Transfer = Message<TransferValue>;

impl Transfer {
    /// create a new transfer message
    pub fn new(from_address: Address, to_address: Address, amount: Amount) -> Self {
        let transfer_value = TransferValue::new(from_address, to_address, amount);
        Self {
            transfer_type: "cosmos-sdk/MsgSend".into(),
            value: transfer_value,
        }
    }
}

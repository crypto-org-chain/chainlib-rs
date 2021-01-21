use crate::error::Error;
use crate::key_service::KeyService;
use crate::types::basic::{Amount, Fee, SyncMode};
use crate::types::signature::SignDoc;
use crate::types::signature::Signature;
use crate::types::transaction::{Transaction, Tx};
use serde::Serialize;

/// struct for building signed transactions
#[derive(Clone)]
pub struct TxBuilder<T: KeyService + Clone, M: Serialize + Clone> {
    /// signing backend
    pub key_service: T,
    /// network identifier
    pub chain_id: String,
    /// messages to be executed
    pub messages: Vec<M>,
    /// extra payload
    pub memo: String,
    /// global nonce
    pub account_number: u64,
    /// local nonce
    pub sequence: u64,
    /// fee to be paid
    pub fee: Option<Amount>,
    /// gas limit
    pub gas: Option<u64>,
}

impl<T, M> TxBuilder<T, M>
where
    T: KeyService + Clone,
    M: Serialize + Clone,
{
    /// instatiate new tx builder
    pub fn new(
        key_service: T,
        chain_id: String,
        memo: Option<String>,
        fee: Option<Amount>,
        gas: Option<u64>,
    ) -> Self {
        let memo = memo.unwrap_or_default();
        Self {
            fee,
            gas,
            memo,
            key_service,
            chain_id,
            sequence: 0,
            account_number: 0,
            messages: vec![],
        }
    }

    /// set global nonce
    pub fn set_account_number(&mut self, account_number: u64) -> &mut Self {
        self.account_number = account_number;
        self
    }

    /// set local nonce
    pub fn set_sequence(&mut self, sequence: u64) -> &mut Self {
        self.sequence = sequence;
        self
    }

    /// add tx message to be executed
    pub fn add_message(&mut self, msg: M) -> &mut Self {
        self.messages.push(msg);
        self
    }

    #[inline]
    fn get_fee(&self) -> Fee {
        let amount = if self.fee.is_some() {
            vec![self.fee.clone().unwrap()]
        } else {
            vec![]
        };
        Fee {
            gas: self.gas.unwrap_or(20000),
            amount,
        }
    }

    async fn sign(&mut self) -> Result<Signature, Error> {
        let fee = self.get_fee();
        let sign_doc = SignDoc {
            account_number: self.account_number,
            sequence: self.sequence,
            chain_id: self.chain_id.clone(),
            memo: self.memo.clone(),
            fee,
            msgs: self.messages.clone(),
        };
        let raw_doc = sign_doc.encode()?;
        let signature = self.key_service.sign(&raw_doc).await?;
        let public_key = self.key_service.public_key()?;

        let signature = Signature {
            signature,
            pub_key: public_key.into(),
            account_number: self.account_number,
            sequence: self.sequence,
        };
        Ok(signature)
    }

    /// finalize transaction
    pub async fn build(&mut self, sync_mode: SyncMode) -> Result<Transaction<M>, Error> {
        let signature = self.sign().await?;
        let fee = self.get_fee();
        let tx = Tx {
            messages: self.messages.clone(),
            fee,
            memo: self.memo.clone(),
            signatures: vec![signature],
        };
        let transaction = Transaction {
            tx,
            mode: sync_mode,
        };
        Ok(transaction)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constant::ACCOUNT_ADDRESS_PREFIX;
    use crate::hd_wallet::mnemonic::Mnemonic;
    use crate::key_service::private_key_service::PrivateKeyService;
    use crate::message::{Transfer, TransferValue};
    use crate::types::basic::{Amount, Denom};
    use crate::types::key::PublicKey;
    use stdtx::Address;

    #[tokio::test]
    async fn test_tx_builder() {
        let fee = Amount::new(100000, Denom::Basecro);
        let gas = Some(300000);
        let memo = None;
        let words = "dune car envelope chuckle elbow slight proud fury remove candy uphold puzzle call select sibling sport gadget please want vault glance verb damage gown";
        let mnemonic = Mnemonic::from_str(words, None).unwrap();
        let key_service = PrivateKeyService::new_from_mnemonic(mnemonic).unwrap();
        let chain_id = "test".to_string();
        let mut builder = TxBuilder::new(key_service, chain_id, memo, Some(fee.clone()), gas);
        let (_, to_address) =
            Address::from_bech32("cro1s2gsnugjhpzac8m7necv3527jp28z9w002najd").unwrap();
        let from_address = builder.key_service.address().unwrap();
        let amount = Amount::new(100000000, Denom::Basecro);
        let msg = Transfer::new(from_address, to_address, amount);
        builder.add_message(msg);
        let account_number = 0;
        let sequence = 0;
        let transfer = builder.build(SyncMode::Sync).await.unwrap();
        let transfer_expected = Transaction {
            tx: Tx {
                fee: Fee {
                    gas: 300000,
                    amount: vec![fee],
                },
                memo: "".into(),
                signatures: vec![
                    Signature {
                        signature: "xi3rvdsoZMXhWq7MlgAMXpoVIZ0kv7uB00OrSRS8wxwoZhojZ5uGZ4shobn3ztOev4M1k5WVcBvVd+zTvzRHCg==".into(),
                        pub_key: PublicKey::from_base64_str("AntL+UxMyJ9NZ9DGLp2v7a3dlSxiNXMaItyOXSRw8iYi").unwrap().into(),
                        account_number,
                        sequence,
                    }
                ],
                messages: vec![
                    Transfer {
                        transfer_type: "cosmos-sdk/MsgSend".into(),
                        value: TransferValue {
                            from_address: "cro1u9q8mfpzhyv2s43js7l5qseapx5kt3g2rf7ppf".into(),
                            to_address: to_address.to_bech32(ACCOUNT_ADDRESS_PREFIX),
                            amount: vec![Amount::new(100000000, Denom::Basecro)],
                        }
                    }
                ]
            },
            mode: SyncMode::Sync,
        };
        assert_eq!(transfer, transfer_expected);
    }
}

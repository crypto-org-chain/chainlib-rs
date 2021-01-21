use crate::constant::ACCOUNT_ADDRESS_PREFIX;
use crate::error::Error;
use crate::key_service::KeyService;
use crate::message::Msg;
use crate::proto::cosmos::bank::v1beta1::MsgSend;
use crate::proto::cosmos::base::v1beta1::Coin;
use crate::proto::cosmos::tx::v1beta1::{
    mode_info, AuthInfo, Fee, ModeInfo, SignDoc, SignerInfo, TxBody, TxRaw,
};

/// struct for building signed transactions
pub struct TxBuilder<T: KeyService + Clone> {
    /// signing backend
    pub key_service: T,
    chain_id: String,
    messages: Vec<Msg>,
    memo: Option<String>,
    timeout_height: u64,
    account_number: u64,
    sequence: u64,
    fee: Option<Fee>,
}

fn encode<T: prost::Message>(msg: &T) -> Result<Vec<u8>, Error> {
    let mut buf = vec![];
    prost::Message::encode(msg, &mut buf)?;
    Ok(buf)
}

impl<T: KeyService + Clone> TxBuilder<T> {
    /// instatiate new tx builder
    pub fn new(
        key_service: T,
        chain_id: String,
        memo: Option<String>,
        timeout_height: u64,
        fee: Option<Fee>,
    ) -> Self {
        Self {
            key_service,
            messages: vec![],
            chain_id,
            memo,
            timeout_height,
            account_number: 0,
            sequence: 0,
            fee,
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

    /// add a message to execute
    pub fn add_message(&mut self, msg: Msg) -> &mut Self {
        self.messages.push(msg);
        self
    }

    /// public key encoded as protobuf Any
    pub fn pk_any(&self) -> Result<prost_types::Any, Error> {
        let pk = self.key_service.public_key()?;
        let mut buf = Vec::new();
        prost::Message::encode(&pk.as_ref().serialize().to_vec(), &mut buf)?;
        let pk_any = prost_types::Any {
            type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
            value: buf,
        };
        Ok(pk_any)
    }

    /// raw tx bytes
    pub fn raw_tx_body(&self) -> Result<Vec<u8>, Error> {
        let body = TxBody {
            messages: self.messages.iter().map(|msg| msg.clone().into()).collect(),
            memo: self.memo.clone().unwrap_or_default(),
            timeout_height: self.timeout_height,
            extension_options: Default::default(),
            non_critical_extension_options: Default::default(),
        };
        // A protobuf serialization of a TxBody
        let mut body_buf = Vec::new();
        prost::Message::encode(&body, &mut body_buf)?;
        Ok(body_buf)
    }

    /// signing metadata
    pub fn auth_info(&self) -> Result<AuthInfo, Error> {
        let single = mode_info::Single { mode: 1 };

        let mode = Some(ModeInfo {
            sum: Some(mode_info::Sum::Single(single)),
        });

        let pk_any = self.pk_any()?;

        let signer_info = SignerInfo {
            public_key: Some(pk_any),
            mode_info: mode,
            sequence: self.sequence,
        };

        Ok(AuthInfo {
            signer_infos: vec![signer_info],
            fee: self.fee.clone(),
        })
    }

    /// create a MsgSend type message
    pub fn create_msg(&self, to_address: String, amount: Coin) -> Result<Msg, Error> {
        let from_address = self.key_service.address()?;
        let address_str = from_address.to_bech32(ACCOUNT_ADDRESS_PREFIX);
        let msg = MsgSend {
            from_address: address_str,
            to_address,
            amount: vec![amount],
        };
        let buf = encode(&msg)?;
        let any = prost_types::Any {
            type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
            value: buf,
        };
        Ok(Msg::from(any))
    }

    /// get signing payload
    pub fn sign_doc(&self) -> Result<SignDoc, Error> {
        let body_bytes = self.raw_tx_body()?;
        let auth_info_bytes = encode(&self.auth_info()?)?;
        let sign_doc = SignDoc {
            body_bytes,
            auth_info_bytes,
            chain_id: self.chain_id.clone(),
            account_number: self.account_number,
        };
        Ok(sign_doc)
    }

    /// build a tx
    pub async fn build(&self) -> Result<String, Error> {
        let sign_doc = self.sign_doc()?;
        let signdoc_buf = encode(&sign_doc)?;
        let signature_base64 = self.key_service.sign(&signdoc_buf).await?;
        let signature = base64::decode(signature_base64).map_err(|e| {
            Error::SerializeError(format!("invalid base64 signature, decode error: {:?}", e))
        })?;
        let body_bytes = self.raw_tx_body()?;
        let auth_info_bytes = encode(&self.auth_info()?)?;
        let tx_raw = TxRaw {
            body_bytes,
            auth_info_bytes,
            signatures: vec![signature],
        };
        let bytes = encode(&tx_raw)?;
        let string_b64 = base64::encode(bytes);
        Ok(string_b64)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hd_wallet::mnemonic::Mnemonic;
    use crate::key_service::private_key_service::PrivateKeyService;
    use prost::Message;

    #[tokio::test]
    async fn test_tx_buider() {
        let words = "dune car envelope chuckle elbow slight proud fury remove candy uphold puzzle call select sibling sport gadget please want vault glance verb damage gown";
        let mnemonic = Mnemonic::from_str(words, None).unwrap();
        let key_service = PrivateKeyService::new_from_mnemonic(mnemonic).unwrap();
        let chain_id = "test".into();
        let fee = Fee {
            amount: vec![Coin {
                denom: "basecro".to_string(),
                amount: 10000.to_string(),
            }],
            gas_limit: 300000,
            payer: "".to_string(),
            granter: "".to_string(),
        };
        let mut builder = TxBuilder::new(key_service, chain_id, None, 1, Some(fee));
        builder.set_account_number(9).set_sequence(4);

        // test public key
        let pk_any = builder.pk_any().unwrap();
        let pk_buf = vec![
            10, 33, 2, 123, 75, 249, 76, 76, 200, 159, 77, 103, 208, 198, 46, 157, 175, 237, 173,
            221, 149, 44, 98, 53, 115, 26, 34, 220, 142, 93, 36, 112, 242, 38, 34,
        ];
        assert_eq!(
            pk_any,
            prost_types::Any {
                type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
                value: pk_buf,
            }
        );

        // test auth info
        let auth_info = builder.auth_info().unwrap();
        let auth_info_bytes = vec![
            10, 80, 10, 70, 10, 31, 47, 99, 111, 115, 109, 111, 115, 46, 99, 114, 121, 112, 116,
            111, 46, 115, 101, 99, 112, 50, 53, 54, 107, 49, 46, 80, 117, 98, 75, 101, 121, 18, 35,
            10, 33, 2, 123, 75, 249, 76, 76, 200, 159, 77, 103, 208, 198, 46, 157, 175, 237, 173,
            221, 149, 44, 98, 53, 115, 26, 34, 220, 142, 93, 36, 112, 242, 38, 34, 18, 4, 10, 2, 8,
            1, 24, 4, 18, 22, 10, 16, 10, 7, 98, 97, 115, 101, 99, 114, 111, 18, 5, 49, 48, 48, 48,
            48, 16, 224, 167, 18,
        ];
        assert_eq!(auth_info, AuthInfo::decode(&*auth_info_bytes).unwrap());

        // add msg
        let to_address = "cro1fj6jpmuykvra4kxrw0cp20e4vx4r8eda8q3yn9".into();
        let amount = Coin {
            denom: "basecro".into(),
            amount: 100000000.to_string(),
        };
        let msg = builder.create_msg(to_address, amount).unwrap();
        builder.add_message(msg);

        // test signature
        let sign_doc = builder.sign_doc().unwrap();
        let raw_sign_doc = encode(&sign_doc).unwrap();
        let signature = builder.key_service.sign(&raw_sign_doc).await.unwrap();
        assert_eq!(signature, "jlqBo5nxRbq2RIYpjo4+gjevBEDALw+IjmqEPu4igfIgD8l4/CR3vmetHvhpyeQaYZ/bJJfehT6Z/RpxofJnxA==");

        // // test tx raw
        let tx = builder.build().await.unwrap();
        let tx_expect = "CpMBCo4BChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEm4KKmNybzF1OXE4bWZwemh5djJzNDNqczdsNXFzZWFweDVrdDNnMnJmN3BwZhIqY3JvMWZqNmpwbXV5a3ZyYTRreHJ3MGNwMjBlNHZ4NHI4ZWRhOHEzeW45GhQKB2Jhc2Vjcm8SCTEwMDAwMDAwMBgBEmoKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQJ7S/lMTMifTWfQxi6dr+2t3ZUsYjVzGiLcjl0kcPImIhIECgIIARgEEhYKEAoHYmFzZWNybxIFMTAwMDAQ4KcSGkCOWoGjmfFFurZEhimOjj6CN68EQMAvD4iOaoQ+7iKB8iAPyXj8JHe+Z60e+GnJ5Bphn9skl96FPpn9GnGh8mfE";
        assert_eq!(tx, tx_expect);
    }
}

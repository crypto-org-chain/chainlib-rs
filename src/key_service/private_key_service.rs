use crate::error::Error;
use crate::hd_wallet::mnemonic::Mnemonic;
use crate::types::key::{PrivateKey, PublicKey};

use crate::key_service::KeyService;
use async_trait::async_trait;
use bitcoin_hashes::{ripemd160, sha256};
use bitcoin_hashes::{Hash, HashEngine};
use hdwallet::secp256k1::{Message, Secp256k1};
use stdtx::address::{Address, ADDRESS_SIZE};

/// stores private key
#[derive(Clone)]
pub struct PrivateKeyService {
    pub private_key: PrivateKey,
}

impl PrivateKeyService {
    /// create a new KeyService from Mnemonic
    pub fn new_from_mnemonic(mnemonic: Mnemonic) -> Result<Self, Error> {
        let private_key = mnemonic.private_key()?;
        Ok(Self { private_key })
    }

    /// create a new KeyService from a PrivateKey
    pub fn new(private_key: PrivateKey) -> Self {
        Self { private_key }
    }

    /// sig msg
    fn sign(&self, msg: &[u8]) -> Result<String, Error> {
        let mut engine = sha256::Hash::engine();
        engine.input(msg);
        let hash = sha256::Hash::from_engine(engine);
        let message = Message::from_slice(hash.as_inner())?;
        let signer = Secp256k1::signing_only();
        let signature = signer.sign(&message, self.private_key.as_ref());
        let raw = signature.serialize_compact();
        let signature_str = base64::encode(&raw);
        Ok(signature_str)
    }
}

#[async_trait]
impl KeyService for PrivateKeyService {
    fn public_key(&self) -> Result<PublicKey, Error> {
        Ok(PublicKey::from(&self.private_key))
    }

    /// Address returns a Bitcoin style account addresses: RIPEMD160(SHA256(pubkey))
    fn address(&self) -> Result<Address, Error> {
        let pubkey = PublicKey::from(&self.private_key);
        let pubkey_bytes = pubkey.as_ref().serialize();
        let mut engine = sha256::Hash::engine();
        engine.input(&pubkey_bytes);
        let sha = sha256::Hash::from_engine(engine);
        let mut engine = ripemd160::Hash::engine();
        engine.input(sha.as_inner());
        let raw = ripemd160::Hash::from_engine(engine);
        let bits = raw.into_inner();
        if bits.len() != ADDRESS_SIZE {
            return Err(Error::InputError(
                "invalid bits length to generate address".to_string(),
            ));
        }
        let mut raw = [0; ADDRESS_SIZE];
        raw.copy_from_slice(&bits);
        Ok(raw.into())
    }

    /// sig msg
    async fn sign(&self, msg: &[u8]) -> Result<String, Error> {
        let result = self.sign(msg)?;
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constant::ACCOUNT_ADDRESS_PREFIX;

    #[tokio::test]
    async fn test_hd_key() {
        let words = "dune car envelope chuckle elbow slight proud fury remove candy uphold puzzle call select sibling sport gadget please want vault glance verb damage gown";
        let mnemonic = Mnemonic::from_str(words, None).unwrap();
        let private_key_service = PrivateKeyService::new_from_mnemonic(mnemonic).unwrap();

        // test address
        let address = private_key_service.address().unwrap();
        assert_eq!(
            address.to_bech32(ACCOUNT_ADDRESS_PREFIX),
            "cro1u9q8mfpzhyv2s43js7l5qseapx5kt3g2rf7ppf"
        );

        // test private key
        let private_raw = base64::decode("1Jp5fbY7YcFI0XZ+YW/xXD3ZyDtjy6YcIY6hcvI4Yio=").unwrap();
        assert_eq!(
            private_key_service.private_key.as_ref(),
            PrivateKey::from_slice(&private_raw).unwrap().as_ref()
        );

        // test public key
        let public_key = PublicKey::from(&private_key_service.private_key);
        let pubkey_str = public_key.to_string();
        assert_eq!(pubkey_str, "AntL+UxMyJ9NZ9DGLp2v7a3dlSxiNXMaItyOXSRw8iYi");

        // test sign
        let sign_msg = vec![
            123, 34, 97, 99, 99, 111, 117, 110, 116, 95, 110, 117, 109, 98, 101, 114, 34, 58, 34,
            48, 34, 44, 34, 99, 104, 97, 105, 110, 95, 105, 100, 34, 58, 34, 116, 101, 115, 116,
            34, 44, 34, 102, 101, 101, 34, 58, 123, 34, 97, 109, 111, 117, 110, 116, 34, 58, 91,
            123, 34, 97, 109, 111, 117, 110, 116, 34, 58, 34, 49, 48, 48, 48, 48, 48, 34, 44, 34,
            100, 101, 110, 111, 109, 34, 58, 34, 98, 97, 115, 101, 99, 114, 111, 34, 125, 93, 44,
            34, 103, 97, 115, 34, 58, 34, 51, 48, 48, 48, 48, 48, 34, 125, 44, 34, 109, 101, 109,
            111, 34, 58, 34, 34, 44, 34, 109, 115, 103, 115, 34, 58, 91, 123, 34, 116, 121, 112,
            101, 34, 58, 34, 99, 111, 115, 109, 111, 115, 45, 115, 100, 107, 47, 77, 115, 103, 83,
            101, 110, 100, 34, 44, 34, 118, 97, 108, 117, 101, 34, 58, 123, 34, 97, 109, 111, 117,
            110, 116, 34, 58, 91, 123, 34, 97, 109, 111, 117, 110, 116, 34, 58, 34, 49, 48, 48, 48,
            48, 48, 48, 48, 48, 34, 44, 34, 100, 101, 110, 111, 109, 34, 58, 34, 98, 97, 115, 101,
            99, 114, 111, 34, 125, 93, 44, 34, 102, 114, 111, 109, 95, 97, 100, 100, 114, 101, 115,
            115, 34, 58, 34, 99, 114, 111, 49, 117, 57, 113, 56, 109, 102, 112, 122, 104, 121, 118,
            50, 115, 52, 51, 106, 115, 55, 108, 53, 113, 115, 101, 97, 112, 120, 53, 107, 116, 51,
            103, 50, 114, 102, 55, 112, 112, 102, 34, 44, 34, 116, 111, 95, 97, 100, 100, 114, 101,
            115, 115, 34, 58, 34, 99, 114, 111, 49, 119, 97, 118, 48, 114, 118, 101, 110, 107, 117,
            48, 57, 113, 56, 114, 113, 120, 50, 110, 118, 117, 55, 119, 100, 108, 54, 106, 121, 53,
            100, 120, 48, 48, 48, 57, 117, 108, 106, 34, 125, 125, 93, 44, 34, 115, 101, 113, 117,
            101, 110, 99, 101, 34, 58, 34, 48, 34, 125,
        ];
        let s = private_key_service.sign(&sign_msg).unwrap();
        let s_expect = "bpPVZg1frGFAKM54i5Wr9PRcg31wk4vBNruYUuN9O9QvIJs+rFshRqZlhd++qBQYUvMdhHO4g/0UuB7JRaESvA==";
        println!("{}", s);
        assert_eq!(s, s_expect);
    }
}

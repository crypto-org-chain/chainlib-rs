use crate::ledger_app::CryptoApp;
use crate::ledger_app::PubkeyAddress;
use async_trait::async_trait;
use hdwallet::secp256k1::PublicKey as InnerPublicKey;
use ledger_transport::APDUTransport;
use std::sync::Arc;
use zx_bip44::BIP44Path;

use crate::error::Error;
use crate::key_service::KeyService;
use crate::types::key::PublicKey;

/// Hedger Service
#[derive(Clone)]
pub struct LedgerServiceHID {
    /// account address prefix
    pub acc_address_prefix: String,
    /// chain path
    pub path: Arc<BIP44Path>,
    /// crypto app of ledger
    pub app: Arc<CryptoApp>,
    /// public key and address
    pubkey_address: PubkeyAddress,
    /// confirmation on ledger or not
    pub require_confirmation: bool,
}

impl std::fmt::Debug for LedgerServiceHID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LedgerService")
            .field("app", &"CryptoApp")
            .field("require_confirmation", &self.require_confirmation)
            .finish()
    }
}

impl LedgerServiceHID {
    /// create a new LedgerService
    pub async fn new(
        acc_address_prefix: String,
        chain_path: &str,
        require_confirmation: bool,
    ) -> Result<Self, Error> {
        let wrapper = ledger::TransportNativeHID::new().map_err(|e| {
            let msg = format!("can't find ledger device: {:?}, see more: https://support.ledger.com/hc/en-us/articles/115005165269-Fix-connection-issues", e);
            Error::LedgerError(msg)
        })?;
        let transport = APDUTransport {
            transport_wrapper: Box::new(wrapper),
        };
        let app = CryptoApp::new(transport);
        let app_info = app
            .get_app_info()
            .await
            .map_err(|e| Error::LedgerError(format!("get app info failed: {:?}", e)))?;
        log::debug!("app info: {:?}", app_info);
        if app_info.app_name.to_lowercase() != "cryp" {
            return Err(Error::LedgerError(format!(
                "{} not CRO app",
                app_info.app_name
            )));
        }
        let app_version = app
            .get_version()
            .await
            .map_err(|e| Error::LedgerError(format!("get version failed: {:?}", e)))?;
        log::debug!("app version: {:?}", app_version);
        if app_version.major != 2 {
            return Err(Error::LedgerError(
                "only support v2 major version".to_string(),
            ));
        }

        let path = BIP44Path::from_string(chain_path)
            .map_err(|_e| Error::InputError("input invalid hd path".to_string()))?;

        // get public key and address
        let pubkey_address = app
            .get_pubkey_address(&acc_address_prefix, &path, false)
            .await
            .map_err(|e| Error::LedgerError(format!("get address failed: {:?}", e)))?;

        Ok(Self {
            acc_address_prefix,
            path: Arc::new(path),
            pubkey_address,
            app: Arc::new(app),
            require_confirmation,
        })
    }
}

#[async_trait]
impl KeyService for LedgerServiceHID {
    fn public_key(&self) -> Result<PublicKey, Error> {
        let public_key_raw = self.pubkey_address.public_key;
        let pubkey = InnerPublicKey::from_slice(&public_key_raw)
            .map_err(|e| Error::InputError(format!("invalid public key: {:?}", e)))?;
        Ok(pubkey.into())
    }

    fn address(&self) -> Result<stdtx::Address, Error> {
        let address_str = self.pubkey_address.address.clone();
        let (_, address) = stdtx::Address::from_bech32(address_str)
            .map_err(|e| Error::InputError(format!("invalid address: {:?}", e)))?;
        Ok(address)
    }

    async fn sign(&self, msg: &[u8]) -> Result<String, Error> {
        let raw = self
            .app
            .sign(&self.path, msg)
            .await
            .map_err(|e| Error::LedgerError(e.to_string()))?;
        let signature_str = base64::encode(&raw);
        Ok(signature_str)
    }
}

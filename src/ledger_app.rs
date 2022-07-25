// Copyright (c) 2020, Zondax GmbH (licensed under the Apache License Version 2.0)
// Modifications Copyright (c) 2021-present Crypto.org(licensed under the Apache License, Version 2.0)

//! Support library for Crypto Ledger Nano S/X apps

#![deny(warnings, trivial_casts, trivial_numeric_casts)]
#![deny(unused_import_braces, unused_qualifications)]
#![deny(missing_docs)]

use ledger_transport::{APDUCommand, APDUErrorCodes, APDUTransport};
use ledger_zondax_generic::{
    map_apdu_error_description, AppInfo, ChunkPayloadType, DeviceInfo, LedgerAppError, Version,
};
use std::str;
use zx_bip44::BIP44Path;

const CLA: u8 = 0x55;
const INS_GET_ADDR_SECP256K1: u8 = 0x04;
const INS_SIGN_SECP256K1: u8 = 0x02;

const PK_LEN: usize = 33;
const SIGNATURE_LEN: usize = 65;

/// Ledger App
pub struct CryptoApp {
    apdu_transport: APDUTransport,
}

type PublicKey = [u8; PK_LEN];

/// Kusama address (includes pubkey and the corresponding ss58 address)
#[allow(dead_code)]
#[derive(Clone)]
pub struct PubkeyAddress {
    /// Public Key
    pub public_key: PublicKey,
    /// Address (exposed as SS58)
    pub address: String,
}

type Signature = [u8; SIGNATURE_LEN];

impl CryptoApp {
    /// Connect to the Ledger App
    pub fn new(apdu_transport: APDUTransport) -> Self {
        CryptoApp { apdu_transport }
    }

    fn cla(&self) -> u8 {
        CLA
    }

    /// Retrieve the app version
    pub async fn get_version(&self) -> Result<Version, LedgerAppError> {
        ledger_zondax_generic::get_version(self.cla(), &self.apdu_transport).await
    }

    /// Retrieve the app info
    pub async fn get_app_info(&self) -> Result<AppInfo, LedgerAppError> {
        ledger_zondax_generic::get_app_info(&self.apdu_transport).await
    }

    /// Retrieve the device info
    pub async fn get_device_info(&self) -> Result<DeviceInfo, LedgerAppError> {
        ledger_zondax_generic::get_device_info(&self.apdu_transport).await
    }

    /// Retrieves the public key and address
    pub async fn get_pubkey_address(
        &self,
        acc_address_prefix: &str,
        path: &BIP44Path,
        require_confirmation: bool,
    ) -> Result<PubkeyAddress, LedgerAppError> {
        let mut data = vec![];
        let acc_address_prefix_len = acc_address_prefix.as_bytes().len();
        data.push(acc_address_prefix_len as u8);
        let mut acc_address_prefix_raw = acc_address_prefix.as_bytes().to_vec();
        data.append(&mut acc_address_prefix_raw);
        let mut serialized_path = path.serialize();
        data.append(&mut serialized_path);
        let p1 = if require_confirmation { 1 } else { 0 };

        let command = APDUCommand {
            cla: self.cla(),
            ins: INS_GET_ADDR_SECP256K1,
            p1,
            p2: 0x00,
            data,
        };

        log::debug!("apdu command: {:?}", command);

        let response = self.apdu_transport.exchange(&command).await?;
        if response.retcode != 0x9000 {
            return Err(LedgerAppError::AppSpecific(
                response.retcode,
                map_apdu_error_description(response.retcode).to_string(),
            ));
        }

        log::info!("Received response {}", response.data.len());
        if response.data.len() < PK_LEN {
            return Err(LedgerAppError::InvalidPK);
        }

        let mut pubkey_address = PubkeyAddress {
            public_key: [0; PK_LEN],
            address: "".to_string(),
        };

        pubkey_address
            .public_key
            .copy_from_slice(&response.data[..PK_LEN]);
        pubkey_address.address = str::from_utf8(&response.data[PK_LEN..])
            .map_err(|_e| LedgerAppError::Utf8)?
            .to_owned();
        log::debug!("address: {:?}", pubkey_address.address);
        Ok(pubkey_address)
    }

    /// Sign a transaction
    pub async fn sign(
        &self,
        path: &BIP44Path,
        message: &[u8],
    ) -> Result<Signature, LedgerAppError> {
        let serialized_path = path.serialize();
        let start_command = APDUCommand {
            cla: self.cla(),
            ins: INS_SIGN_SECP256K1,
            p1: ChunkPayloadType::Init as u8,
            p2: 0x00,
            data: serialized_path,
        };

        log::info!("sign ->");
        let response =
            ledger_zondax_generic::send_chunks(&self.apdu_transport, &start_command, message)
                .await?;
        log::info!("sign OK");

        if response.data.is_empty() && response.retcode == APDUErrorCodes::NoError as u16 {
            return Err(LedgerAppError::NoSignature);
        }

        // Last response should contain the answer
        if response.data.len() < SIGNATURE_LEN {
            return Err(LedgerAppError::InvalidSignature);
        }

        log::info!("{}", hex::encode(&response.data[..]));

        let mut sig: Signature = [0u8; SIGNATURE_LEN];
        sig.copy_from_slice(&response.data[..SIGNATURE_LEN]);

        Ok(sig)
    }
}

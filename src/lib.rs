#![deny(unsafe_code, unstable_features)]
//! # Crypto.org Chain Rust library
//!
//! This crate helps with creating HD wallets and
//! signing [Crypto.org Chain](https://github.com/crypto-org-chain/chain-main) transfer transactions offline.

/// client for communication with chain-main API / RPC
pub mod client;
/// common constants used in Crypto.org Chain
pub mod constant;
/// possible errors
pub mod error;
/// utils for HD wallet operations
pub mod hd_wallet;
/// trait for abstracting over a signing backend
pub mod key_service;
/// communication with the Ledger device app
#[cfg(feature = "ledger")]
pub mod ledger_app;
/// transaction message types
pub mod message;
/// protobuf types
#[cfg(feature = "grpc")]
pub mod proto;
/// utils for creating a transaction
pub mod tx_builder;
/// common data types (tx, sig, key)
pub mod types;
/// utitilies for ledger communication
pub mod utils;

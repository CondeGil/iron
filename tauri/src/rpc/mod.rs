pub mod commands;
mod error;
mod send_transaction;

use std::{collections::HashMap, str::FromStr};

use ethers::{
    abi::AbiEncode,
    prelude::SignerMiddleware,
    providers::{Middleware, ProviderError},
    signers::Signer,
    types::{transaction::eip712, Address, Bytes},
};
use jsonrpc_core::{ErrorCode, IoHandler, Params};
use serde_json::json;

pub use self::error::{Error, Result};
use self::send_transaction::SendTransaction;
use crate::{
    networks::Networks,
    types::GlobalState,
    wallets::{WalletControl, Wallets},
};

pub struct Handler {
    io: IoHandler,
}

impl Default for Handler {
    fn default() -> Self {
        let mut res = Self {
            io: IoHandler::default(),
        };
        res.add_handlers();
        res
    }
}

fn ethers_to_jsonrpc_error(e: ProviderError) -> jsonrpc_core::Error {
    // TODO: probable handle more error types here
    match e {
        ProviderError::JsonRpcClientError(e) => {
            if let Some(e) = e.as_error_response() {
                jsonrpc_core::Error {
                    code: ErrorCode::ServerError(e.code),
                    data: e.data.clone(),
                    message: e.message.clone(),
                }
            } else if e.as_serde_error().is_some() {
                jsonrpc_core::Error::invalid_request()
            } else {
                jsonrpc_core::Error::internal_error()
            }
        }
        _ => jsonrpc_core::Error::internal_error(),
    }
}

impl Handler {
    pub async fn handle(&self, request: String) -> Option<String> {
        self.io.handle_request(&request).await
    }

    fn add_handlers(&mut self) {
        macro_rules! self_handler {
            ($name:literal, $fn:path) => {
                self.io
                    .add_method($name, |params: Params| async move { $fn(params).await });
            };
        }

        macro_rules! provider_handler {
            ($name:literal) => {
                self.io.add_method($name, |params: Params| async move {
                    let provider = Networks::read().await.get_current_provider();

                    let res: jsonrpc_core::Result<serde_json::Value> = provider
                        .request::<_, serde_json::Value>($name, params)
                        .await
                        .map_err(ethers_to_jsonrpc_error);
                    res
                });
            };
        }

        // delegate directly to provider
        provider_handler!("eth_estimateGas");
        provider_handler!("eth_call");
        provider_handler!("eth_blockNumber");
        provider_handler!("net_version");

        // handle internally
        self_handler!("eth_accounts", Self::accounts);
        self_handler!("eth_requestAccounts", Self::accounts);
        self_handler!("eth_chainId", Self::chain_id);
        self_handler!("eth_sendTransaction", Self::send_transaction);
        self_handler!("eth_sign", Self::eth_sign);

        self_handler!("personal_sign", Self::eth_sign);
        self_handler!("metamask_getProviderState", Self::provider_state);

        self_handler!("wallet_switchEthereumChain", Self::switch_chain);

        self_handler!("eth_signTypedData", Self::eth_sign_typed_data_v4);
        self_handler!("eth_signTypedData_v4", Self::eth_sign_typed_data_v4);
    }

    async fn accounts(_: Params) -> jsonrpc_core::Result<serde_json::Value> {
        let wallets = Wallets::read().await;
        let address = wallets.get_current_wallet().get_current_address().await;

        Ok(json!([address]))
    }

    async fn chain_id(_: Params) -> jsonrpc_core::Result<serde_json::Value> {
        let networks = Networks::read().await;
        let network = networks.get_current_network();
        Ok(json!(network.chain_id_hex()))
    }

    async fn provider_state(_: Params) -> jsonrpc_core::Result<serde_json::Value> {
        let networks = Networks::read().await;
        let wallets = Wallets::read().await;

        let network = networks.get_current_network();
        let address = wallets.get_current_wallet().get_current_address().await;

        Ok(json!({
            "isUnlocked": true,
            "chainId": network.chain_id_hex(),
            "networkVersion": network.name,
            "accounts": [address],
        }))
    }

    async fn switch_chain(params: Params) -> jsonrpc_core::Result<serde_json::Value> {
        let params = params.parse::<Vec<HashMap<String, String>>>().unwrap();
        let chain_id_str = params[0].get("chainId").unwrap().clone();
        let chain_id = u32::from_str_radix(&chain_id_str[2..], 16).unwrap();

        let mut networks = Networks::write().await;
        match networks.set_current_network_by_id(chain_id) {
            Ok(_) => Ok(serde_json::Value::Null),
            Err(e) => Err(jsonrpc_core::Error::invalid_params(e.to_string())),
        }
    }

    async fn send_transaction<T: Into<serde_json::Value>>(
        params: T,
    ) -> jsonrpc_core::Result<serde_json::Value> {
        // TODO: should we scope these rwlock reads so they don't stick during sining?
        let networks = Networks::read().await;
        let wallets = Wallets::read().await;

        let network = networks.get_current_network();
        let wallet = wallets.get_current_wallet();

        let signer = wallet
            .build_signer(network.chain_id)
            .await
            .map_err(|e| Error::SignerBuild(e.to_string()))?;

        let mut sender = SendTransaction::default();

        let sender = sender
            .set_params(params.into())
            .set_chain_id(network.chain_id)
            .set_signer(SignerMiddleware::new(network.get_provider(), signer))
            .estimate_gas()
            .await;

        if network.is_dev() && wallet.is_dev() {
            sender.skip_dialog();
        }

        let result = sender.finish().await;

        match result {
            Ok(res) => Ok(res.tx_hash().encode_hex().into()),
            Err(e) => Ok(e.to_string().into()),
        }
    }

    async fn eth_sign(params: Params) -> jsonrpc_core::Result<serde_json::Value> {
        let params = params.parse::<Vec<Option<String>>>().unwrap();
        let msg = params[0].as_ref().cloned().unwrap();
        let address = Address::from_str(&params[1].as_ref().cloned().unwrap()).unwrap();

        // TODO: ensure from == signer

        let networks = Networks::read().await;
        let network = networks.get_current_network();
        let provider = network.get_provider();
        let signer = Wallets::read()
            .await
            .get_current_wallet()
            .build_signer(network.chain_id)
            .await
            .unwrap();
        let signer = SignerMiddleware::new(provider, signer);

        let bytes = Bytes::from_str(&msg).unwrap();
        let res = signer.sign(bytes, &address).await;

        match res {
            Ok(res) => Ok(format!("0x{}", res).into()),
            Err(e) => Ok(e.to_string().into()),
        }
    }

    async fn eth_sign_typed_data_v4(params: Params) -> jsonrpc_core::Result<serde_json::Value> {
        let params = params.parse::<Vec<Option<String>>>().unwrap();
        let _address = Address::from_str(&params[0].as_ref().cloned().unwrap()).unwrap();
        let data = params[1].as_ref().cloned().unwrap();
        let typed_data: eip712::TypedData = serde_json::from_str(&data).unwrap();

        let networks = Networks::read().await;
        let network = networks.get_current_network();
        let signer = Wallets::read()
            .await
            .get_current_wallet()
            .build_signer(network.chain_id)
            .await
            .unwrap();
        // TODO: ensure from == signer

        let res = signer.sign_typed_data(&typed_data).await;

        match res {
            Ok(res) => Ok(format!("0x{}", res).into()),
            Err(e) => Ok(e.to_string().into()),
        }
    }
}

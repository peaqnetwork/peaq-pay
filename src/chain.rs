use codec::{Decode, Encode};
use keyring::sr25519::sr25519;
use log::trace;
use scale_info::TypeInfo;
use serde::Serialize;
use sp_core::RuntimeDebug;
use sp_runtime::{AccountId32 as AccountId, MultiAddress};
use std::{error::Error, str::FromStr};
use subclient::{rpc::WsRpcClient, Api, Pair};
use substrate_api_client::{self as subclient, rpc as subclient_rpc};

use crate::utils;

type BlockNumber = u32;

pub enum ChainError {
    Error(String),
    None,
}
#[derive(
    Serialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Default,
    RuntimeDebug,
    TypeInfo,
)]
pub struct Timepoint<BlockNumber> {
    pub height: BlockNumber,
    pub index: u32,
}

pub struct ApproveTransactionParams {
    pub ws_url: String,
    pub threshold: u16,
    pub max_weight: u64,
    pub other_signatories: Vec<String>,
    pub timepoint: Timepoint<BlockNumber>,
    pub call_hash: String,
    pub seed: String,
}

pub fn fund_multisig_wallet(
    ws_url: String,
    address: String,
    amount: subclient::Balance,
    seed: String,
) -> Option<ChainError> {
    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from: sr25519::Pair = utils::generate_pair(&seed.as_str());
    if let Ok(api) = init_api_client(from.clone(), &ws_url.as_str()) {
        let to = sr25519::Public::from_str(&address.as_str()).unwrap();
        let to = AccountId::decode(&mut &to.0[..]).unwrap();
        let from_account = AccountId::decode(&mut &from.public().0[..]).unwrap();

        let mut former_balance: subclient::Balance = 0;

        if let Some(account) = api.get_account_data(&to).unwrap() {
            former_balance = account.free;
        }

        match api.get_account_data(&from_account).unwrap() {
            Some(account) => {
                if account.free < amount {
                    return Some(ChainError::Error("Insufficient Funds".to_string()));
                }
            }
            None => {
                return Some(ChainError::Error(
                    "Can't fetch account data from chain".to_string(),
                ));
            }
        }
        // generate extrinsic
        let xt = api.balance_transfer(MultiAddress::Id(to.clone()), amount);

        // send and watch extrinsic until finalized
        let result = api.send_extrinsic(xt.hex_encode(), subclient::XtStatus::Finalized);

        if let Err(_) = result {
            return Some(ChainError::Error("Transfer failed on chain".to_string()));
        }
        // verify that Account's free Balance increased
        let result = api.get_account_data(&to);
        if let Err(_) = result {
            return Some(ChainError::Error(
                "fetching account data from chain failed".to_string(),
            ));
        }

        if let Some(account) = result.unwrap() {
            if account.free < former_balance {
                return Some(ChainError::Error("Transfer failed".to_string()));
            }
        }
        return Some(ChainError::None);
    } else {
        Some(ChainError::Error(
            "error occurred while connecting to node".to_string(),
        ))
    }
}

pub fn approve_transaction(params: ApproveTransactionParams) -> Option<ChainError> {
    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from: sr25519::Pair = utils::generate_pair(&params.seed.as_str());

    if let Ok(api) = init_api_client(from, &params.ws_url.as_str()) {
        let call_hash = params.call_hash.strip_prefix("0x").unwrap();
        let call_hash_data = hex::decode(call_hash).unwrap();
        let call_hash: [u8; 32] = call_hash_data[..].try_into().unwrap();

        let threshold = params.threshold;
        let other_signatories = params.other_signatories;
        let maybe_timepoint = Some(params.timepoint);
        let max_weight: u64 = 1000000000;
        // trace!("\n Composed Call: {:?}\n", multi_param);

        let other_signatories: Vec<AccountId> = other_signatories
            .iter()
            .map(|si| utils::parse_signatories(si.as_str()))
            .collect();

        // compose the extrinsic with all the element
        #[allow(clippy::redundant_clone)]
        let xt: subclient::UncheckedExtrinsicV4<_> = subclient::compose_extrinsic!(
            api,
            "MultiSig",
            "approve_as_multi",
            threshold,
            other_signatories,
            maybe_timepoint,
            call_hash,
            max_weight
        );

        // trace!("\n Composed Extrinsic: {:?}\n", xt);

        let xt_hash = xt.hex_encode(); //.strip_prefix("0x").unwrap().to_string();

        // trace!("\n Composed Extrinsic: {:?}\n", &xt_hash,);

        // send and watch extrinsic until Finalized
        let res = api.send_extrinsic(xt_hash.clone(), subclient::XtStatus::Finalized);

        match res {
            Ok(hash) => {
                if let Some(tx_hash) = hash {
                    trace!("Multisig Transaction got included. Hash: {:?}", tx_hash);
                    return Some(ChainError::None);
                }
                return Some(ChainError::Error("Transaction Approval Failed".to_string()));
            }
            Err(e) => {
                trace!("Multisig Transaction failed: Err: {:?}", e.to_string());

                return Some(ChainError::Error("Transaction Approval Failed".to_string()));
            }
        }
    } else {
        Some(ChainError::Error(
            "error occurred while connecting to node".to_string(),
        ))
    }
}

fn init_api_client(
    from: sr25519::Pair,
    ws_url: &str,
) -> Result<Api<sr25519::Pair, WsRpcClient>, Box<dyn Error>> {
    let client = subclient_rpc::WsRpcClient::new(ws_url);
    let api_result = subclient::Api::new(client).map(|api| api.set_signer(from.clone()));

    match api_result {
        Ok(a) => Ok(a),
        Err(e) => Err(From::from(e.to_string())),
    }
}

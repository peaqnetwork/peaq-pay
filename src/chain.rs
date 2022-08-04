use codec::Decode;
use keyring::sr25519::sr25519;
use sp_runtime::{AccountId32 as AccountId, MultiAddress};
use std::str::FromStr;
use subclient::Pair;
use substrate_api_client::{self as subclient, rpc as subclient_rpc};

use crate::utils;

pub enum ChainError {
    Error(String),
    None,
}

pub fn fund_multisig_wallet(
    ws_url: String,
    address: String,
    amount: subclient::Balance,
    seed: String,
) -> Option<ChainError> {
    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from: sr25519::Pair = utils::generate_pair(&seed.as_str());

    let client = subclient_rpc::WsRpcClient::new(&ws_url);
    let api = subclient::Api::new(client)
        .map(|api| api.set_signer(from.clone()))
        .unwrap();

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
}

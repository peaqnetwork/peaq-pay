use codec::{Decode, Encode};
use keyring::sr25519;
use log::trace;
use sp_core::blake2_256;
use sp_runtime::AccountId32 as AccountId;
use std::io::Error;
use std::str::FromStr;

pub fn create_multisig_wallet(signatories: Vec<String>, threshold: u16) -> Result<String, Error> {
    let mut signatories: Vec<AccountId> =
        signatories.iter().map(|si| parse_signatories(si)).collect();

    let _ = &signatories.sort();
    let prefix = b"modlpy/utilisuba";

    let entropy = (prefix, signatories, threshold).using_encoded(blake2_256);
    trace!("entropy:: {:?}", &entropy);

    let multi = AccountId::decode(&mut &entropy[..]).unwrap();

    // match multi_result {
    //     Ok((m)) =>
    //     Err(_) => todo!(),
    // }

    trace!("MULTI:: {}", &multi);

    Ok(multi.to_string())
}

pub fn parse_signatories(address: &str) -> AccountId {
    let to = sr25519::sr25519::Public::from_str(&address).unwrap();
    let to = AccountId::decode(&mut &to.0[..]).unwrap();
    to
}

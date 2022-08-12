# Peaq Pay

This package is developed to facilitate payments on the peaq network, although it can be use on any substrate based network.

Current MVP1 is built to only focus on the transactions between a consumer and a provider of service on the network.

The Peaq Pay package is currently being used on our [CharmEV mobile DApp](https://github.com/peaqnetwork/peaq-network-charmev). It currently facilatates:
* the creation of the multisig wallet where the consumer of the service deposit a certain amount required for a service. 
* Funding multisig wallet using sender existing wallet on the network. 
* Approving refund and spent transactions after a completed charging session.

### Requirements
* [Multisig Pallet](https://crates.io/crates/pallet-multisig) installed on the network.

### Installation

```
[dependencies]
peaq-pay = { git = "https://github.com/peaqnetwork/peaq-pay.git", branch = "dev"}

```

### Docs & Usage
On this current MVP, three API methods were exposed;

* create_multisig_wallet
* fund_multisig_wallet
* approve_transaction

#### create_multisig_wallet(signatories: Vec\<String\>, threshold: u16) -> Result\<String, Error\>

`signatories:` List of wallet addresses of signatories to the escrow account.

`threshold:` Number of signatures needed to release funds on the escrow account.

Returns multisig wallet address created using the wallet addresses of all signatories or error.

#### Example: 
```
fn main() {
    let threshold = 1;

    let signatories = vec![
        "5Cqhv4WScz1RF9ZUjnGRX...".to_string(),
        "5FA1TANVTDRkzE1TGK43J...".to_string(),
    ];

    let address = peaq_pay::utils::create_multisig_wallet(signatories, threshold).unwrap();
    println!("multisig wallet: {}", address);
}
```

#### fund_multisig_wallet(ws_url: String, address: String, amount: u128, seed: String ) -> Option\<ChainError\>

`ws_url:` Network websocket node address.

`address:` Multisig wallet to fund.

`amount:` Amount to fund the wallet. 

`seed:` Secret seed used for signing the transaction.

Enum: `ChainError`
```
pub enum ChainError {
    Error(String),
    None,
}
```

Returns an optional `ChainError` enums.
#### Example: 
```
fn main() {
    let amount: Balance = 20000000`;

    let address = "5Cqhv4WScz1RF9ZUjnGRXaZ...".to_string();

    let ev_res = peaq_pay::chain::fund_multisig_wallet(ws_url, address, amount, seed).unwrap();

    match ev_res {
        peaq_pay::chain::ChainError::Error(err) => {
            // return the error data if transfer error occurred
            println!("{}", err)
        }
        _ => {
            println!("approval success")
        }
    }
}
```


### approve_transaction(params: ApproveTransactionParams) -> Option\<ChainError\>

Param: ApproveTransactionParams 
```
pub struct ApproveTransactionParams {
    pub ws_url: String,
    pub threshold: u16,
    pub other_signatories: Vec<String>,
    pub timepoint: Timepoint<BlockNumber>,
    pub call_hash: String,
    pub max_weight: u64,
    pub seed: String,
}
```

`ws_url:` Network websocket node address.

`threshold:` Number of signatures needed to release funds on the escrow account.

`other_signatories:` List of wallet address of signatories (other than the sender) who can sign this transaction.

`timepoint:` A global extrinsic index, formed as the extrinsic index within a block, together with that block's height. This allows a transaction in which a multisig operation of a particular composite was created to be uniquely identified. 

`call_hash:` The call to be executed. 

`max_weight:` The weight of the call.

`seed:` Secret seed used for signing the transaction.


Timepoint
```
pub struct Timepoint<BlockNumber> {
    pub height: BlockNumber,
    pub index: u32,
}
```
#### Example: 
```
fn main() {

    let timepoint = peaq_pay::chain::Timepoint {
        height: timepoint_height,
        index: timepoint_index,
    };

    let params = peaq_pay::chain::ApproveTransactionParams {
        ws_url,
        threshold,
        timepoint,
        max_weight: 1000000000,
        other_signatories,
        call_hash,
        seed,
    };

    let ev_res = peaq_pay::chain::approve_transaction(params).unwrap();

    match ev_res {
        peaq_pay::chain::ChainError::Error(err) => {
            // return the error data if transfer error occurred
            println!("{}", err)
        }
        _ => {
            println!("approval success")
        }
    }
}
```

### Upcoming Features
* Support a stable means of payment with as many local currencies as possible (starting with USD, EURO as primary focus).
* Support traditional payment methods such as credit cards or bank accounts.
* Support consumers & providers to on- & offramp their peaq tokens.




## License

[Apache 2.0](https://choosealicense.com/licenses/apache-2.0/)


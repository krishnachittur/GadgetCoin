use super::gas::{
    Gas,
};

use super::type_aliases::{
    ETHAddress,
    TxnSignature
};

// EVM transaction
pub struct ETHTxn {
    pub nonce: u32,
    pub gasprice: Gas,
    pub gaslimit: Gas,
    pub recipient: ETHAddress,
    pub value: Gas,
    // init: Option<!>, // data only used for smart contracts
    pub signature: TxnSignature // ECDSA signature
}
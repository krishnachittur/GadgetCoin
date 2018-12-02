use super::wei::{
    Wei,
};

use super::aliases::{
    ETHAddress,
    TxnSignature,
};

// EVM transaction
pub struct ETHTxn {
    pub nonce: u32,
    pub gasprice: Wei,
    pub gaslimit: Wei,
    pub recipient: ETHAddress,
    pub value: Wei,
    pub code: Vec<u8>,
    pub signature: TxnSignature, // ECDSA signature
}
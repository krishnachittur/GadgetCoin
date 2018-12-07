use super::aliases::ETHAddress;
use super::wei::Wei;

#[derive(Debug)]
pub struct ETHAccount {
    pub address: ETHAddress,
    pub balance: Wei,
    pub nonce: u32,
}

impl ETHAccount {
    pub fn new(addr: ETHAddress) -> ETHAccount {
        ETHAccount {
            address: addr,
            balance: Wei::from_wei(0),
            nonce: 0,
        }
    }
}

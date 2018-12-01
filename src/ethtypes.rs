use std::collections::HashMap;

type ETHAddress = [u8; 20];
type BlockHash = ring::digest::Digest;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gas {
    wei: u128,
}

impl Gas {
    const SZABO_PER_WEI: u128 = 1_000_000_000_000;
    const FINNEY_PER_WEI: u128 = 1_000_000_000_000_000;
    const ETH_PER_WEI: u128 = 1_000_000_000_000_000_000;

    pub fn from_wei(init: u128) -> Self {
        Self{wei: init}
    }
    pub fn from_szabo(init: u128) -> Self {
        Self{wei: init * Self::SZABO_PER_WEI}
    }
    pub fn from_finney(init: u128) -> Self {
        Self{wei: init * Self::FINNEY_PER_WEI}
    }
    pub fn from_eth(init: u128) -> Self {
        Self{wei: init * Self::ETH_PER_WEI}
    }
    pub fn add(&self, other: Self) -> Self {
        Self{wei: self.wei + other.wei}
    }
    /// returns Option since negative Gas is invalid 
    pub fn sub(&self, other: Self) -> Option<Self> {
        self.wei
            .checked_sub(other.wei)
            .map(|c| Self{wei: c})
    }
}

pub struct ETHAccount {
    pub address : ETHAddress,
    pub balance : u32,
    pub nonce : u32,
    // contract_code : Option<!>, //only used for smart contracts
    // storage : !, //only used for smart contracts
}

// EVM transaction
pub struct ETHTxn {
    pub recipient: ETHAddress,
    pub eth: Gas,
    pub gasprice: Gas,
    pub gaslimit: Gas,
    // data: Option<!>, // optional data field only for smart contracts
}

pub struct ETHBlock {
    pub prev_hash: BlockHash,
    pub transactions: Vec<ETHTxn>,
    pub hash: Option<BlockHash>,
}

pub struct ETHBlockchain {
    pub blocks: Vec<ETHBlock>,
}

pub struct ETHState {
    pub accounts: HashMap<ETHAddress, ETHAccount>,
}

pub struct EVMState {
    // TODO:
}
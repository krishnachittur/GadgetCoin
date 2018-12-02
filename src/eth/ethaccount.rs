use super::aliases::{
    ETHAddress,
};

pub struct ETHAccount {
    pub address : ETHAddress,
    pub balance : u32,
    pub nonce : u32,
    // contract_code : Option<!>, // only used for smart contracts
    // storage : !, // only used for smart contracts
}

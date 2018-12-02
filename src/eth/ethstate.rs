use std::collections::HashMap;

use super::ethaccount::{
    ETHAccount,
};

use super::aliases::{
    ETHAddress
};

pub struct ETHState {
    pub accounts: HashMap<ETHAddress, ETHAccount>,
}
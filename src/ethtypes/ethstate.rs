use std::collections::HashMap;

use super::ethaccount::{
    ETHAccount,
};

use super::type_aliases::{
    ETHAddress
};

pub struct ETHState {
    pub accounts: HashMap<ETHAddress, ETHAccount>,
}
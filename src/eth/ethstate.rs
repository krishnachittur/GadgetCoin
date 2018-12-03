use std::collections::HashMap;

use super::wei::Wei;
use super::ethtxn::ETHTxn;
use super::aliases::ETHAddress;
use super::ethaccount::ETHAccount;

pub struct ETHState {
    accounts: HashMap<ETHAddress, ETHAccount>,
}

impl ETHState {
    pub fn new() -> ETHState {
        ETHState {
            accounts: HashMap::new()
        }
    }

    pub fn increment_nonce(&mut self, addr: &ETHAddress) {
        self.accounts.get_mut(addr).unwrap().nonce += 1;
    }

    pub fn invalid_nonce(&self, txn: &ETHTxn, addr: &ETHAddress) -> bool {
        self.accounts.contains_key(addr) &&
        (self.accounts[addr].nonce + 1 != txn.nonce)
    }

    pub fn safe_deduct(&mut self, addr: &ETHAddress, amount: Wei) -> bool {
        if let Some(account) = self.accounts.get_mut(addr) {
            account.balance =  match account.balance - amount {
                Some(val) => val,
                None => return false
            };
            true
        } else {
            false
        }
    }

    pub fn pay(&mut self, addr: &ETHAddress, amount: Wei) {
        self.accounts
            .entry(*addr)
            .or_insert(
                ETHAccount::new(*addr)
            ).balance += amount;
    }
}
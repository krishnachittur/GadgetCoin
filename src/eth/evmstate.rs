use super::aliases::StackItem;
use super::gas::Gas;
use super::ethtxn::ETHTxn;
use super::ethstate::ETHState;
use super::ops::{Op};

pub struct EVMState {
    pub stack: [StackItem; 1024],
    pub pc: u32,
    pub gas: Gas,
    pub code: Vec<Op>,
    pub transaction: ETHTxn,
    pub block_state: ETHState,
}

#[cfg(test)]
mod tests {
    use super::EVMState;
    #[test]
    fn test_evmstate() {
        // TODO: write a basic script here and check output
    }
}
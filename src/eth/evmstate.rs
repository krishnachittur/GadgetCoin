use super::aliases::StackItem;
use super::gas::Gas;
use super::ethtxn::ETHTxn;
use super::ethstate::ETHState;
use super::ops::Op;

pub struct EVMState {
    pub stack: [StackItem; 1024],
    pub pc: u32,
    pub gas: Gas,
    pub transaction: ETHTxn,
    pub block_state: ETHState,
    pub code: Vec<Op>
}

impl EVMState {
    pub fn run() -> ! {
        loop {
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::{
    //     EVMState,
    //     super::{
    //         ops::*,
    //         wei::*,
    //         aliases::*,
    //         ethtxn::*,
    //     },
    // };

    // fn get_sample_signature() -> TxnSignature {

    // }

    // fn get_sample_address() -> ETHAddress {
    //     [0x00, 0x01, 0x02, 0x03, 0x04,
    //      0x10, 0x11, 0x12, 0x13, 0x14,
    //      0x20, 0x21, 0x22, 0x23, 0x24,
    //      0x30, 0x31, 0x32, 0x33, 0x34,]
    // }

    // fn get_sample_transaction() -> ETHTxn {
    //     ETHTxn{
    //         nonce: 42,
    //         gasprice: 2,
    //         gaslimit: 100,
    //         recipient: get_sample_address(),
    //         value: Wei::from_wei(10),
    //         code: Vec::new(),
    //         signature: get_sample_signature(),
    //     }
    // }

    #[test]
    fn test_evmstate() {
        // let code = vec![, Op::STOP];
    }
}
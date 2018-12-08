use super::aliases::ETHAddress;
use super::ethblock::ETHBlock;
use super::ethstate::ETHState;
use super::ethtxn::ETHTxn;
use super::evmstate::{EVMState, FailureReason};

pub struct ETHBlockchain {
    block_txn_limit: usize,
    miner: ETHAddress,
    difficulty: u32,
    outstanding_txns: Vec<ETHTxn>,
    evmstate: EVMState,
    blocks: Vec<ETHBlock>,
}

impl ETHBlockchain {
    pub fn new(block_txn_limit: usize, difficulty: u32, miner: ETHAddress) -> ETHBlockchain {
        ETHBlockchain {
            block_txn_limit,
            miner,
            difficulty,
            outstanding_txns: vec![],
            evmstate: EVMState::new(ETHState::default(), miner),
            blocks: vec![ETHBlock::genesis()],
        }
    }

    // returns a block with an uncomputed nonce
    pub fn flush_txns(&mut self) -> ETHBlock {
        let txns = std::mem::replace(&mut self.outstanding_txns, vec![]);
        ETHBlock::new(
            // safe to unwrap due to genesis block
            self.blocks.last().unwrap(),
            self.miner,
            self.difficulty,
            txns,
        )
    }

    // if the transactions hit the limit, create a new unvalidated block
    pub fn process_transaction(&mut self, txn: ETHTxn) -> Option<ETHBlock> {
        if let Err(txn_failure) = self.evmstate.run_transaction(&txn) {
            match txn_failure {
                FailureReason::InvalidSignature | FailureReason::InvalidNonce => {
                    return None;
                }
                _ => {}
            }
        }
        self.outstanding_txns.push(txn);
        if self.outstanding_txns.len() >= self.block_txn_limit {
            Some(self.flush_txns())
        } else {
            None
        }
    }

    pub fn add_block(&mut self, block: ETHBlock) -> bool {
        if !block.is_valid() {
            return false;
        }
        self.evmstate.reward_miner(block.get_miner_address());
        self.blocks.push(block);
        true
    }
}

#[cfg(test)]
pub mod tests {}

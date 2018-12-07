use super::aliases::{BlockHash, ProofOfWork};
use super::ethtxn::ETHTxn;

pub struct ETHBlock {
    pub prev_hash: BlockHash,
    pub transactions: Vec<ETHTxn>,
    pub difficulty: u32,
    pub nonce: ProofOfWork,
}

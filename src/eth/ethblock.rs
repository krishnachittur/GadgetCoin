use bincode::serialize;
use rand::Rng;
use sha3::{Digest, Sha3_256};

use super::aliases::{BlockHash, ETHAddress, ProofOfWork};
use super::ethtxn::ETHTxn;

#[derive(Debug, Serialize, Clone)]
pub struct ETHBlock {
    prev_hash: BlockHash,
    miner_id: ETHAddress,
    transactions: Vec<ETHTxn>,
    difficulty: u32,
    nonce: ProofOfWork,
}

impl ETHBlock {
    pub fn genesis() -> ETHBlock {
        ETHBlock {
            prev_hash: [0; 32],
            miner_id: [0; 20],
            transactions: vec![],
            difficulty: 0,
            nonce: 0,
        }
    }

    pub fn new(
        prev_block: &ETHBlock,
        miner_id: ETHAddress,
        difficulty: u32,
        transactions: Vec<ETHTxn>,
    ) -> ETHBlock {
        ETHBlock {
            prev_hash: prev_block.hash(),
            miner_id,
            transactions,
            difficulty,
            nonce: 0,
        }
    }

    pub fn binary_serialization(&self) -> Vec<u8> {
        serialize(self).unwrap()
    }

    pub fn hash(&self) -> BlockHash {
        let mut hasher = Sha3_256::default();
        hasher.input(self.binary_serialization());
        let mut result: BlockHash = [0; 32];
        let hasher_result = hasher.result();
        let slice = hasher_result.as_slice();
        result.copy_from_slice(slice);
        result
    }

    pub fn randomize_nonce(&mut self, rng: &mut rand::ThreadRng) {
        self.nonce = rng.next_u32();
    }

    pub fn set_nonce(&mut self, nonce: ProofOfWork) {
        self.nonce = nonce
    }

    pub fn get_nonce(&self) -> ProofOfWork {
        self.nonce
    }

    pub fn is_valid(&self) -> bool {
        let zero_bytes = self.difficulty as usize / 8;
        let zero_bits = self.difficulty % 8;
        let hash = self.hash();

        for byte in hash[..zero_bytes].iter() {
            if *byte != 0 {
                return false;
            }
        }
        hash[zero_bytes] <= (255 >> zero_bits)
    }

    pub fn get_miner_address(&self) -> ETHAddress {
        self.miner_id
    }
}

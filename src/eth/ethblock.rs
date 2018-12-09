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

    fn valid_hash(hash: BlockHash, difficulty: u32) -> bool {
        let zero_bytes = difficulty as usize / 8;
        let zero_bits = difficulty % 8;

        for byte in hash[..zero_bytes].iter() {
            if *byte != 0 {
                return false;
            }
        }
        hash[zero_bytes] <= (255 >> zero_bits)
    }

    pub fn is_valid(&self) -> bool {
        Self::valid_hash(self.hash(), self.difficulty)
    }

    pub fn get_miner_address(&self) -> ETHAddress {
        self.miner_id
    }
}

#[cfg(test)]
mod tests {
    use super::ETHBlock;

    #[test]
    fn test_hashing() {
        let hash1 = [0; 32];
        assert!(ETHBlock::valid_hash(hash1, 255));

        let hash2 = [1; 32];
        assert!(ETHBlock::valid_hash(hash2, 7));
        assert!(!ETHBlock::valid_hash(hash2, 8));

        let mut hash3 = [2; 32];
        hash3[0] = 0;
        hash3[1] = 0;
        assert!(ETHBlock::valid_hash(hash3, 10));
        assert!(ETHBlock::valid_hash(hash3, 16));
        assert!(ETHBlock::valid_hash(hash3, 16 + 6));
        assert!(!ETHBlock::valid_hash(hash3, 16 + 7));
    }
}

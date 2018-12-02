pub type ETHAddress = [u8; 20];
//type BlockHash = ring::digest::Digest;
pub type BlockHash = [u8; 32];
pub type ProofOfWork = [u8; 128];
pub type StackItem = [u8; 32];
pub type TxnSignature = ring::signature::Signature;
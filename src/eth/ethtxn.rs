use bincode::serialize;
use sha3::Digest;

use super::aliases::ETHAddress;
use super::gas::Gas;
use super::wei::Wei;

#[derive(Debug, Serialize, Clone)]
pub struct ETHTxn {
    pub nonce: u32,
    pub gasprice: Wei,
    pub gaslimit: Gas,
    pub recipient: ETHAddress,
    pub value: Wei,
    pub code: Vec<u8>,

    #[serde(skip_serializing)]
    pub ecdsa_fields: (secp256k1::Signature, secp256k1::RecoveryId),
}

impl ETHTxn {
    /// Returns a byte-wise serialization of the transaction struct
    pub fn binary_serialization(&self) -> Vec<u8> {
        serialize(self).unwrap()
    }

    /// Converts the ETHTxn instance to raw bytes and then converts it to a
    /// secp256k1::Message.
    pub fn hashed_message(encoded: &[u8]) -> Result<secp256k1::Message, secp256k1::Error> {
        let hash = sha3::Keccak256::digest(encoded);
        secp256k1::Message::parse_slice(&hash)
    }

    /// Recovers the public key from the ETHTxn.
    pub fn recover_public_key(&self) -> Result<secp256k1::PublicKey, secp256k1::Error> {
        let msg = self.binary_serialization();
        let hashed_message = Self::hashed_message(&msg)?;
        let &(ref signature, ref recovery_id) = &self.ecdsa_fields;
        secp256k1::recover(&hashed_message, signature, recovery_id)
    }

    /// Given a public key, computes the address.
    pub fn get_address_from_public_key(
        pubkey: &secp256k1::PublicKey,
    ) -> Result<ETHAddress, secp256k1::Error> {
        let pub_key_bytes = pubkey.serialize();
        let pub_hash = sha3::Keccak256::digest(&pub_key_bytes[1..]);

        let mut sender_addr: [u8; 20] = [0; 20];
        sender_addr.copy_from_slice(&pub_hash[12..]);
        Ok(sender_addr)
    }

    /// Returns the sender's address.
    pub fn get_sender_addr(&self) -> Result<ETHAddress, secp256k1::Error> {
        let pub_key = self.recover_public_key()?;
        Self::get_address_from_public_key(&pub_key)
    }

    pub fn sign_transaction(&mut self, sender_secret: &secp256k1::SecretKey) {
        let msg = self.binary_serialization();
        let msg = match ETHTxn::hashed_message(&msg) {
            Ok(val) => val,
            _ => panic!("Couldn't retrieve message"),
        };
        self.ecdsa_fields = match secp256k1::sign(&msg, sender_secret) {
            Ok(val) => val,
            _ => panic!("Signature couldn't be generated"),
        };
    }
}

pub mod utils {
    use super::ETHTxn;
    /// Returns sample ECSDA fields
    pub fn get_bs_ecsda_field(
        bs_secret_key: &secp256k1::SecretKey,
    ) -> (secp256k1::Signature, secp256k1::RecoveryId) {
        let bs_msg_bytes = b"deadbeef";
        let bs_msg = match ETHTxn::hashed_message(bs_msg_bytes) {
            Ok(val) => val,
            _ => panic!("Couldn't generate BS hashed message"),
        };
        match secp256k1::sign(&bs_msg, &bs_secret_key) {
            Ok(val) => val,
            _ => panic!("Signature couldn't be generated"),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::{super::wei::Wei, utils::get_bs_ecsda_field, ETHTxn};

    #[test]
    fn test_basic_crypto_should_pass() {
        let mut rng = rand::thread_rng();

        let sender_secretkey = secp256k1::SecretKey::random(&mut rng);
        let sender_pubkey = secp256k1::PublicKey::from_secret_key(&sender_secretkey);

        let receiver_secretkey = secp256k1::SecretKey::random(&mut rng);
        let receiver_pubkey = secp256k1::PublicKey::from_secret_key(&receiver_secretkey);

        let mut sample_txn = ETHTxn {
            nonce: 13,
            gasprice: Wei::from_wei(20),
            gaslimit: 400,
            recipient: match ETHTxn::get_address_from_public_key(&receiver_pubkey) {
                Ok(val) => val,
                _ => panic!("Address couldn't be generated"),
            },
            value: Wei::from_wei(10),
            code: vec![0x31, 0x3a, 0x56, 0x57, 0x50, 0x05],
            ecdsa_fields: get_bs_ecsda_field(&secp256k1::SecretKey::random(&mut rng)),
        };

        let msg = {
            let msg = sample_txn.binary_serialization();
            match ETHTxn::hashed_message(&msg) {
                Ok(val) => val,
                _ => panic!("Couldn't retrieve message"),
            }
        };
        sample_txn.ecdsa_fields = match secp256k1::sign(&msg, &sender_secretkey) {
            Ok(val) => val,
            _ => panic!("Signature couldn't be generated"),
        };

        assert_eq!(
            sample_txn.get_sender_addr().unwrap(),
            ETHTxn::get_address_from_public_key(&sender_pubkey).unwrap()
        );
    }

    #[test]
    fn test_basic_crypto_should_fail() {
        let mut rng = rand::thread_rng();

        let sender_secretkey = secp256k1::SecretKey::random(&mut rng);

        let receiver_secretkey = secp256k1::SecretKey::random(&mut rng);
        let receiver_pubkey = secp256k1::PublicKey::from_secret_key(&receiver_secretkey);

        let mut sample_txn = ETHTxn {
            nonce: 13,
            gasprice: Wei::from_wei(20),
            gaslimit: 400,
            recipient: match ETHTxn::get_address_from_public_key(&receiver_pubkey) {
                Ok(val) => val,
                _ => panic!("Address couldn't be generated"),
            },
            value: Wei::from_wei(10),
            code: vec![0x31, 0x3a, 0x56, 0x57, 0x50, 0x05],
            ecdsa_fields: get_bs_ecsda_field(&secp256k1::SecretKey::random(&mut rng)),
        };

        let msg = {
            let msg = sample_txn.binary_serialization();
            match ETHTxn::hashed_message(&msg) {
                Ok(val) => val,
                _ => panic!("Couldn't retrieve message"),
            }
        };
        sample_txn.ecdsa_fields = match secp256k1::sign(&msg, &sender_secretkey) {
            Ok(val) => val,
            _ => panic!("Signature couldn't be generated"),
        };

        let random_secretkey = secp256k1::SecretKey::random(&mut rng);
        let random_pubkey = secp256k1::PublicKey::from_secret_key(&random_secretkey);

        assert_ne!(
            sample_txn.get_sender_addr().unwrap(),
            ETHTxn::get_address_from_public_key(&random_pubkey).unwrap()
        );
    }
}

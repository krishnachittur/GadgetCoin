use super::aliases::ETHAddress;
use super::ethstate::ETHState;
use super::ethtxn::ETHTxn;
use super::evmexec::ExecutionContext;
use super::ops::Op;
use super::wei::Wei;

pub struct EVMState {
    world_state: ETHState,
    miner_address: ETHAddress,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FailureReason {
    InvalidSignature,
    InvalidNonce,
    InsufficientBalance,
    InvalidCode,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EndState {
    SUCCESS,
    FAILURE(FailureReason),
}

impl EVMState {
    pub fn new(state: ETHState, miner: ETHAddress) -> EVMState {
        EVMState {
            world_state: state,
            miner_address: miner,
        }
    }

    pub fn get_final_state(self) -> ETHState {
        self.world_state
    }

    // ingest and process a well-formed transaction
    pub fn run_transaction(&mut self, txn: &ETHTxn) -> EndState {
        // make sure sender's address exists and transaction is correctly signed
        let sender_addr = match txn.get_sender_addr() {
            Ok(addr) => {
                if self.world_state.key_exists(&addr) {
                    addr
                } else {
                    return EndState::FAILURE(FailureReason::InvalidSignature);
                }
            }
            Err(_) => return EndState::FAILURE(FailureReason::InvalidSignature),
        };
        if self.world_state.invalid_nonce(&txn, &sender_addr) {
            return EndState::FAILURE(FailureReason::InvalidNonce);
        }

        self.world_state.increment_nonce(&sender_addr);

        // calculate transaction fee and subtract from sender's account balance
        let max_fee = Wei::from_gas(txn.gasprice, txn.gaslimit);
        if !self.world_state.safe_deduct(&sender_addr, max_fee) {
            // not enough money
            return EndState::FAILURE(FailureReason::InsufficientBalance);
        }

        let mut exec_context =
            ExecutionContext::new(txn.gaslimit, Op::from_bytes(&txn.code), txn.value);

        // execute code, making sure to track new transaction value
        // terminate on invalid code or STOP instruction
        let valid_termination = exec_context.finish_executing();

        // refund remaining gas to sender
        let sender_refund = Wei::from_gas(txn.gasprice, exec_context.get_gas_left());
        self.world_state.pay(&sender_addr, sender_refund);

        // pay miner for their work
        let miner_fee = match max_fee - Wei::from_gas(txn.gasprice, exec_context.get_gas_left()) {
            None => panic!("gas left somehow exceeds initial gas"),
            Some(v) => v,
        };
        self.world_state.pay(&self.miner_address, miner_fee);

        // terminate early if code was invalid
        if !valid_termination {
            return EndState::FAILURE(FailureReason::InvalidCode);
        }

        // complete transaction if the value doesn't exceed the money in the sender's account
        if !self
            .world_state
            .safe_deduct(&sender_addr, exec_context.get_value())
        {
            return EndState::FAILURE(FailureReason::InsufficientBalance);
        }
        self.world_state
            .pay(&txn.recipient, exec_context.get_value());
        EndState::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::aliases::ETHAddress, super::ethstate::ETHState,
        super::ethtxn::tests::get_bs_ecsda_field, super::ethtxn::ETHTxn, super::wei::Wei, EVMState,
        EndState, FailureReason,
    };

    struct Ctx {
        miner: ETHAddress,
        sender: ETHAddress,
        sender_secret: secp256k1::SecretKey,
        receiver: ETHAddress,
        txn: ETHTxn,
        evm_state: EVMState,
    }

    impl Ctx {
        fn sign_transaction(&mut self) {
            let msg = self.txn.binary_serialization();
            let msg = match ETHTxn::hashed_message(&msg) {
                Ok(val) => val,
                _ => panic!("Couldn't retrieve message"),
            };
            self.txn.ecdsa_fields = match secp256k1::sign(&msg, &self.sender_secret) {
                Ok(val) => val,
                _ => panic!("Signature couldn't be generated"),
            };
        }
    }

    fn get_basic_test_ctx() -> Ctx {
        let mut rng = rand::thread_rng();

        let miner_secretkey = secp256k1::SecretKey::random(&mut rng);
        let miner_pubkey = secp256k1::PublicKey::from_secret_key(&miner_secretkey);
        let miner_addr = ETHTxn::get_address_from_public_key(&miner_pubkey).unwrap();

        let sender_secretkey = secp256k1::SecretKey::random(&mut rng);
        let sender_pubkey = secp256k1::PublicKey::from_secret_key(&sender_secretkey);
        let sender_addr = ETHTxn::get_address_from_public_key(&sender_pubkey).unwrap();

        let receiver_secretkey = secp256k1::SecretKey::random(&mut rng);
        let receiver_pubkey = secp256k1::PublicKey::from_secret_key(&receiver_secretkey);
        let receiver_addr = ETHTxn::get_address_from_public_key(&receiver_pubkey).unwrap();

        let sample_txn = ETHTxn {
            nonce: 1,
            gasprice: Wei::from_wei(20),
            gaslimit: 0,
            recipient: receiver_addr.clone(),
            value: Wei::from_wei(10),
            code: vec![],
            ecdsa_fields: get_bs_ecsda_field(secp256k1::SecretKey::random(&mut rng)),
        };

        let mut ctx = Ctx {
            miner: miner_addr,
            sender: sender_addr,
            sender_secret: sender_secretkey,
            receiver: receiver_addr,
            txn: sample_txn,
            evm_state: EVMState::new(ETHState::default(), miner_addr),
        };

        ctx.sign_transaction();
        ctx
    }

    #[test]
    fn general() {
        let mut test_ctx = get_basic_test_ctx();

        let mut txn_val_in_wei = 10;
        test_ctx.txn.value = Wei::from_wei(txn_val_in_wei);

        // initialize accounts
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.sender, Wei::from_wei(30));

        let mut expected_sender_balance = 30 - txn_val_in_wei;
        let mut expected_miner_balance = 0;

        // set gasprice and limit
        let gasprice_in_wei = 2;
        let gaslimit = 5;
        test_ctx.txn.gasprice = Wei::from_wei(gasprice_in_wei);
        test_ctx.txn.gaslimit = gaslimit;

        // increment the nonce just cause
        test_ctx.txn.nonce += 1;
        test_ctx
            .evm_state
            .world_state
            .increment_nonce(&test_ctx.sender);

        // code: [Op::PUSH1(2), Op::ADDVAL, Op::STOP];
        test_ctx.txn.code = vec![0x60, 2, 0xb1, 0x00];
        txn_val_in_wei += 2;
        expected_sender_balance -= 2;
        let code_cost = (3 + 2) * gasprice_in_wei;
        expected_miner_balance += code_cost;
        expected_sender_balance -= code_cost;

        test_ctx.sign_transaction();

        assert_eq!(
            test_ctx.evm_state.run_transaction(&test_ctx.txn),
            EndState::SUCCESS
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.sender)
                .unwrap(),
            Wei::from_wei(expected_sender_balance)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.receiver)
                .unwrap(),
            Wei::from_wei(txn_val_in_wei)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.miner)
                .unwrap(),
            Wei::from_wei(expected_miner_balance)
        );
    }

    #[test]
    fn invalid_nonce_fails() {
        let mut test_ctx = get_basic_test_ctx();

        let sender_bal = 10;
        let receiver_bal = 20;
        let miner_bal = 30;
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.sender, Wei::from_wei(sender_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.receiver, Wei::from_wei(receiver_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.miner, Wei::from_wei(miner_bal));

        // increment the nonce twice to cause an error
        test_ctx.txn.nonce += 2;
        test_ctx
            .evm_state
            .world_state
            .increment_nonce(&test_ctx.sender);

        test_ctx.sign_transaction();

        assert_eq!(
            test_ctx.evm_state.run_transaction(&test_ctx.txn),
            EndState::FAILURE(FailureReason::InvalidNonce)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.sender)
                .unwrap(),
            Wei::from_wei(sender_bal)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.receiver)
                .unwrap(),
            Wei::from_wei(receiver_bal)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.miner)
                .unwrap(),
            Wei::from_wei(miner_bal)
        );
    }

    #[test]
    fn invalid_sender_signature_fails() {
        let mut test_ctx = get_basic_test_ctx();

        let sender_bal = 10;
        let receiver_bal = 20;
        let miner_bal = 30;
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.sender, Wei::from_wei(sender_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.receiver, Wei::from_wei(receiver_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.miner, Wei::from_wei(miner_bal));

        test_ctx.txn.gaslimit = 100; // previously generated signature will now be invalid

        assert_eq!(
            test_ctx.evm_state.run_transaction(&test_ctx.txn),
            EndState::FAILURE(FailureReason::InvalidSignature)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.sender)
                .unwrap(),
            Wei::from_wei(sender_bal)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.receiver)
                .unwrap(),
            Wei::from_wei(receiver_bal)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.miner)
                .unwrap(),
            Wei::from_wei(miner_bal)
        );
    }

    #[test]
    fn gas_money_checks() {
        let mut test_ctx = get_basic_test_ctx();

        let sender_bal = 10;
        let receiver_bal = 20;
        let miner_bal = 30;
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.sender, Wei::from_wei(sender_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.receiver, Wei::from_wei(receiver_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.miner, Wei::from_wei(miner_bal));

        test_ctx.txn.gaslimit = 10000; // really big number
        test_ctx.sign_transaction();

        assert_eq!(
            test_ctx.evm_state.run_transaction(&test_ctx.txn),
            EndState::FAILURE(FailureReason::InsufficientBalance)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.sender)
                .unwrap(),
            Wei::from_wei(sender_bal)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.receiver)
                .unwrap(),
            Wei::from_wei(receiver_bal)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.miner)
                .unwrap(),
            Wei::from_wei(miner_bal)
        );
    }

    #[test]
    fn txn_value_money_fails() {
        let mut test_ctx = get_basic_test_ctx();

        let sender_bal = 10;
        let receiver_bal = 20;
        let miner_bal = 30;
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.sender, Wei::from_wei(sender_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.receiver, Wei::from_wei(receiver_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.miner, Wei::from_wei(miner_bal));

        test_ctx.txn.value = Wei::from_wei(10000); // really big number
        test_ctx.sign_transaction();

        assert_eq!(
            test_ctx.evm_state.run_transaction(&test_ctx.txn),
            EndState::FAILURE(FailureReason::InsufficientBalance)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.sender)
                .unwrap(),
            Wei::from_wei(sender_bal)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.receiver)
                .unwrap(),
            Wei::from_wei(receiver_bal)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.miner)
                .unwrap(),
            Wei::from_wei(miner_bal)
        );
    }

    #[test]
    fn code_never_stops() {
        let mut test_ctx = get_basic_test_ctx();

        let sender_bal = 100;
        let receiver_bal = 20;
        let miner_bal = 30;
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.sender, Wei::from_wei(sender_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.receiver, Wei::from_wei(receiver_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.miner, Wei::from_wei(miner_bal));

        test_ctx.txn.gaslimit = 50;
        let gas_price_in_wei = 2;
        test_ctx.txn.gasprice = Wei::from_wei(gas_price_in_wei);
        // [Op::PUSH1(100), Op::PUSH1(0), Op::JUMPI, Op::STOP];
        test_ctx.txn.code = vec![0x60, 100, 0x60, 0, 0x57, 0x00];
        test_ctx.sign_transaction();

        let gas_cost = 48; // b/c gas_left = 50%(COST(JUMP)*2 + COST(JUMPI)) = 50%(3*2+10) = 2
        let expected_sender_balance = sender_bal - gas_cost * gas_price_in_wei;
        let expected_receiver_balance = receiver_bal;
        let expected_miner_balance = miner_bal + gas_cost * gas_price_in_wei;

        assert_eq!(
            test_ctx.evm_state.run_transaction(&test_ctx.txn),
            EndState::FAILURE(FailureReason::InvalidCode)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.sender)
                .unwrap(),
            Wei::from_wei(expected_sender_balance)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.receiver)
                .unwrap(),
            Wei::from_wei(expected_receiver_balance)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.miner)
                .unwrap(),
            Wei::from_wei(expected_miner_balance)
        );
    }

    #[test]
    fn invalid_code() {
        let mut test_ctx = get_basic_test_ctx();

        let sender_bal = 100;
        let receiver_bal = 20;
        let miner_bal = 30;
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.sender, Wei::from_wei(sender_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.receiver, Wei::from_wei(receiver_bal));
        test_ctx
            .evm_state
            .world_state
            .pay(&test_ctx.miner, Wei::from_wei(miner_bal));

        // [PUSH1(100), PUSH1(0), INVALID(0x05)]
        test_ctx.txn.code = vec![0x60, 100, 0x60, 0, 0x05];
        test_ctx.txn.gaslimit = 20;
        let gas_price_in_wei = 5;
        test_ctx.txn.gasprice = Wei::from_wei(gas_price_in_wei);
        test_ctx.txn.value = Wei::from_wei(10);
        test_ctx.sign_transaction();

        let gas_cost = 6;
        let expected_sender_balance = sender_bal - gas_cost * gas_price_in_wei;
        let expected_receiver_balance = receiver_bal;
        let expected_miner_balance = miner_bal + gas_cost * gas_price_in_wei;

        assert_eq!(
            test_ctx.evm_state.run_transaction(&test_ctx.txn),
            EndState::FAILURE(FailureReason::InvalidCode)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.sender)
                .unwrap(),
            Wei::from_wei(expected_sender_balance)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.receiver)
                .unwrap(),
            Wei::from_wei(expected_receiver_balance)
        );
        assert_eq!(
            test_ctx
                .evm_state
                .world_state
                .get_value(&test_ctx.miner)
                .unwrap(),
            Wei::from_wei(expected_miner_balance)
        );
    }
}

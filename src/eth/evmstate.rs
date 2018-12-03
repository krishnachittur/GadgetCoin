use super::wei::Wei;
use super::ethtxn::ETHTxn;
use super::ethstate::ETHState;
use super::ops::Op;
use super::aliases::ETHAddress;
use super::evmexec::ExecutionContext;

pub struct EVMState {
    pub world_state: ETHState,
    pub miner_address: ETHAddress,
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
    pub fn run_transaction(&mut self, txn: &ETHTxn) -> bool {

        // make sure sender's address exists and transaction is correctly signed
        let sender_addr = match txn.get_sender_addr() {
            Ok(addr) => addr,
            Err(_) => return false,
        };
        if self.world_state.invalid_nonce(&txn, &sender_addr) {
            return false;
        }

        self.world_state.increment_nonce(&sender_addr);

        // calculate transaction fee and subtract from sender's account balance
        let max_fee = Wei::from_gas(txn.gasprice, txn.gaslimit);
        if !self.world_state.safe_deduct(&sender_addr, max_fee) {
            // not enough money
            return false;
        }

        let mut exec_context = ExecutionContext::new(
            txn.gaslimit, Op::from_bytes(&txn.code), txn.value,
        );

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
            return false;
        }

        // complete transaction if the value doesn't exceed the money in the sender's account
        if !self.world_state.safe_deduct(&sender_addr, exec_context.get_value()) {
            return false;
        }
        self.world_state.pay(&txn.recipient, exec_context.get_value());
        true
    }
}

#[cfg(test)]
mod tests {
    // TODO
    // test end-to-end transaction processing
    // assume that engine runs correctly - focus on other things:
        // invalid nonce
        // invalid signature
        // sender doesn't have enough money for gas
        // sender doesn't have enough money for txn value
    // correct calculation of all Wei collected/paid
}
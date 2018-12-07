use super::gas::Gas;
use super::ops::Op;
use super::wei::Wei;

pub struct ExecutionContext {
    stack: Vec<u8>,
    pc: u32,
    gas_left: Gas,
    code: Vec<Op>,
    txn_value: Wei,
}

enum OperationResult {
    Continue,
    Stop,
}

impl ExecutionContext {
    pub fn new(gaslimit: Gas, code: Vec<Op>, txn_value: Wei) -> ExecutionContext {
        ExecutionContext {
            stack: Vec::new(),
            pc: 0,
            gas_left: gaslimit,
            code,
            txn_value,
        }
    }

    pub fn get_gas_left(&self) -> Gas {
        self.gas_left
    }

    pub fn get_value(&self) -> Wei {
        self.txn_value
    }

    // return true if terminated normally, false on error
    pub fn finish_executing(&mut self) -> bool {
        while let Ok(result) = self.execute_cycle() {
            match result {
                OperationResult::Continue => continue,
                OperationResult::Stop => return true,
            }
        }
        false // got an error
    }

    fn pop(&mut self) -> Result<u8, ()> {
        self.stack.pop().ok_or(())
    }

    fn push(&mut self, b: u8) {
        self.stack.push(b);
    }

    fn execute_cycle(&mut self) -> Result<OperationResult, ()> {
        if self.code.is_empty() {
            return Ok(OperationResult::Stop);
        }
        if self.pc as usize >= self.code.len() {
            // pc out of bounds
            return Err(());
        }
        // default pc increment
        let mut new_pc = self.pc + 1;
        let op = self.code[self.pc as usize];
        match op {
            Op::STOP => return Ok(OperationResult::Stop),
            Op::ADD => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(a + b);
            }
            Op::MUL => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(a * b);
            }
            Op::SUB => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(match a.checked_sub(b) {
                    Some(x) => x,
                    None => 0,
                });
            }
            Op::DIV => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(match a.checked_div(b) {
                    Some(x) => x,
                    None => 0,
                });
            }
            Op::LT => {
                let a = self.pop()?;
                let b = self.pop()?;
                if a < b {
                    self.push(1);
                } else {
                    self.push(0);
                }
            }
            Op::GT => {
                let a = self.pop()?;
                let b = self.pop()?;
                if a > b {
                    self.push(1);
                } else {
                    self.push(0);
                }
            }
            Op::EQ => {
                let a = self.pop()?;
                let b = self.pop()?;
                if a == b {
                    self.push(1);
                } else {
                    self.push(0);
                }
            }
            Op::ISZERO => {
                let a = self.pop()?;
                if a == 0 {
                    self.push(1);
                } else {
                    self.push(0);
                }
            }
            Op::POP => {
                self.pop()?;
            }
            Op::JUMP => {
                new_pc = u32::from(self.pop()?);
            }
            Op::JUMPI => {
                let a = self.pop()?;
                let b = self.pop()?;
                if b != 0 {
                    new_pc = u32::from(a);
                }
            }
            Op::PUSH1(val) => {
                self.push(val);
            }
            Op::SETVAL => {
                let a = self.pop()?;
                self.txn_value = Wei::from_wei(a.into());
            }
            Op::ADDVAL => {
                let a = self.pop()?;
                self.txn_value += Wei::from_wei(a.into());
            }
            Op::SUBVAL => {
                let a = self.pop()?;
                let wei = Wei::from_wei(a.into());
                self.txn_value = match self.txn_value - wei {
                    Some(x) => x,
                    None => Wei::from_wei(0),
                };
            }
            Op::INVALID(_) => return Err(()),
        };
        self.pc = new_pc;
        self.gas_left = match self.gas_left.checked_sub(op.to_cost()) {
            Some(val) => val,
            None => return Err(()),
        };
        Ok(OperationResult::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::ops::Op::*, super::wei::Wei, ExecutionContext};

    #[test]
    fn basic_evmexec_execution() {
        let mut engine = ExecutionContext::new(
            20,                 // gas limit
            vec![STOP],         //ops
            Wei::from_wei(100), // transaction value
        );
        assert!(engine.finish_executing());
        assert_eq!(engine.get_gas_left(), 20 - STOP.to_cost());
        assert_eq!(engine.get_value(), Wei::from_wei(100));
    }

    #[test]
    fn evmexec_math() {
        let ops = vec![
            PUSH1(2),
            PUSH1(3),
            PUSH1(4),
            PUSH1(7),
            PUSH1(1),
            ADD,
            SUB,
            MUL,
            DIV,
            SETVAL,
            STOP,
        ];
        let gascost = ops.iter().fold(0, |sum, x| sum + x.to_cost());
        let mut engine = ExecutionContext::new(gascost + 20, ops, Wei::from_wei(100));
        assert!(engine.finish_executing());
        assert_eq!(engine.get_gas_left(), 20);
        assert_eq!(engine.get_value(), Wei::from_wei(6));
    }

    #[test]
    fn evmexec_jumping_subval() {
        let ops = vec![
            PUSH1(2),
            PUSH1(3),
            GT,
            PUSH1(7),
            JUMPI,
            INVALID(0xff),
            INVALID(0xff),
            PUSH1(9),
            SUBVAL,
            STOP,
        ];
        let mut engine = ExecutionContext::new(1000, ops, Wei::from_wei(4));
        assert!(engine.finish_executing());
        assert_eq!(engine.get_value(), Wei::from_wei(0));
    }

    #[test]
    fn evmexec_infinite_loop() {
        let ops = vec![PUSH1(100), PUSH1(0), JUMPI, STOP];
        let mut engine = ExecutionContext::new(1000, ops, Wei::from_wei(4));
        assert!(!engine.finish_executing());
    }
}

use super::ops::Op;
use super::gas::Gas;
use super::wei::Wei;

pub struct ExecutionContext {
    pub stack: Vec<u8>,
    pub pc: u32,
    pub gas_left: Gas,
    pub code: Vec<Op>,
    pub txn_value: Wei,
}

impl ExecutionContext {

    // return true if terminated normally, false on error
    pub fn finish_executing(&mut self) -> bool {
        while let Ok(result) = self.execute_cycle() {
            if result != true {
                return true;
            }
        }
        return false; // got an error
    }

    fn pop(&mut self) -> Result<u8, ()> {
        self.stack.pop().ok_or(())
    }

    fn push(&mut self, b: u8) {
        self.stack.push(b);
    }
    
    // execute operations and modify the PC and stack
    // CONTINUE -> Ok(true), STOP -> Ok(false)
    // INVALID -> Err
    fn execute_cycle(&mut self) -> Result<bool, ()> {
        if self.pc as usize >= self.code.len() {
            // pc out of bounds
            return Err(())
        }
        // default pc increment
        let new_pc = self.pc + 1;
        let op = self.code[self.pc as usize];
        match op {
            Op::STOP => return Ok(false),
            Op::ADD => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(a + b);
            },
            Op::MUL => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(a * b);
            },
            Op::SUB => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(
                    match a.checked_sub(b) {
                        Some(x) => x,
                        None => 0,
                    }
                );
            },
            Op::DIV => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(
                    match a.checked_div(b) {
                        Some(x) => x,
                        None => 0,
                    }
                );
            },
            Op::LT => {
                let a = self.pop()?;
                let b = self.pop()?;
                if a < b {
                    self.push(1);
                } else {
                    self.push(0);
                }
            },
            Op::GT => {
                let a = self.pop()?;
                let b = self.pop()?;
                if a > b {
                    self.push(1);
                } else {
                    self.push(0);
                }
            },
            Op::EQ => {
                let a = self.pop()?;
                let b = self.pop()?;
                if a == b {
                    self.push(1);
                } else {
                    self.push(0);
                }
            },
            Op::ISZERO => {
                let a = self.pop()?;
                if a == 0 {
                    self.push(1);
                } else {
                    self.push(0);
                }
            },
            Op::ADDRESS => {
                
            },
            Op::BALANCE => {

            },
            Op::GASPRICE => {

            },
            Op::DIFFICULTY => {

            },
            Op::GASLIMIT => {

            },
            Op::POP => {
                self.pop()?;
            },
            Op::JUMP => {

            },
            Op::JUMPI => {

            },
            Op::GAS => {

            },
            Op::PUSH1(val) => {
                self.push(val);
            },
            Op::SETVAL => {

            },
            Op::ADDVAL => {

            },
            Op::SUBVAL => {

            },
            Op::INVALID(_) => {
                return Err(())
            },
        };
        self.pc = new_pc;
        self.gas_left = match self.gas_left.checked_sub(op.to_cost()) {
            Some(val) => val,
            None => return Err(())
        };
        Ok(true)
    }
}
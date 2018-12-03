use super::{gas, gas::Gas};

#[derive(Debug, PartialEq, Eq, Copy)]
pub enum Op {
    STOP,
    ADD,
    MUL,
    SUB,
    DIV,

    LT,
    GT,
    EQ,
    ISZERO,

    ADDRESS,
    BALANCE,
    GASPRICE,
    DIFFICULTY,
    GASLIMIT,
    
    POP,
    JUMP,
    JUMPI,
    GAS,

    PUSH1(u8),
    SETVAL,
    ADDVAL,
    SUBVAL,

    INVALID(u8),
}

impl Op {
    pub fn to_cost(&self) -> Gas {
        match &self {
            Op::STOP => gas::GZERO,
            Op::ADD => gas::GVERYLOW,
            Op::MUL => gas::GLOW,
            Op::SUB => gas::GVERYLOW,
            Op::DIV => gas::GLOW,

            Op::LT => gas::GVERYLOW,
            Op::GT => gas::GVERYLOW,
            Op::EQ => gas::GVERYLOW,
            Op::ISZERO => gas::GVERYLOW,

            Op::ADDRESS => gas::GBASE,
            Op::BALANCE => gas::GBALANCE,
            Op::GASPRICE => gas::GBASE,
            Op::DIFFICULTY => gas::GBASE,
            Op::GASLIMIT => gas::GBASE,

            Op::POP => gas::GBASE,
            Op::JUMP => gas::GMID,
            Op::JUMPI => gas::GHIGH,
            Op::GAS => gas::GBASE,

            Op::PUSH1(_) => gas::GVERYLOW,
            Op::SETVAL => gas::GBASE,
            Op::ADDVAL => gas::GBASE,
            Op::SUBVAL => gas::GBASE,

            Op::INVALID(_) => gas::GZERO,
        }
    }

    pub fn from_byte(byte: u8) -> Op {
        match byte {
            0x00 => Op::STOP,
            0x01 => Op::ADD,
            0x02 => Op::MUL,
            0x03 => Op::SUB,
            0x04 => Op::DIV,

            0x10 => Op::LT,
            0x11 => Op::GT,
            0x14 => Op::EQ,
            0x15 => Op::ISZERO,

            0x30 => Op::ADDRESS,
            0x31 => Op::BALANCE,
            0x3a => Op::GASPRICE,
            0x44 => Op::DIFFICULTY,
            0x45 => Op::GASLIMIT,

            0x50 => Op::POP,
            0x56 => Op::JUMP,
            0x57 => Op::JUMPI,
            0x5a => Op::GAS,

            0x60 => Op::PUSH1(0),
            0xb0 => Op::SETVAL,
            0xb1 => Op::ADDVAL,
            0xb2 => Op::SUBVAL,   

            x => Op::INVALID(x),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Vec<Op> {
        let mut ops = Vec::new();
        let mut iter = bytes.iter();
        loop {
            match iter.next() {
                None => break,
                Some(byte) => {
                    match Op::from_byte(*byte) {
                        Op::PUSH1(_) => {
                            if let Some(val) = iter.next() {
                                ops.push(Op::PUSH1(*val));
                            }
                        },
                        op => {
                            ops.push(op)
                        },
                    }
                }
            }
        }
        ops
    }
}

impl Clone for Op {
    fn clone(&self) -> Op {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::Op;

    fn compare_vecs<T: Eq>(v1:& Vec<T>, v2:& Vec<T>) -> bool {
        v1.iter()
            .zip(v2)
            .filter(|&(v1_el, v2_el)| {
                v1_el != v2_el
            })
            .count() == 0
    }

    #[test]
    fn test_stop() {
        let opcodes: Vec<u8> = vec![0x00];
        let actual = Op::from_bytes(opcodes);
        let expected = vec![Op::STOP];
        assert_eq!(compare_vecs(&actual, &expected), true);
    }

    #[test]
    fn test_stop_setval_difficulty_iszero() {
        let opcodes: Vec<u8> = vec![0x00, 0xb0, 0x44, 0x15];
        let actual = Op::from_bytes(opcodes);
        let expected = vec![Op::STOP, Op::SETVAL, Op::DIFFICULTY, Op::ISZERO];
        assert_eq!(compare_vecs(&actual, &expected), true);
    }

    #[test]
    fn test_balance_gasprice_jump_jumpi_pop_invalid() {
        let opcodes: Vec<u8> = vec![0x31, 0x3a, 0x56, 0x57, 0x50, 0x05];
        let actual = Op::from_bytes(opcodes);
        let expected = vec![Op::BALANCE, Op::GASPRICE, Op::JUMP, Op::JUMPI, Op::POP, Op::INVALID(0x05)];
        assert_eq!(compare_vecs(&actual, &expected), true);
    }

    #[test]
    fn test_lt_push_gt_eq() {
        let opcodes: Vec<u8> = vec![0x10, 0x60, 0x10, 0x11, 0x14];
        let actual = Op::from_bytes(opcodes);
        let expected = vec![Op::LT, Op::PUSH1(0x10), Op::GT, Op::EQ];
        assert_eq!(compare_vecs(&actual, &expected), true);
    }
}

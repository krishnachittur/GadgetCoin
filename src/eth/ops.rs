use super::{gas, gas::Gas};

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

    PUSH1,
    SETVAL,
    ADDVAL,
    SUBVAL,

    INVALID,
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
            Op::PUSH1 => gas::GVERYLOW,
            Op::SETVAL => gas::GBASE,
            Op::ADDVAL => gas::GBASE,
            Op::SUBVAL => gas::GBASE,
            Op::INVALID => gas::GZERO,
        }
    }

    pub fn from_opcode(opcode: u8) -> Op {
        match opcode {
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
            0x60 => Op::PUSH1,
            0xb0 => Op::SETVAL,
            0xb1 => Op::ADDVAL,
            0xb2 => Op::SUBVAL,            
            _ => Op::INVALID,            
        }
    }
}
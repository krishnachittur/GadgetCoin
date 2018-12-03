use std::ops::{Add, Sub, AddAssign};
use std::clone::Clone;
use super::gas::Gas;

#[derive(Debug, PartialEq, Eq, PartialOrd,
    Ord, Serialize, Copy)]
pub struct Wei {
    wei: u128,
}

impl Wei {
    pub const WEI_PER_GWEI: u128 = 1_000_000_000;
    pub const WEI_PER_SZABO: u128 = 1_000_000_000_000;
    pub const WEI_PER_FINNEY: u128 = 1_000_000_000_000_000;
    pub const WEI_PER_ETH: u128 = 1_000_000_000_000_000_000;

    pub fn get_wei(&self) -> u128 {
        self.wei
    }
    pub fn from_wei(init: u128) -> Self {
        Self{wei: init}
    }
    pub fn from_szabo(init: u128) -> Self {
        Self{wei: init * Self::WEI_PER_SZABO}
    }
    pub fn from_finney(init: u128) -> Self {
        Self{wei: init * Self::WEI_PER_FINNEY}
    }
    pub fn from_eth(init: u128) -> Self {
        Self{wei: init * Self::WEI_PER_ETH}
    }
    pub fn from_gwei(init: u128) -> Self {
        Self{wei: init * Self::WEI_PER_GWEI}
    }
    pub fn from_gas(gasprice: Wei, gas: Gas) -> Self {
        Self{wei: gasprice.wei * gas}
    }
}

impl Add for Wei {
    type Output = Wei;

    fn add(self, other: Wei) -> Self::Output {
        Self{wei: self.wei + other.wei}
    }
}

impl Sub for Wei {
    type Output = Option<Wei>;

    fn sub(self, other: Wei) -> Self::Output {
        self.wei
            .checked_sub(other.wei)
            .map(|c| Self{wei: c})
    }
}

impl AddAssign for Wei {
    fn add_assign(&mut self, other: Wei) {
        self.wei += other.wei;
    }
}


impl Clone for Wei {
    fn clone(&self) -> Wei {
        Self{wei: self.wei}
    }
}


#[cfg(test)]
mod tests {
    use super::Wei;
    #[test]
    fn test_wei_eq() {
        assert_eq!(
            Wei::from_wei(400 * Wei::WEI_PER_ETH),
            Wei::from_eth(400)
        );
        assert_ne!(
            Wei::from_szabo(300),
            Wei::from_finney(300)
        );
    }
    #[test]
    fn test_wei_cmp() {
        assert!(Wei::from_eth(30) < Wei::from_eth(31));
        assert!(Wei::from_eth(40) > Wei::from_szabo(40));
    }
    #[test]
    fn test_wei_math() {
        assert_eq!(
            Wei::from_szabo(10),
            Wei::from_szabo(7) + Wei::from_szabo(3)
        );
        assert_eq!(
            Wei::from_wei(50) - Wei::from_wei(51),
            None
        );
        assert_eq!(
            Wei::from_wei(56) - Wei::from_wei(51),
            Some(Wei::from_wei(5))
        );
    }
}

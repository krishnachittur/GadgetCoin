use std::ops::{Add, Sub};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gas {
    wei: u128,
}

impl Gas {
    pub const SZABO_PER_WEI: u128 = 1_000_000_000_000;
    pub const FINNEY_PER_WEI: u128 = 1_000_000_000_000_000;
    pub const ETH_PER_WEI: u128 = 1_000_000_000_000_000_000;

    pub fn from_wei(init: u128) -> Self {
        Self{wei: init}
    }
    pub fn from_szabo(init: u128) -> Self {
        Self{wei: init * Self::SZABO_PER_WEI}
    }
    pub fn from_finney(init: u128) -> Self {
        Self{wei: init * Self::FINNEY_PER_WEI}
    }
    pub fn from_eth(init: u128) -> Self {
        Self{wei: init * Self::ETH_PER_WEI}
    }
}

impl Add for Gas {
    type Output = Gas;

    fn add(self, other: Gas) -> Self::Output {
        Self{wei: self.wei + other.wei}
    }
}

impl Sub for Gas {
    type Output = Option<Gas>;

    fn sub(self, other: Gas) -> Self::Output {
        self.wei
            .checked_sub(other.wei)
            .map(|c| Self{wei: c})
    }
}


#[cfg(test)]
mod tests {
    use super::Gas;
    #[test]
    fn test_gas_eq() {
        assert_eq!(
            Gas::from_wei(400 * Gas::ETH_PER_WEI),
            Gas::from_eth(400)
        );
        assert_ne!(
            Gas::from_szabo(300),
            Gas::from_finney(300)
        );
    }
    #[test]
    fn test_gas_cmp() {
        assert!(Gas::from_eth(30) < Gas::from_eth(31));
        assert!(Gas::from_eth(40) > Gas::from_szabo(40));
    }
    #[test]
    fn test_gas_math() {
        assert_eq!(
            Gas::from_szabo(10),
            Gas::from_szabo(7) + Gas::from_szabo(3)
        );
        assert_eq!(
            Gas::from_wei(50) - Gas::from_wei(51),
            None
        );
        assert_eq!(
            Gas::from_wei(56) - Gas::from_wei(51),
            Some(Gas::from_wei(5))
        );
    }
}
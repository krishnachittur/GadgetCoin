#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gas {
    wei: u128,
}

impl Gas {
    const SZABO_PER_WEI: u128 = 1_000_000_000_000;
    const FINNEY_PER_WEI: u128 = 1_000_000_000_000_000;
    const ETH_PER_WEI: u128 = 1_000_000_000_000_000_000;

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
    pub fn add(&self, other: Self) -> Self {
        Self{wei: self.wei + other.wei}
    }
    /// returns Option since negative Gas is invalid 
    pub fn sub(&self, other: Self) -> Option<Self> {
        self.wei
            .checked_sub(other.wei)
            .map(|c| Self{wei: c})
    }
}
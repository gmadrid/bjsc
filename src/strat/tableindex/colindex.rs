use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ColIndex(u8);

impl ColIndex {
    fn new(val: u8) -> Result<ColIndex, ()> {
        if !(1..=10).contains(&val) {
            return Err(());
        }
        Ok(ColIndex(val))
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl FromStr for ColIndex {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: u8 = s.parse().map_err(|_| ())?;
        ColIndex::new(val)
    }
}

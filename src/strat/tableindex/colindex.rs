use crate::card::Card;
use crate::{BjError, BjResult};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ColIndex(u8);

impl ColIndex {
    pub fn new_with_card(card: Card) -> BjResult<Self> {
        ColIndex::new(card.value())
    }

    fn new(val: u8) -> BjResult<ColIndex> {
        if val == 11 {
            return Ok(ColIndex(1));
        }

        if !(1..=10).contains(&val) {
            return Err(BjError::ValueOutOfRange(val, 1, 10));
        }
        Ok(ColIndex(val))
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Display for ColIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ColIndex {
    type Err = BjError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: u8 = s.parse()?;
        ColIndex::new(val)
    }
}

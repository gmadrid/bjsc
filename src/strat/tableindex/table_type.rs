use crate::BjError::{UnknownTableType, ValueOutOfRange};
use crate::{BjError, BjResult};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TableType {
    Hard,
    Soft,
    Split,
    Surrender,
}

impl TableType {
    // TODO: verify these ranges against the strategy tables.
    pub fn range_check(self, row: u8) -> BjResult<()> {
        match self {
            TableType::Hard | TableType::Soft => {
                if (2u8..=21).contains(&row) {
                    Ok(())
                } else {
                    Err(BjError::ValueOutOfRange(row, 2, 21))
                }
            }
            TableType::Split => {
                // TODO: we need to fix this to use cards, not totals.
                if (2u8..=21).contains(&row) && row % 2 == 0 {
                    Ok(())
                } else {
                    Err(ValueOutOfRange(row, 2, 21))
                }
            }
            TableType::Surrender => {
                if (2u8..=20).contains(&row) {
                    Ok(())
                } else {
                    Err(ValueOutOfRange(row, 2, 20))
                }
            }
        }
    }
}

impl Display for TableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TableType::Hard => "hard",
            TableType::Soft => "sort",
            TableType::Split => "split",
            TableType::Surrender => "surrender",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for TableType {
    type Err = BjError;
    fn from_str(s: &str) -> BjResult<Self> {
        match s {
            "hard" => Ok(TableType::Hard),
            "soft" => Ok(TableType::Soft),
            "split" => Ok(TableType::Split),
            "surrender" => Ok(TableType::Surrender),
            _ => Err(UnknownTableType(s.to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_table_index() {
        assert_eq!(TableType::Hard, "hard".parse().unwrap());
        assert_eq!(TableType::Soft, "soft".parse().unwrap());
        assert_eq!(TableType::Split, "split".parse().unwrap());
        assert_eq!(TableType::Surrender, "surrender".parse().unwrap());
        assert_eq!(
            Err(BjError::UnknownTableType("hards".to_string())),
            "hards".parse::<TableType>()
        );
    }

    #[test]
    fn test_hard_range_check() {
        assert!(TableType::Hard.range_check(0).is_err());
        assert!(TableType::Hard.range_check(1).is_err());
        assert!(TableType::Hard.range_check(2).is_ok());
        assert!(TableType::Hard.range_check(21).is_ok());
        assert!(TableType::Hard.range_check(22).is_err());
    }

    #[test]
    fn test_soft_range_check() {
        assert!(TableType::Soft.range_check(0).is_err());
        assert!(TableType::Soft.range_check(1).is_err());
        assert!(TableType::Soft.range_check(2).is_ok());
        assert!(TableType::Soft.range_check(21).is_ok());
        assert!(TableType::Soft.range_check(22).is_err());
    }

    #[test]
    fn test_split_range_check() {
        assert!(TableType::Split.range_check(0).is_err());
        assert!(TableType::Split.range_check(1).is_err());
        assert!(TableType::Split.range_check(2).is_ok());
        assert!(TableType::Split.range_check(10).is_ok());
        assert!(TableType::Split.range_check(11).is_err());
        assert!(TableType::Split.range_check(20).is_ok());
        assert!(TableType::Split.range_check(21).is_err());
        assert!(TableType::Split.range_check(22).is_err());
    }

    #[test]
    fn test_surrender_range_check() {
        assert!(TableType::Surrender.range_check(0).is_err());
        assert!(TableType::Surrender.range_check(1).is_err());
        assert!(TableType::Surrender.range_check(2).is_ok());
        assert!(TableType::Surrender.range_check(20).is_ok());
        assert!(TableType::Surrender.range_check(21).is_err());
    }
}

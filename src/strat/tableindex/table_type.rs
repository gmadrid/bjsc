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
    pub fn range_check(self, row: u8) -> Result<(), ()> {
        match self {
            TableType::Hard | TableType::Soft => {
                if (2u8..=21).contains(&row) {
                    Ok(())
                } else {
                    Err(())
                }
            }
            TableType::Split => {
                if (1u8..=10).contains(&row) {
                    Ok(())
                } else {
                    Err(())
                }
            }
            TableType::Surrender => {
                if (2u8..=20).contains(&row) {
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }
}

impl FromStr for TableType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hard" => Ok(TableType::Hard),
            "soft" => Ok(TableType::Soft),
            "split" => Ok(TableType::Split),
            "surrender" => Ok(TableType::Surrender),
            _ => Err(()),
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
        assert_eq!(Err(()), "hards".parse::<TableType>());
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
        assert!(TableType::Split.range_check(1).is_ok());
        assert!(TableType::Split.range_check(2).is_ok());
        assert!(TableType::Split.range_check(10).is_ok());
        assert!(TableType::Split.range_check(11).is_err());
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

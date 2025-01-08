use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TableIndex {
    pub table_type: TableType,
    pub cell: Cell,
}

impl FromStr for TableIndex {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pieces = s.split(":");
        let tt_val = pieces.next().ok_or(())?.trim();
        let cell_val = pieces.next().ok_or(())?.trim();
        if pieces.next().is_some() {
            return Err(());
        }
        let table_type = TableType::from_str(tt_val).map_err(|_| ())?;
        let cell = Cell::from_str(cell_val).map_err(|_| ())?;

        table_type.range_check(cell.row)?;

        Ok(TableIndex { table_type, cell })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TableType {
    Hard,
    Soft,
    Split,
    Surrender,
}

impl TableType {
    fn range_check(self, row: u8) -> Result<(), ()> {
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Cell {
    // row has no range restrictions. Range will be imposed by the table type.
    pub row: u8,
    pub col: DealerCard,
}

impl Cell {
    pub fn new(row: u8, col: DealerCard) -> Cell {
        Cell { row, col }
    }
}

impl FromStr for Cell {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pieces = s.split(",");
        let left = pieces.next().ok_or(())?.trim();
        let right = pieces.next().ok_or(())?.trim();
        if pieces.next().is_some() {
            return Err(());
        }

        let row = left.parse::<u8>().map_err(|_| ())?;
        let col = right.parse::<DealerCard>()?;

        Ok(Cell { row, col })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct DealerCard(u8);

impl DealerCard {
    fn new(val: u8) -> Result<DealerCard, ()> {
        if !(1..=10).contains(&val) {
            return Err(());
        }
        Ok(DealerCard(val))
    }
}

impl FromStr for DealerCard {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: u8 = s.parse().map_err(|_| ())?;
        DealerCard::new(val)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hard_index() {
        assert_eq!(
            TableIndex {
                table_type: TableType::Hard,
                cell: Cell {
                    row: 3,
                    col: DealerCard::new(4).unwrap()
                },
            },
            "hard:3,4".parse().unwrap()
        );
        assert_eq!(
            TableIndex {
                table_type: TableType::Hard,
                cell: Cell {
                    row: 8,
                    col: DealerCard::new(1).unwrap()
                },
            },
            "  hard   :   8   ,    1   ".parse().unwrap()
        );

        assert_eq!(Err(()), TableIndex::from_str("hard:4"));
        assert_eq!(Err(()), TableIndex::from_str("hard:4,8:8"));
        assert_eq!(Err(()), TableIndex::from_str("hard:1,1"));
        assert_eq!(Err(()), TableIndex::from_str("hard:22,1"));
        assert_eq!(Err(()), TableIndex::from_str("hard:1,22"));
    }

    #[test]
    fn test_soft_index() {
        assert_eq!(
            TableIndex {
                table_type: TableType::Soft,
                cell: Cell {
                    row: 3,
                    col: DealerCard::new(4).unwrap()
                },
            },
            "soft:3,4".parse().unwrap()
        );
        assert_eq!(
            TableIndex {
                table_type: TableType::Soft,
                cell: Cell {
                    row: 8,
                    col: DealerCard::new(1).unwrap()
                },
            },
            "  soft   :   8   ,    1   ".parse().unwrap()
        );

        assert_eq!(Err(()), TableIndex::from_str("soft:4"));
        assert_eq!(Err(()), TableIndex::from_str("soft:4,8:8"));
        assert_eq!(Err(()), TableIndex::from_str("soft:1,1"));
        assert_eq!(Err(()), TableIndex::from_str("soft:22,1"));
        assert_eq!(Err(()), TableIndex::from_str("soft:1,22"));
    }

    #[test]
    fn test_split_index() {
        assert_eq!(
            TableIndex {
                table_type: TableType::Split,
                cell: Cell {
                    row: 3,
                    col: DealerCard::new(4).unwrap()
                },
            },
            "split:3,4".parse().unwrap()
        );
        assert_eq!(
            TableIndex {
                table_type: TableType::Split,
                cell: Cell {
                    row: 8,
                    col: DealerCard::new(1).unwrap()
                },
            },
            "  split   :   8   ,    1   ".parse().unwrap()
        );

        assert_eq!(Err(()), TableIndex::from_str("split:4"));
        assert_eq!(Err(()), TableIndex::from_str("split:4,8:8"));
        assert_eq!(Err(()), TableIndex::from_str("split:0,1"));
        assert_eq!(Err(()), TableIndex::from_str("split:11,1"));
        assert_eq!(Err(()), TableIndex::from_str("split:10,22"));
    }

    #[test]
    fn test_parse_table_index() {
        assert_eq!(TableType::Hard, "hard".parse().unwrap());
        assert_eq!(TableType::Soft, "soft".parse().unwrap());
        assert_eq!(TableType::Split, "split".parse().unwrap());
        assert_eq!(TableType::Surrender, "surrender".parse().unwrap());
        assert_eq!(Err(()), "hards".parse::<TableType>());
    }

    #[test]
    fn test_new_dealer_card() {
        assert_eq!(DealerCard(1), DealerCard::new(1).unwrap());
        assert_eq!(DealerCard(2), DealerCard::new(2).unwrap());
        assert_eq!(DealerCard(3), DealerCard::new(3).unwrap());
        assert_eq!(DealerCard(4), DealerCard::new(4).unwrap());
        assert_eq!(DealerCard(5), DealerCard::new(5).unwrap());
        assert_eq!(DealerCard(6), DealerCard::new(6).unwrap());
        assert_eq!(DealerCard(7), DealerCard::new(7).unwrap());
        assert_eq!(DealerCard(8), DealerCard::new(8).unwrap());
        assert_eq!(DealerCard(9), DealerCard::new(9).unwrap());
        assert_eq!(DealerCard(10), DealerCard::new(10).unwrap());

        assert_eq!(Err(()), DealerCard::new(0));
        assert_eq!(Err(()), DealerCard::new(11));
    }

    #[test]
    fn test_parse_dealer_card() {
        assert_eq!(DealerCard(1), "1".parse().unwrap());
        assert_eq!(DealerCard(1), "01".parse().unwrap());
        assert_eq!(DealerCard(10), "10".parse().unwrap());

        assert_eq!(Err(()), "0".parse::<DealerCard>());
        assert_eq!(Err(()), "11".parse::<DealerCard>());
        assert_eq!(Err(()), "hard".parse::<DealerCard>());
    }

    #[test]
    fn test_parse_cell() {
        let dc_ace = DealerCard::new(1).unwrap();
        let dc_five = DealerCard::new(5).unwrap();

        assert_eq!(Cell::new(1, dc_ace), "1,1".parse::<Cell>().unwrap());
        assert_eq!(Cell::new(7, dc_five), "7,5".parse::<Cell>().unwrap());

        assert_eq!(
            Cell::new(50, dc_ace),
            "   50  , 1 ".parse::<Cell>().unwrap()
        );

        // Errors
        assert_eq!(Err(()), "".parse::<Cell>());
        assert_eq!(Err(()), "x, 5".parse::<Cell>());
        assert_eq!(Err(()), "5, x".parse::<Cell>());
        assert_eq!(Err(()), "5, 6, 7".parse::<Cell>());
    }
}

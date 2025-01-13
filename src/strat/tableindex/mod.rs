mod colindex;
mod rowindex;
mod table_type;

use crate::strat::tableindex::rowindex::RowIndex;
use crate::{BjError, BjResult};
pub use colindex::ColIndex;
use std::fmt::Display;
use std::str::FromStr;
pub use table_type::TableType;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TableIndex {
    pub row: RowIndex,
    pub col: ColIndex,
}

impl TableIndex {
    fn new(row: RowIndex, col: ColIndex) -> BjResult<TableIndex> {
        Ok(TableIndex { row, col })
    }

    pub fn table_type(&self) -> TableType {
        self.row.table_type
    }

    pub fn row_index(&self) -> u8 {
        self.row.index
    }

    pub fn col_index(&self) -> ColIndex {
        self.col
    }
}

impl Display for TableIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.row, self.col)
    }
}

impl FromStr for TableIndex {
    type Err = BjError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (row_str, col_str) = s
            .split_once(',')
            .ok_or(BjError::BadTableIndex(s.to_string()))?;
        let row = RowIndex::from_str(row_str.trim())?;
        let col = ColIndex::from_str(col_str.trim())?;
        TableIndex::new(row, col)
    }
}

#[cfg(test)]
mod test {
    // use super::*;

    // #[test]
    // fn test_new_table_index() {
    //     let row = "hard:5".parse::<RowIndex>().unwrap();
    //     let col = "6".parse::<ColIndex>().unwrap();
    //
    //     let index = TableIndex::new(row, col).unwrap();
    //     assert_eq!(TableIndex { row, col }, index);
    // }
    //
    // #[test]
    // fn test_parse() {
    //     let row = "hard:5".parse::<RowIndex>().unwrap();
    //     let col = "6".parse::<ColIndex>().unwrap();
    //
    //     let index = TableIndex::new(row, col).unwrap();
    //     assert_eq!(index, "hard:5,6".parse().unwrap());
    // }
}

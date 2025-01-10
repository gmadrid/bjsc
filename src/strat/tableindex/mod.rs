use crate::strat::tableindex::rowindex::RowIndex;
use std::str::FromStr;

mod colindex;
mod rowindex;
mod table_type;

pub use colindex::ColIndex;
pub use table_type::TableType;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TableIndex {
    pub row: RowIndex,
    pub col: ColIndex,
}

impl TableIndex {
    fn new(row: RowIndex, col: ColIndex) -> Result<TableIndex, ()> {
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

impl FromStr for TableIndex {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (row_str, col_str) = s.split_once(',').ok_or(())?;
        let row = RowIndex::from_str(row_str.trim())?;
        let col = ColIndex::from_str(col_str.trim())?;
        TableIndex::new(row, col)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_table_index() {
        let row = "hard:5".parse::<RowIndex>().unwrap();
        let col = "6".parse::<ColIndex>().unwrap();

        let index = TableIndex::new(row, col).unwrap();
        assert_eq!(TableIndex { row, col }, index);
    }

    #[test]
    fn test_parse() {
        let row = "hard:5".parse::<RowIndex>().unwrap();
        let col = "6".parse::<ColIndex>().unwrap();

        let index = TableIndex::new(row, col).unwrap();
        assert_eq!(index, "hard:5,6".parse().unwrap());
    }
}

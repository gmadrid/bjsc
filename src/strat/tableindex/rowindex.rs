use crate::strat::tableindex::table_type::TableType;
use crate::BjError::BadRowIndex;
use crate::{BjError, BjResult};
use std::fmt::Display;
use std::str::FromStr;

// The RowIndex corresponds to a row in a Strategy Chart.
// In indicates both the name of the table and the row in the chart.
// The row number is interpreted differently for each table.
// See the Table in question for details.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct RowIndex {
    pub table_type: TableType,
    pub index: u8,
}

impl RowIndex {
    pub fn new(table_type: TableType, index: u8) -> BjResult<Self> {
        table_type.range_check(index)?;
        Ok(RowIndex { table_type, index })
    }
}

impl Display for RowIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.table_type, self.index)
    }
}

impl FromStr for RowIndex {
    type Err = BjError;

    fn from_str(row: &str) -> BjResult<Self> {
        let (table_str, index_str) = row.split_once(":").ok_or(BadRowIndex(row.to_string()))?;
        RowIndex::new(table_str.parse()?, index_str.parse()?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_row_index() {
        assert_eq!(
            RowIndex {
                table_type: "hard".parse().unwrap(),
                index: 3
            },
            RowIndex::new(TableType::Hard, 3).unwrap()
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            RowIndex::new(TableType::Hard, 3).unwrap(),
            "hard:3".parse().unwrap()
        );
    }

    #[test]
    fn test_range_check_works() {
        assert_eq!(
            Err(BjError::ValueOutOfRange(0, 2, 21)),
            "hard:0".parse::<RowIndex>()
        );
    }
}

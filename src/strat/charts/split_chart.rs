use crate::strat::charts::ChartAction::{NSpt, SDas, Splt};
use crate::strat::charts::{as_chart_column, ChartAction};
use crate::strat::tableindex::{TableIndex, TableType};

// Standard Basic Strategy Pair Splitting from BJA
const SPLIT_CHART: [[ChartAction; 10]; 10] = [
    /* A, A */
    [Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt],
    /* 2, 2 */
    [SDas, SDas, Splt, Splt, Splt, Splt, NSpt, NSpt, NSpt, NSpt],
    /* 3, 3 */
    [SDas, SDas, Splt, Splt, Splt, Splt, NSpt, NSpt, NSpt, NSpt],
    /* 4, 4 */
    [NSpt, NSpt, NSpt, SDas, SDas, NSpt, NSpt, NSpt, NSpt, NSpt],
    /* 5, 5 */
    [NSpt, NSpt, NSpt, NSpt, NSpt, NSpt, NSpt, NSpt, NSpt, NSpt],
    /* 6, 6 */
    [SDas, Splt, Splt, Splt, Splt, NSpt, NSpt, NSpt, NSpt, NSpt],
    /* 7, 7 */
    [Splt, Splt, Splt, Splt, Splt, Splt, NSpt, NSpt, NSpt, NSpt],
    /* 8, 8 */
    [Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt, Splt],
    /* 9, 9 */
    [Splt, Splt, Splt, Splt, Splt, NSpt, Splt, Splt, NSpt, NSpt],
    /* T, T */
    [NSpt, NSpt, NSpt, NSpt, NSpt, NSpt, NSpt, NSpt, NSpt, NSpt],
];

pub fn lookup(index: TableIndex) -> Result<ChartAction, ()> {
    if index.table_type() != TableType::Split {
        return Err(());
    }

    let row_index = index.row_index();
    let col_index = index.col_index();
    let chart_index = as_chart_column(col_index);

    Ok(SPLIT_CHART[(row_index - 1) as usize][chart_index])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::strat::charts::test::ti;

    #[test]
    fn test_lookup() {
        assert_eq!(SDas, lookup(ti("split:2, 2")).unwrap());
        assert_eq!(SDas, lookup(ti("split:2, 3")).unwrap());
        assert_eq!(Splt, lookup(ti("split:2, 4")).unwrap());
        assert_eq!(Splt, lookup(ti("split:2, 7")).unwrap());
        assert_eq!(NSpt, lookup(ti("split:2, 8")).unwrap());
        assert_eq!(NSpt, lookup(ti("split:2, 10")).unwrap());
        assert_eq!(NSpt, lookup(ti("split:2, 1")).unwrap());

        assert_eq!(NSpt, lookup(ti("split:4, 2")).unwrap());
        assert_eq!(NSpt, lookup(ti("split:4, 4")).unwrap());
        assert_eq!(SDas, lookup(ti("split:4, 5")).unwrap());
        assert_eq!(SDas, lookup(ti("split:4, 6")).unwrap());
        assert_eq!(NSpt, lookup(ti("split:4, 7")).unwrap());
        assert_eq!(NSpt, lookup(ti("split:4, 10")).unwrap());
        assert_eq!(NSpt, lookup(ti("split:4, 1")).unwrap());

        assert_eq!(Splt, lookup(ti("split:9, 2")).unwrap());
        assert_eq!(Splt, lookup(ti("split:9, 6")).unwrap());
        assert_eq!(NSpt, lookup(ti("split:9, 7")).unwrap());
        assert_eq!(Splt, lookup(ti("split:9, 8")).unwrap());
        assert_eq!(Splt, lookup(ti("split:9, 9")).unwrap());
        assert_eq!(NSpt, lookup(ti("split:9, 10")).unwrap());
        assert_eq!(NSpt, lookup(ti("split:9, 1")).unwrap());

        assert_eq!(Splt, lookup(ti("split:1, 2")).unwrap());
        assert_eq!(Splt, lookup(ti("split:1, 6")).unwrap());
        assert_eq!(Splt, lookup(ti("split:1, 7")).unwrap());
        assert_eq!(Splt, lookup(ti("split:1, 8")).unwrap());
        assert_eq!(Splt, lookup(ti("split:1, 9")).unwrap());
        assert_eq!(Splt, lookup(ti("split:1, 10")).unwrap());
        assert_eq!(Splt, lookup(ti("split:1, 1")).unwrap());
    }
}

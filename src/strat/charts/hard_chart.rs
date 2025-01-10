use crate::strat::charts::ChartAction::{DblH, Hit_, Stnd};
use crate::strat::charts::{as_chart_column, ChartAction};
use crate::strat::tableindex::{TableIndex, TableType};

// Standard Basic Strategy Hard Totals from BJA
const HARD_CHART: [[ChartAction; 10]; 10] = [
    /* 8 and lower */
    [Hit_, Hit_, Hit_, Hit_, Hit_, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 9 */
    [Hit_, DblH, DblH, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 10 */
    [DblH, DblH, DblH, DblH, DblH, DblH, DblH, DblH, Hit_, Hit_],
    /* 11 */
    [DblH, DblH, DblH, DblH, DblH, DblH, DblH, DblH, DblH, DblH],
    /* 12 */
    [Hit_, Hit_, Stnd, Stnd, Stnd, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 13 */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 14 */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 15 */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 16 */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 17+ */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd],
];

pub fn lookup(index: TableIndex) -> Result<ChartAction, ()> {
    if index.table_type() != TableType::Hard {
        return Err(());
    }

    let row_index = index.row_index();
    let col_index = index.col_index();
    let chart_index = as_chart_column(col_index);

    if row_index <= 8 {
        Ok(HARD_CHART[0][chart_index])
    } else if row_index >= 17 {
        Ok(HARD_CHART[9][chart_index])
    } else {
        Ok(HARD_CHART[(row_index - 8) as usize][chart_index])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::strat::charts::test::ti;

    #[test]
    fn test_lookup_low() {
        // Hit at or below 8
        assert_eq!(Hit_, lookup(ti("hard:2,1")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:3,2")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:4,3")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:5,4")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:6,5")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:7,6")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:8,10")).unwrap());
    }

    #[test]
    fn test_lookup_mid() {
        // We are spot-checking some key values to try to check the chart lookup.
        assert_eq!(Hit_, lookup(ti("hard:9,2")).unwrap());
        assert_eq!(DblH, lookup(ti("hard:9,3")).unwrap());
        assert_eq!(DblH, lookup(ti("hard:9,6")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:9,7")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:9,1")).unwrap());

        assert_eq!(DblH, lookup(ti("hard:11,2")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:12,2")).unwrap());
        assert_eq!(Stnd, lookup(ti("hard:13,2")).unwrap());

        assert_eq!(DblH, lookup(ti("hard:11,3")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:12,3")).unwrap());
        assert_eq!(Stnd, lookup(ti("hard:13,3")).unwrap());

        assert_eq!(DblH, lookup(ti("hard:11,4")).unwrap());
        assert_eq!(Stnd, lookup(ti("hard:12,4")).unwrap());
        assert_eq!(Stnd, lookup(ti("hard:13,4")).unwrap());

        assert_eq!(DblH, lookup(ti("hard:11,6")).unwrap());
        assert_eq!(Stnd, lookup(ti("hard:12,6")).unwrap());
        assert_eq!(Stnd, lookup(ti("hard:13,6")).unwrap());

        assert_eq!(DblH, lookup(ti("hard:11,7")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:12,7")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:13,7")).unwrap());

        assert_eq!(DblH, lookup(ti("hard:11,9")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:12,9")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:13,9")).unwrap());

        assert_eq!(Hit_, lookup(ti("hard:10,10")).unwrap());
        assert_eq!(DblH, lookup(ti("hard:11,10")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:12,10")).unwrap());

        assert_eq!(Hit_, lookup(ti("hard:10,1")).unwrap());
        assert_eq!(DblH, lookup(ti("hard:11,1")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:12,1")).unwrap());
        assert_eq!(Hit_, lookup(ti("hard:13,1")).unwrap());
    }

    #[test]
    fn test_lookup_high() {
        // Stand at or above 17
        assert_eq!(Stnd, lookup(ti("hard:17,1")).unwrap());
        assert_eq!(Stnd, lookup(ti("hard:18,3")).unwrap());
        assert_eq!(Stnd, lookup(ti("hard:19,7")).unwrap());
        assert_eq!(Stnd, lookup(ti("hard:20,9")).unwrap());
        assert_eq!(Stnd, lookup(ti("hard:21,10")).unwrap());
    }
}

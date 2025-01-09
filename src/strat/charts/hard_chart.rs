use crate::strat::charts::ChartAction::{DBLH, HIT_, STND};
use crate::strat::charts::{as_chart_column, ChartAction};
use crate::strat::tableindex::{TableIndex, TableType};

// Standard Basic Strategy Hard Totals from BJA
const HARD_CHART: [[ChartAction; 10]; 10] = [
    /* 8 and lower */
    [HIT_, HIT_, HIT_, HIT_, HIT_, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 9 */
    [HIT_, DBLH, DBLH, DBLH, DBLH, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 10 */
    [DBLH, DBLH, DBLH, DBLH, DBLH, DBLH, DBLH, DBLH, HIT_, HIT_],
    /* 11 */
    [DBLH, DBLH, DBLH, DBLH, DBLH, DBLH, DBLH, DBLH, DBLH, DBLH],
    /* 12 */
    [HIT_, HIT_, STND, STND, STND, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 13 */
    [STND, STND, STND, STND, STND, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 14 */
    [STND, STND, STND, STND, STND, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 15 */
    [STND, STND, STND, STND, STND, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 16 */
    [STND, STND, STND, STND, STND, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 17+ */
    [STND, STND, STND, STND, STND, STND, STND, STND, STND, STND],
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

    fn ti(s: &str) -> TableIndex {
        s.parse().unwrap()
    }

    #[test]
    fn test_lookup_low() {
        // Hit at or below 8
        assert_eq!(HIT_, lookup(ti("hard:2,1")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:3,2")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:4,3")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:5,4")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:6,5")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:7,6")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:8,10")).unwrap());
    }

    #[test]
    fn test_lookup_mid() {
        // We are spot-checking some key values to try to check the chart lookup.
        assert_eq!(HIT_, lookup(ti("hard:9,2")).unwrap());
        assert_eq!(DBLH, lookup(ti("hard:9,3")).unwrap());
        assert_eq!(DBLH, lookup(ti("hard:9,6")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:9,7")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:9,1")).unwrap());

        assert_eq!(DBLH, lookup(ti("hard:11,2")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:12,2")).unwrap());
        assert_eq!(STND, lookup(ti("hard:13,2")).unwrap());

        assert_eq!(DBLH, lookup(ti("hard:11,3")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:12,3")).unwrap());
        assert_eq!(STND, lookup(ti("hard:13,3")).unwrap());

        assert_eq!(DBLH, lookup(ti("hard:11,4")).unwrap());
        assert_eq!(STND, lookup(ti("hard:12,4")).unwrap());
        assert_eq!(STND, lookup(ti("hard:13,4")).unwrap());

        assert_eq!(DBLH, lookup(ti("hard:11,6")).unwrap());
        assert_eq!(STND, lookup(ti("hard:12,6")).unwrap());
        assert_eq!(STND, lookup(ti("hard:13,6")).unwrap());

        assert_eq!(DBLH, lookup(ti("hard:11,7")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:12,7")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:13,7")).unwrap());

        assert_eq!(DBLH, lookup(ti("hard:11,9")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:12,9")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:13,9")).unwrap());

        assert_eq!(HIT_, lookup(ti("hard:10,10")).unwrap());
        assert_eq!(DBLH, lookup(ti("hard:11,10")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:12,10")).unwrap());

        assert_eq!(HIT_, lookup(ti("hard:10,1")).unwrap());
        assert_eq!(DBLH, lookup(ti("hard:11,1")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:12,1")).unwrap());
        assert_eq!(HIT_, lookup(ti("hard:13,1")).unwrap());
    }

    #[test]
    fn test_lookup_high() {
        // Stand at or above 17
        assert_eq!(STND, lookup(ti("hard:17,1")).unwrap());
        assert_eq!(STND, lookup(ti("hard:18,3")).unwrap());
        assert_eq!(STND, lookup(ti("hard:19,7")).unwrap());
        assert_eq!(STND, lookup(ti("hard:20,9")).unwrap());
        assert_eq!(STND, lookup(ti("hard:21,10")).unwrap());
    }
}

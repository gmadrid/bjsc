use crate::strat::charts::ChartAction::{DblH, DblS, Hit_, Stnd};
use crate::strat::charts::{as_chart_column, ChartAction};
use crate::strat::tableindex::{TableIndex, TableType};

// Standard Basic Strategy Soft Totals from BJA
const SOFT_CHART: [[ChartAction; 10]; 8] = [
    /* 13 (A, 2) */
    [Hit_, Hit_, Hit_, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 14 (A, 3) */
    [Hit_, Hit_, Hit_, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 15 (A, 4) */
    [Hit_, Hit_, DblH, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 16 (A, 5) */
    [Hit_, Hit_, DblH, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 17 (A, 6) */
    [Hit_, DblH, DblH, DblH, DblH, Hit_, Hit_, Hit_, Hit_, Hit_],
    /* 18 (A, 7) */
    [DblS, DblS, DblS, DblS, DblS, Stnd, Stnd, Hit_, Hit_, Hit_],
    /* 19 (A, 8) */
    [Stnd, Stnd, Stnd, Stnd, DblS, Stnd, Stnd, Stnd, Stnd, Stnd],
    /* 20 (A, 9) */
    [Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd, Stnd],
];

pub fn lookup(index: TableIndex) -> Result<ChartAction, ()> {
    if index.table_type() != TableType::Soft {
        return Err(());
    }

    let row_index = index.row_index();

    if !(13..=20).contains(&row_index) {
        return Err(());
    }

    let col_index = index.col_index();
    let chart_index = as_chart_column(col_index);
    Ok(SOFT_CHART[(row_index - 13) as usize][chart_index])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::strat::charts::test::ti;

    #[test]
    fn test_lookup() {
        assert_eq!(Hit_, lookup(ti("soft:13,1")).unwrap());
        assert_eq!(Hit_, lookup(ti("soft:13,2")).unwrap());
        assert_eq!(Hit_, lookup(ti("soft:13,4")).unwrap());
        assert_eq!(DblH, lookup(ti("soft:13,5")).unwrap());
        assert_eq!(DblH, lookup(ti("soft:13,6")).unwrap());
        assert_eq!(Hit_, lookup(ti("soft:13,7")).unwrap());

        assert_eq!(Hit_, lookup(ti("soft:17,1")).unwrap());
        assert_eq!(Hit_, lookup(ti("soft:17,2")).unwrap());
        assert_eq!(DblH, lookup(ti("soft:17,3")).unwrap());
        assert_eq!(DblH, lookup(ti("soft:17,5")).unwrap());
        assert_eq!(DblH, lookup(ti("soft:17,6")).unwrap());
        assert_eq!(Hit_, lookup(ti("soft:17,7")).unwrap());

        assert_eq!(Hit_, lookup(ti("soft:18,1")).unwrap());
        assert_eq!(DblS, lookup(ti("soft:18,2")).unwrap());
        assert_eq!(DblS, lookup(ti("soft:18,3")).unwrap());
        assert_eq!(DblS, lookup(ti("soft:18,5")).unwrap());
        assert_eq!(DblS, lookup(ti("soft:18,6")).unwrap());
        assert_eq!(Stnd, lookup(ti("soft:18,7")).unwrap());
        assert_eq!(Stnd, lookup(ti("soft:18,8")).unwrap());
        assert_eq!(Hit_, lookup(ti("soft:18,10")).unwrap());

        assert_eq!(Stnd, lookup(ti("soft:19,1")).unwrap());
        assert_eq!(Stnd, lookup(ti("soft:19,2")).unwrap());
        assert_eq!(Stnd, lookup(ti("soft:19,5")).unwrap());
        assert_eq!(DblS, lookup(ti("soft:19,6")).unwrap());
        assert_eq!(Stnd, lookup(ti("soft:19,7")).unwrap());
        assert_eq!(Stnd, lookup(ti("soft:19,10")).unwrap());
    }
}

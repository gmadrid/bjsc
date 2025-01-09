use crate::strat::charts::ChartAction::{DBLH, DBLS, HIT_, STND};
use crate::strat::charts::{as_chart_column, ChartAction};
use crate::strat::tableindex::{TableIndex, TableType};

const SOFT_CHART: [[ChartAction; 10]; 8] = [
    /* 13 (A, 2) */
    [HIT_, HIT_, HIT_, DBLH, DBLH, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 14 (A, 3) */
    [HIT_, HIT_, HIT_, DBLH, DBLH, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 15 (A, 4) */
    [HIT_, HIT_, DBLH, DBLH, DBLH, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 16 (A, 5) */
    [HIT_, HIT_, DBLH, DBLH, DBLH, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 17 (A, 6) */
    [HIT_, DBLH, DBLH, DBLH, DBLH, HIT_, HIT_, HIT_, HIT_, HIT_],
    /* 18 (A, 7) */
    [DBLS, DBLS, DBLS, DBLS, DBLS, STND, STND, HIT_, HIT_, HIT_],
    /* 19 (A, 8) */
    [STND, STND, STND, STND, DBLS, STND, STND, STND, STND, STND],
    /* 20 (A, 9) */
    [STND, STND, STND, STND, STND, STND, STND, STND, STND, STND],
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

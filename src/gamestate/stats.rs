use crate::strat::TableIndex;
use crate::Action;
use crate::Action::Surrender;

#[derive(Debug, Default, Clone)]
pub struct Stats {
    question_count: u32,
    questions_wrong: u32,

    surrender_count: u32,
    surrenders_wrong: u32,

    split_count: u32,
    splits_wrong: u32,

    double_count: u32,
    doubles_wrong: u32,

    soft_count: u32,
    soft_wrong: u32,

    hard_count: u32,
    hard_wrong: u32,
}

impl Stats {
    pub fn count(&mut self, wrong: bool, action: Action, table_index: TableIndex) {
        self.question_count += 1;
        if wrong {
            self.questions_wrong += 1;
        }

        if action == Surrender {
            self.surrender_count += 1;
            if wrong {
                self.surrenders_wrong += 1;
            }
        }
    }
}

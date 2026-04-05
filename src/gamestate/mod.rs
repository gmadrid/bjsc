use crate::hand::Hand;
use crate::hand_builder::build_hand_for_index;
use crate::shoe::Shoe;
use crate::strat::{
    lookup_action, phrase_for_row, Action, ChartAction, RowIndex, TableIndex, TableType,
};
use crate::studymode::StudyMode;
use crate::table_index_keys::{indices_for_mode, keys_for_mode, table_index_to_key};
use crate::BjResult;
use rand::prelude::*;
use spaced_rep::Deck;

pub mod stats;
use stats::Stats;

const NUM_DECKS: usize = 6;

/// Result of checking a player's answer.
pub struct AnswerResult {
    pub correct: bool,
    pub correct_action: Option<Action>,
    pub player_action: Action,
    /// Error log entry for wrong answers.
    pub log_entry: Option<String>,
    /// The TableIndex of the question.
    pub table_index: Option<TableIndex>,
    /// String key for the table index (for answer logging).
    pub table_index_key: Option<String>,
}

impl AnswerResult {
    /// Format a user-facing status message.
    pub fn status_message(&self) -> String {
        if self.correct {
            format!("Correct: {}", self.player_action)
        } else {
            format!(
                "WRONG: {}",
                self.correct_action
                    .map(|a| a.to_string())
                    .unwrap_or_default()
            )
        }
    }

    /// Extract answer log data: (table_index_key, correct, player_action, correct_action).
    pub fn log_data(&self) -> Option<(String, bool, String, String)> {
        self.table_index_key.clone().map(|key| {
            (
                key,
                self.correct,
                self.player_action.to_string(),
                self.correct_action
                    .map(|a| a.to_string())
                    .unwrap_or_default(),
            )
        })
    }
}

#[derive(Debug)]
pub struct GameState {
    shoe: Shoe,
    player_hand: Hand,
    dealer_hand: Hand,

    study_mode: StudyMode,
    stats: Stats,
    deck: Deck,
}

impl GameState {
    pub fn new() -> Self {
        let mut shoe = Shoe::new(NUM_DECKS);
        shoe.shuffle();

        GameState {
            shoe,
            player_hand: Default::default(),
            dealer_hand: Default::default(),
            study_mode: StudyMode::default(),
            stats: Stats::default(),
            deck: Deck::new(),
        }
    }

    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    pub fn study_mode(&self) -> StudyMode {
        self.study_mode
    }

    pub fn set_study_mode(&mut self, mode: StudyMode) {
        self.study_mode = mode;
    }

    pub fn deck(&self) -> &Deck {
        &self.deck
    }

    pub fn deck_summary(&self) -> spaced_rep::DeckSummary {
        let keys = keys_for_mode(self.study_mode);
        self.deck.summary(&keys)
    }

    pub fn box_counts(&self) -> [u32; spaced_rep::NUM_BOXES as usize] {
        let keys = keys_for_mode(self.study_mode);
        self.deck.box_counts(&keys)
    }

    pub fn box_due_counts(&self) -> [u32; spaced_rep::NUM_BOXES as usize] {
        let keys = keys_for_mode(self.study_mode);
        self.deck.box_due_counts(&keys)
    }

    pub fn unseen_count(&self) -> u32 {
        let keys = keys_for_mode(self.study_mode);
        self.deck.unseen_count(&keys)
    }

    pub fn set_deck(&mut self, deck: Deck) {
        self.deck = deck;
    }

    pub fn chart_action(&self) -> BjResult<(ChartAction, Option<TableIndex>)> {
        lookup_action(&self.player_hand, &self.dealer_hand)
    }

    pub fn dealer_hand(&self) -> &Hand {
        &self.dealer_hand
    }

    pub fn player_hand(&self) -> &Hand {
        &self.player_hand
    }

    /// Check the player's answer and update all state (stats, spaced rep).
    pub fn check_answer(&mut self, action: Action) -> Option<AnswerResult> {
        let (chart_action, table_index) = self.chart_action().ok()?;
        let correct_action = chart_action.apply_rules()?;
        let correct = action == correct_action;

        // For splittable hands, override the table index to use the split chart
        // so that stats, spaced rep, and error messages all track the split decision
        let table_index = table_index.map(|ti| {
            if self.player_hand.splittable() && ti.table_type() != TableType::Split {
                let card_val = self
                    .player_hand
                    .first_card()
                    .map(|c| c.value())
                    .unwrap_or(0);
                let split_row = if card_val == 11 { 1 } else { card_val };
                if let Ok(split_ri) = RowIndex::new(TableType::Split, split_row) {
                    crate::strat::new_table_index(split_ri, ti.col_index())
                } else {
                    ti
                }
            } else {
                ti
            }
        });

        // Update stats
        if let Some(ref ti) = table_index {
            self.stats.count(!correct, correct_action, ti);

            // Update spaced rep
            let key = table_index_to_key(ti);
            self.deck.record(&key, correct);
        }

        let log_entry = if !correct {
            if let Some(ref ti) = table_index {
                Some(format!(
                    "{} (P: {}, D: {})",
                    phrase_for_row(ti.row),
                    self.player_hand,
                    self.dealer_hand
                ))
            } else {
                Some(format!(
                    "P: {}, D: {}, Correct: {}, Guess: {}",
                    self.player_hand, self.dealer_hand, correct_action, action
                ))
            }
        } else {
            None
        };

        let table_index_key = table_index.as_ref().map(table_index_to_key);

        Some(AnswerResult {
            correct,
            correct_action: Some(correct_action),
            player_action: action,
            log_entry,
            table_index,
            table_index_key,
        })
    }

    pub fn shuffle(&mut self) {
        self.shoe.shuffle();
    }

    /// Deal the next hand based on the current study mode.
    /// Returns false if the shoe is done (only relevant for All mode).
    pub fn deal_a_hand(&mut self) -> bool {
        match self.study_mode {
            StudyMode::All => self.deal_from_shoe(),
            StudyMode::Drill => self.deal_drill(),
            _ => self.deal_category(),
        }
    }

    /// Deal from the shoe (original behavior). Skips naturals (blackjack).
    fn deal_from_shoe(&mut self) -> bool {
        loop {
            if self.shoe.is_done() {
                return false;
            }

            if let (Some(p1), Some(d1), Some(p2)) =
                (self.shoe.deal(), self.shoe.deal(), self.shoe.deal())
            {
                self.player_hand = Hand::default();
                self.dealer_hand = Hand::default();
                self.player_hand.add_card(p1);
                self.player_hand.add_card(p2);
                self.dealer_hand.add_card(d1);

                // Skip naturals — no decision to make
                if self.player_hand.is_natural() {
                    continue;
                }
                return true;
            } else {
                return false;
            }
        }
    }

    /// Deal a constructed hand for a category study mode.
    fn deal_category(&mut self) -> bool {
        let indices = indices_for_mode(self.study_mode);
        if indices.is_empty() {
            return false;
        }
        let idx = &indices[thread_rng().gen_range(0..indices.len())];
        let (player, dealer) = build_hand_for_index(idx);
        self.player_hand = player;
        self.dealer_hand = dealer;
        true
    }

    /// Deal based on spaced repetition selection.
    fn deal_drill(&mut self) -> bool {
        let keys = keys_for_mode(StudyMode::Drill);
        if keys.is_empty() {
            return false;
        }
        let key = self.deck.next_item(&keys).unwrap_or(&keys[0]);
        if let Ok(idx) = key.parse::<TableIndex>() {
            let (player, dealer) = build_hand_for_index(&idx);
            self.player_hand = player;
            self.dealer_hand = dealer;
            true
        } else {
            false
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

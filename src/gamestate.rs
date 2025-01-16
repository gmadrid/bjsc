use crate::hand::Hand;
use crate::shoe::Shoe;
use crate::strat::{lookup_action, ChartAction, TableIndex};
use crate::BjResult;

const NUM_DECKS: usize = 6;

#[derive(Debug)]
pub struct GameState {
    shoe: Shoe,
    player_hand: Hand,
    dealer_hand: Hand,

    num_questions: usize,
    num_wrong: usize,
}

pub enum GameMode {
    Playing,
    Done,
}

impl GameState {
    pub fn new() -> Self {
        let mut shoe = Shoe::new(NUM_DECKS);
        shoe.shuffle();

        GameState {
            shoe,
            player_hand: Default::default(),
            dealer_hand: Default::default(),
            num_questions: 0,
            num_wrong: 0,
        }
    }

    pub fn num_questions_asked(&self) -> usize {
        self.num_questions
    }

    pub fn num_questions_wrong(&self) -> usize {
        self.num_wrong
    }

    pub fn answered_right(&mut self) {
        self.num_questions += 1;
    }

    pub fn answered_wrong(&mut self) {
        self.num_wrong += 1;
        self.num_questions += 1;
    }

    pub fn mode(&self) -> GameMode {
        if self.shoe.is_done() {
            GameMode::Done
        } else {
            GameMode::Playing
        }
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

    pub fn new_hands(&mut self) {
        self.dealer_hand = Default::default();
        self.player_hand = Default::default();
    }

    // Returns false if the shoe is done.
    pub fn deal_a_hand(&mut self) -> bool {
        if self.shoe.is_done() {
            return false;
        }

        let mut succeeded = false;
        if let Some(p1) = self.shoe.deal() {
            if let Some(d1) = self.shoe.deal() {
                if let Some(p2) = self.shoe.deal() {
                    succeeded = true;
                    self.new_hands();
                    self.player_hand.add_card(p1);
                    self.player_hand.add_card(p2);
                    self.dealer_hand.add_card(d1);
                }
            }
        }
        succeeded
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

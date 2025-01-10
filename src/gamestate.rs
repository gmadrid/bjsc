use crate::hand::Hand;
use crate::shoe::Shoe;

const NUM_DECKS: usize = 6;

#[derive(Debug)]
pub struct GameState {
    shoe: Shoe,
    player_hand: Hand,
    dealer_hand: Hand,
}

impl GameState {
    pub fn new() -> Self {
        let mut shoe = Shoe::new(NUM_DECKS);
        shoe.shuffle();

        GameState {
            shoe,
            player_hand: Default::default(),
            dealer_hand: Default::default(),
        }
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

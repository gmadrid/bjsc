use rand::prelude::*;

use crate::card::Card;

const CARDS_IN_A_DECK: usize = 52;
const PEN: usize = 26;

#[derive(Debug)]
pub struct Shoe {
    cards: Vec<Card>,
    next: usize,
    pen: usize,
}

impl Shoe {
    pub fn new(num_decks: usize) -> Shoe {
        let mut cards = Vec::with_capacity(CARDS_IN_A_DECK * num_decks);

        for _ in 0..num_decks {
            for c in 0..CARDS_IN_A_DECK {
                // unwrap: we know the indices are in range.
                cards.push((c as u8).try_into().unwrap());
            }
        }

        Shoe {
            cards,
            next: 0,
            pen: num_decks * CARDS_IN_A_DECK - PEN,
        }
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
        self.next = 0;
    }

    pub fn is_done(&self) -> bool {
        self.next >= self.pen
    }

    pub fn deal(&mut self) -> Option<Card> {
        if self.next >= self.cards.len() {
            return None;
        }
        let card = self.cards[self.next];
        self.next += 1;
        Some(card)
    }
}

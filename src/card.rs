use crate::BjError::{BadPipValue, BadSuitValue, ValueOutOfRange};
use crate::{BjError, BjResult};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Card {
    pub pip: Pip,
    pub suit: Suit,
}

impl Card {
    pub fn value(&self) -> u8 {
        self.pip.value()
    }
}

impl TryFrom<u8> for Card {
    type Error = BjError;

    fn try_from(value: u8) -> BjResult<Self> {
        let pip = (value % 13).try_into()?;
        let suit = (value / 13).try_into()?;
        Ok(Card { pip, suit })
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.pip, self.suit)
    }
}

impl FromStr for Card {
    type Err = BjError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let suit_str = &trimmed[trimmed.len() - 1..];
        let pip_str = &trimmed[0..trimmed.len() - 1].trim();

        let suit = suit_str.parse()?;
        let pip = pip_str.parse()?;
        Ok(Card { pip, suit })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Pip {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Pip {
    pub fn value(&self) -> u8 {
        match self {
            Pip::Ace => 11,
            Pip::Two => 2,
            Pip::Three => 3,
            Pip::Four => 4,
            Pip::Five => 5,
            Pip::Six => 6,
            Pip::Seven => 7,
            Pip::Eight => 8,
            Pip::Nine => 9,
            Pip::Ten | Pip::Jack | Pip::Queen | Pip::King => 10,
        }
    }
}

impl TryFrom<u8> for Pip {
    type Error = BjError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Pip::Ace),
            1 => Ok(Pip::Two),
            2 => Ok(Pip::Three),
            3 => Ok(Pip::Four),
            4 => Ok(Pip::Five),
            5 => Ok(Pip::Six),
            6 => Ok(Pip::Seven),
            7 => Ok(Pip::Eight),
            8 => Ok(Pip::Nine),
            9 => Ok(Pip::Ten),
            10 => Ok(Pip::Jack),
            11 => Ok(Pip::Queen),
            12 => Ok(Pip::King),
            _ => Err(ValueOutOfRange(value, 0, 12)),
        }
    }
}

impl Display for Pip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Pip::Ace => "A",
            Pip::Two => "2",
            Pip::Three => "3",
            Pip::Four => "4",
            Pip::Five => "5",
            Pip::Six => "6",
            Pip::Seven => "7",
            Pip::Eight => "8",
            Pip::Nine => "9",
            Pip::Ten => "T",
            Pip::Jack => "J",
            Pip::Queen => "Q",
            Pip::King => "K",
        };
        write!(f, "{}", str)
    }
}

impl FromStr for Pip {
    type Err = BjError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" | "A" => Ok(Pip::Ace),
            "2" => Ok(Pip::Two),
            "3" => Ok(Pip::Three),
            "4" => Ok(Pip::Four),
            "5" => Ok(Pip::Five),
            "6" => Ok(Pip::Six),
            "7" => Ok(Pip::Seven),
            "8" => Ok(Pip::Eight),
            "9" => Ok(Pip::Nine),
            "10" | "T" => Ok(Pip::Ten),
            "J" => Ok(Pip::Jack),
            "Q" => Ok(Pip::Queen),
            "K" => Ok(Pip::King),
            _ => Err(BadPipValue(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl TryFrom<u8> for Suit {
    type Error = BjError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Suit::Spades),
            1 => Ok(Suit::Hearts),
            2 => Ok(Suit::Diamonds),
            3 => Ok(Suit::Clubs),
            _ => Err(ValueOutOfRange(value, 0, 3)),
        }
    }
}

impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Suit::Spades => "S",
            Suit::Hearts => "H",
            Suit::Diamonds => "D",
            Suit::Clubs => "C",
        };
        write!(f, "{}", str)
    }
}

impl FromStr for Suit {
    type Err = BjError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "S" | "s" => Ok(Suit::Spades),
            "H" | "h" => Ok(Suit::Hearts),
            "D" | "d" => Ok(Suit::Diamonds),
            "C" | "c" => Ok(Suit::Clubs),
            _ => Err(BadSuitValue(s.to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn card(pip: Pip, suit: Suit) -> Card {
        Card { pip, suit }
    }

    #[test]
    fn test_parse_aces() {
        assert_eq!(card(Pip::Ace, Suit::Spades), "AS".parse().unwrap());
        assert_eq!(card(Pip::Ace, Suit::Hearts), "1H".parse().unwrap());
    }

    #[test]
    fn test_parse_tens() {
        assert_eq!(card(Pip::Ten, Suit::Clubs), "TC".parse().unwrap());
        assert_eq!(card(Pip::Ten, Suit::Diamonds), "10D".parse().unwrap());
    }

    #[test]
    fn test_all_suits() {
        assert_eq!(card(Pip::Five, Suit::Spades), "5S".parse().unwrap());
        assert_eq!(card(Pip::Five, Suit::Hearts), "5H".parse().unwrap());
        assert_eq!(card(Pip::Five, Suit::Diamonds), "5D".parse().unwrap());
        assert_eq!(card(Pip::Five, Suit::Clubs), "5C".parse().unwrap());
    }

    #[test]
    fn test_other_pips() {
        assert_eq!(card(Pip::Two, Suit::Spades), "2S".parse().unwrap());
        assert_eq!(card(Pip::Three, Suit::Spades), "3S".parse().unwrap());
        assert_eq!(card(Pip::Four, Suit::Spades), "4S".parse().unwrap());
        assert_eq!(card(Pip::Five, Suit::Spades), "5S".parse().unwrap());
        assert_eq!(card(Pip::Six, Suit::Spades), "6S".parse().unwrap());
        assert_eq!(card(Pip::Seven, Suit::Spades), "7S".parse().unwrap());
        assert_eq!(card(Pip::Eight, Suit::Spades), "8S".parse().unwrap());
        assert_eq!(card(Pip::Nine, Suit::Spades), "9S".parse().unwrap());
    }

    #[test]
    fn test_try_from_card() {
        assert_eq!(card(Pip::Ace, Suit::Spades), 0.try_into().unwrap());
        assert_eq!(card(Pip::King, Suit::Hearts), 25.try_into().unwrap());
        assert_eq!(card(Pip::Five, Suit::Diamonds), 30.try_into().unwrap());
        assert_eq!(card(Pip::King, Suit::Clubs), 51.try_into().unwrap());
    }
}

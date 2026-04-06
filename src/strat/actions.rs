use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Action {
    Hit,
    Stand,
    Split,
    Double,
    Surrender,
}

impl Action {
    pub fn from_key(key: char) -> Option<Self> {
        match key {
            'h' | 'a' => Some(Action::Hit),
            's' => Some(Action::Stand),
            'p' => Some(Action::Split),
            'd' => Some(Action::Double),
            'r' => Some(Action::Surrender),
            _ => None,
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Action::Hit => "Hit",
            Action::Stand => "Stand",
            Action::Split => "Split",
            Action::Double => "Double",
            Action::Surrender => "Surrender",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- from_key(): valid keys ---

    #[test]
    fn from_key_h_returns_hit() {
        assert_eq!(Some(Action::Hit), Action::from_key('h'));
    }

    #[test]
    fn from_key_a_returns_hit() {
        // 'a' is an alternate hit key
        assert_eq!(Some(Action::Hit), Action::from_key('a'));
    }

    #[test]
    fn from_key_s_returns_stand() {
        assert_eq!(Some(Action::Stand), Action::from_key('s'));
    }

    #[test]
    fn from_key_p_returns_split() {
        assert_eq!(Some(Action::Split), Action::from_key('p'));
    }

    #[test]
    fn from_key_d_returns_double() {
        assert_eq!(Some(Action::Double), Action::from_key('d'));
    }

    #[test]
    fn from_key_r_returns_surrender() {
        assert_eq!(Some(Action::Surrender), Action::from_key('r'));
    }

    // --- from_key(): invalid keys ---

    #[test]
    fn from_key_uppercase_h_returns_none() {
        assert_eq!(None, Action::from_key('H'));
    }

    #[test]
    fn from_key_uppercase_s_returns_none() {
        assert_eq!(None, Action::from_key('S'));
    }

    #[test]
    fn from_key_space_returns_none() {
        assert_eq!(None, Action::from_key(' '));
    }

    #[test]
    fn from_key_digit_returns_none() {
        assert_eq!(None, Action::from_key('1'));
    }

    #[test]
    fn from_key_unrelated_letter_returns_none() {
        assert_eq!(None, Action::from_key('x'));
        assert_eq!(None, Action::from_key('q'));
        assert_eq!(None, Action::from_key('z'));
    }

    // --- Display ---

    #[test]
    fn display_hit() {
        assert_eq!("Hit", Action::Hit.to_string());
    }

    #[test]
    fn display_stand() {
        assert_eq!("Stand", Action::Stand.to_string());
    }

    #[test]
    fn display_split() {
        assert_eq!("Split", Action::Split.to_string());
    }

    #[test]
    fn display_double() {
        assert_eq!("Double", Action::Double.to_string());
    }

    #[test]
    fn display_surrender() {
        assert_eq!("Surrender", Action::Surrender.to_string());
    }
}

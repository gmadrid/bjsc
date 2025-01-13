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
            'h' => Some(Action::Hit),
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

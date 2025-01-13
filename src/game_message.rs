use cursive::style::BaseColor::{Red, White};
use cursive::style::{ColorStyle, Style};
use cursive::utils::markup::StyledString;

#[derive(Debug, Default, Clone)]
pub enum GameMessage {
    Correct(String),
    Wrong(String),

    #[default]
    None,
}

impl GameMessage {
    pub fn correct(m: impl Into<String>) -> Self {
        Self::Correct(m.into())
    }

    pub fn wrong(m: impl Into<String>) -> Self {
        Self::Wrong(m.into())
    }
}

impl From<GameMessage> for StyledString {
    fn from(value: GameMessage) -> Self {
        match value {
            GameMessage::Correct(msg) => StyledString::plain(msg),
            GameMessage::Wrong(msg) => {
                let style = Style {
                    effects: Default::default(),
                    color: ColorStyle::new(White, Red.dark()),
                };
                StyledString::styled(format!(" {} ", msg), style)
            }
            GameMessage::None => StyledString::plain(""),
        }
    }
}

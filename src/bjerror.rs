use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum BjError {
    #[error("An expected Dealer card was missing.")]
    MissingDealerCard,

    #[error("Col index couldn't be parsed, '{0}'.")]
    BadColIndex(String),
    #[error("Row index couldn't be parsed, '{0}'.")]
    BadRowIndex(String),
    #[error("Table index couldn't be parsed, '{0}'.")]
    BadTableIndex(String),

    #[error("Unknown table type: '{0}'.")]
    UnknownTableType(String),

    #[error("Value, {0}, is out of range, [{1}, {2}].")]
    ValueOutOfRange(u8, u8, u8),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

pub type BjResult<T> = Result<T, BjError>;

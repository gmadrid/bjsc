use thiserror::Error;

#[derive(Error, Debug)]
pub enum BjError {
    #[error("An expected Dealer card was missing.")]
    MissingDealerCard,

    #[error("Value, {0}, is out of range, [{1}, {2}].")]
    ValueOutOfRange(u8, u8, u8),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

pub type BjResult<T> = Result<T, BjError>;

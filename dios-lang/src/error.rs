use comp_gen::ruler::egg;
use thiserror::Error;

pub type Res<T> = anyhow::Result<T>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json parsing error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("recexpr parse error: {0}")]
    RecExprError(#[from] egg::RecExprParseError<egg::FromOpError>),
}

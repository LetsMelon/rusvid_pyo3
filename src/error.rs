use std::io;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyValueError};
use pyo3::PyErr;
use thiserror::Error;

pub type CustomResult<T> = Result<T, CustomError>;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("{0:?}")]
    FileNotFound(#[from] io::Error),

    #[error("{0:?}")]
    Parsing(String),

    #[error("{0:?}")]
    Drawing(#[from] rusvid_core::plane::PlaneError),

    #[error("{0:?}")]
    ParseNumberValue(#[from] std::num::ParseIntError),
}

impl From<CustomError> for PyErr {
    fn from(value: CustomError) -> Self {
        match value {
            CustomError::FileNotFound(err) => PyFileNotFoundError::new_err(format!("{err}")),
            CustomError::Parsing(err) => PyException::new_err(format!("{:?}", err)),
            CustomError::Drawing(err) => PyException::new_err(format!("{:?}", err)),
            CustomError::ParseNumberValue(err) => PyValueError::new_err(format!("{:?}", err)),
        }
    }
}

impl From<nom::error::Error<&str>> for CustomError {
    fn from(value: nom::error::Error<&str>) -> Self {
        CustomError::Parsing(format!("{}", value))
    }
}

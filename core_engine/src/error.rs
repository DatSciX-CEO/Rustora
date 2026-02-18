use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustoraError {
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),

    #[error("DuckDB error: {0}")]
    DuckDb(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No active dataframe loaded")]
    NoActiveDataFrame,

    #[error("Table not found: {0}")]
    TableNotFound(String),

    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("Invalid edit: {0}")]
    InvalidEdit(String),

    #[error("No project open")]
    NoProjectOpen,

    #[error("Session error: {0}")]
    Session(String),
}

pub type Result<T> = std::result::Result<T, RustoraError>;

//! Rustora core engine — zero-copy data analytics backed by DuckDB and Polars.
//!
//! # Architecture
//! - [`DuckStorage`] provides persistent table storage via an embedded DuckDB database.
//! - [`RustoraSession`] is the primary API, exposing file import, SQL execution,
//!   transformations, and Arrow IPC serialization.
//! - All tabular data leaves this crate as **Arrow IPC bytes** — no JSON is produced.
//!
//! # Quick start
//! ```no_run
//! use core_engine::RustoraSession;
//!
//! let mut session = RustoraSession::new();
//! session.new_project("analysis.duckdb").unwrap();
//! let name = session.import_file("data.csv", None).unwrap();
//! let ipc_bytes = session.get_preview_ipc(&name, 100).unwrap();
//! ```

pub mod error;
pub mod filter;
pub mod session;
pub mod storage;

pub use error::{Result, RustoraError};
pub use filter::{FilterCondition, FilterLogic, FilterOperator, FilterSpec};
pub use session::RustoraSession;
pub use storage::DuckStorage;

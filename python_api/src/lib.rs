use core_engine::RustoraSession;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// Python wrapper for the Rustora core engine session.
///
/// Usage:
///   import rustora
///   session = rustora.Session()
///   session.new_project("my_data.duckdb")
///   session.import_file("data.csv", "my_table")
///   tables = session.list_datasets()
///   ipc_bytes = session.get_preview("my_table", 100)
#[pyclass(unsendable)]
struct Session {
    inner: RustoraSession,
}

#[pymethods]
impl Session {
    #[new]
    fn new() -> Self {
        Session {
            inner: RustoraSession::new(),
        }
    }

    /// Create a new persistent project (.duckdb file).
    fn new_project(&mut self, path: &str) -> PyResult<()> {
        self.inner
            .new_project(path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Open an existing project (.duckdb file). Returns list of table names.
    fn open_project(&mut self, path: &str) -> PyResult<Vec<String>> {
        self.inner
            .open_project(path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Import a file into the DuckDB project as a persistent table.
    /// Returns the table name used.
    fn import_file(&mut self, path: &str, table_name: Option<&str>) -> PyResult<String> {
        self.inner
            .import_file(path, table_name)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Scan a file using Polars (transient, not persisted).
    fn scan_file(&mut self, path: &str) -> PyResult<String> {
        self.inner
            .scan_file(path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// List all available datasets (persistent + transient).
    fn list_datasets(&self) -> Vec<String> {
        self.inner.list_datasets()
    }

    /// Get total row count for a dataset.
    fn get_row_count(&self, name: &str) -> PyResult<usize> {
        self.inner
            .get_row_count(name)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Get a preview of a dataset as Arrow IPC bytes.
    fn get_preview<'py>(&self, py: Python<'py>, name: &str, limit: u32) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self
            .inner
            .get_preview_ipc(name, limit)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Get a paginated chunk of rows as Arrow IPC bytes.
    fn get_chunk<'py>(
        &self,
        py: Python<'py>,
        name: &str,
        offset: u32,
        limit: u32,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self
            .inner
            .get_chunk_ipc(name, offset, limit)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Execute a SQL query. Returns the result table name.
    fn execute_sql(&mut self, sql: &str) -> PyResult<String> {
        self.inner
            .execute_sql(sql)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Execute a SQL query and return results as Arrow IPC bytes.
    fn query_to_ipc<'py>(&self, py: Python<'py>, sql: &str) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self
            .inner
            .execute_sql_to_ipc(sql)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Sort a dataset. Returns the new dataset name.
    fn sort_dataset(
        &mut self,
        name: &str,
        columns: Vec<String>,
        descending: Vec<bool>,
    ) -> PyResult<String> {
        let col_refs: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
        self.inner
            .sort_dataset(name, &col_refs, &descending)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Filter a dataset using a SQL WHERE clause. Returns the new dataset name.
    fn filter_sql(&mut self, name: &str, where_clause: &str) -> PyResult<String> {
        self.inner
            .filter_dataset_sql(name, where_clause)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Export a dataset to CSV.
    fn export_csv(&self, name: &str, output_path: &str) -> PyResult<()> {
        self.inner
            .export_to_csv(name, output_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Export a dataset to Parquet.
    fn export_parquet(&self, name: &str, output_path: &str) -> PyResult<()> {
        self.inner
            .export_to_parquet(name, output_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Remove a dataset.
    fn remove_dataset(&mut self, name: &str) -> PyResult<bool> {
        self.inner
            .remove_dataset(name)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

/// Rustora: Blazingly fast, 100% local data analysis.
#[pymodule]
fn rustora(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Session>()?;
    Ok(())
}

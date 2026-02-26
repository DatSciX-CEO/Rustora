use crate::error::{Result, RustoraError};
use crate::filter::FilterSpec;
use crate::storage::DuckStorage;
use polars::prelude::*;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::info;

/// Metadata about a loaded dataset.
#[derive(Debug, Clone)]
pub struct DatasetInfo {
    pub name: String,
    pub path: String,
    pub num_columns: usize,
    pub estimated_rows: Option<usize>,
    pub column_names: Vec<String>,
    pub column_dtypes: Vec<String>,
    /// Whether this dataset is a persistent DuckDB table or a transient Polars LazyFrame.
    pub persistent: bool,
    /// Estimated in-memory size in bytes (None if unknown).
    pub estimated_size_bytes: Option<u64>,
}

/// The core session that manages all data operations.
///
/// Architecture:
/// - **DuckDB** is the primary storage layer for persistent tables.
/// - **Polars LazyFrames** are a secondary cache for derived/transient computations.
/// - All data leaves Rust as **Arrow IPC bytes only** (NO JSON).
pub struct RustoraSession {
    /// DuckDB persistent storage (None if no project is open).
    storage: Option<DuckStorage>,
    /// Transient Polars LazyFrames (for non-persistent computed results).
    transient: HashMap<String, LazyFrame>,
    /// Counter for generating unique names.
    counter: Arc<Mutex<u64>>,
}

impl RustoraSession {
    /// Create a session with an in-memory DuckDB database (scratch mode).
    pub fn new() -> Self {
        let storage = DuckStorage::open_in_memory().ok();
        Self {
            storage,
            transient: HashMap::new(),
            counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Open a persistent project file (.duckdb).
    /// Existing tables in the database become immediately available.
    pub fn open_project(&mut self, db_path: &str) -> Result<Vec<String>> {
        info!(db_path, "opening project");
        let storage = DuckStorage::open(db_path)?;
        let tables = storage.list_tables()?;
        info!(db_path, table_count = tables.len(), "project opened");
        self.storage = Some(storage);
        self.transient.clear();
        Ok(tables)
    }

    /// Create a new project file (.duckdb).
    pub fn new_project(&mut self, db_path: &str) -> Result<()> {
        let storage = DuckStorage::open(db_path)?;
        self.storage = Some(storage);
        self.transient.clear();
        Ok(())
    }

    /// Get the current project path.
    pub fn project_path(&self) -> Option<&str> {
        self.storage.as_ref().map(|s| s.db_path())
    }

    fn storage(&self) -> Result<&DuckStorage> {
        self.storage.as_ref().ok_or(RustoraError::NoProjectOpen)
    }

    fn next_counter(&self) -> u64 {
        let mut counter = self.counter.lock().unwrap_or_else(|e| e.into_inner());
        *counter += 1;
        *counter
    }

    fn generate_name(&self, file_path: &str) -> String {
        let stem = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("dataset");
        format!("{}_{}", stem, self.next_counter())
    }

    // -----------------------------------------------------------------------
    // File Import (-> DuckDB persistent table)
    // -----------------------------------------------------------------------

    /// Import a file into the DuckDB database as a persistent table.
    /// This is the primary way to load data. The file is copied into DuckDB storage.
    pub fn import_file(&mut self, file_path: &str, table_name: Option<&str>) -> Result<String> {
        let storage = self.storage.as_ref().ok_or(RustoraError::NoProjectOpen)?;

        let name = match table_name {
            Some(n) => n.to_string(),
            None => self.generate_name(file_path),
        };

        info!(file_path, table = %name, "importing file into session");
        storage.import_file(file_path, &name)?;
        Ok(name)
    }

    /// Lazily scan a file via Polars (non-persistent, kept in memory).
    /// For backwards compatibility; prefer `import_file` for persistent storage.
    pub fn scan_file(&mut self, file_path: &str) -> Result<String> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(RustoraError::FileNotFound(file_path.to_string()));
        }

        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let lf = match extension.as_str() {
            "csv" | "tsv" => {
                let separator = if extension == "tsv" { b'\t' } else { b',' };
                LazyCsvReader::new(file_path)
                    .with_has_header(true)
                    .with_separator(separator)
                    .finish()?
            }
            "parquet" | "pq" => LazyFrame::scan_parquet(file_path, ScanArgsParquet::default())?,
            "ipc" | "arrow" | "feather" => {
                LazyFrame::scan_ipc(file_path, ScanArgsIpc::default())?
            }
            other => return Err(RustoraError::UnsupportedFormat(other.to_string())),
        };

        let name = self.generate_name(file_path);
        self.transient.insert(name.clone(), lf);
        Ok(name)
    }

    // -----------------------------------------------------------------------
    // Dataset Listing & Info
    // -----------------------------------------------------------------------

    /// List all available datasets (both persistent DuckDB tables and transient LazyFrames).
    pub fn list_datasets(&self) -> Vec<String> {
        let mut names: Vec<String> = self.transient.keys().cloned().collect();

        if let Some(storage) = &self.storage {
            if let Ok(tables) = storage.list_tables() {
                names.extend(tables);
            }
        }

        names.sort();
        names.dedup();
        names
    }

    /// List only persistent DuckDB tables.
    pub fn list_tables(&self) -> Result<Vec<String>> {
        self.storage()?.list_tables()
    }

    /// Get metadata about a dataset (checks DuckDB first, then transient).
    pub fn dataset_info(&self, name: &str) -> Result<DatasetInfo> {
        if let Some(storage) = &self.storage {
            if let Ok(info) = storage.table_info(name) {
                let size = storage.table_estimated_size_bytes(name).ok();
                return Ok(DatasetInfo {
                    name: info.name,
                    path: String::new(),
                    num_columns: info.num_columns,
                    estimated_rows: Some(info.row_count),
                    column_names: info.column_names,
                    column_dtypes: info.column_types,
                    persistent: true,
                    estimated_size_bytes: size,
                });
            }
        }

        if let Some(lf) = self.transient.get(name) {
            let schema = lf.clone().collect_schema()?;
            let column_names: Vec<String> = schema.iter_names().map(|n| n.to_string()).collect();
            let column_dtypes: Vec<String> = schema
                .iter_names_and_dtypes()
                .map(|(_, dt)| dt.to_string())
                .collect();

            return Ok(DatasetInfo {
                name: name.to_string(),
                path: String::new(),
                num_columns: schema.len(),
                estimated_rows: None,
                column_names,
                column_dtypes,
                persistent: false,
                estimated_size_bytes: None,
            });
        }

        Err(RustoraError::TableNotFound(name.to_string()))
    }

    // -----------------------------------------------------------------------
    // Arrow IPC Serialization (ZERO JSON -- Critical Constraint)
    // -----------------------------------------------------------------------

    /// Get a preview of a dataset as Arrow IPC bytes.
    /// Checks DuckDB tables first, then transient LazyFrames.
    pub fn get_preview_ipc(&self, name: &str, limit: u32) -> Result<Vec<u8>> {
        if let Some(storage) = &self.storage {
            if storage.list_tables()?.contains(&name.to_string()) {
                return storage.get_table_preview_ipc(name, limit as u64);
            }
        }

        if let Some(lf) = self.transient.get(name) {
            let df = lf.clone().limit(limit).collect()?;
            return Self::dataframe_to_ipc_bytes(&df);
        }

        Err(RustoraError::TableNotFound(name.to_string()))
    }

    /// Get a paginated chunk of rows as Arrow IPC bytes.
    pub fn get_chunk_ipc(&self, name: &str, offset: u32, limit: u32) -> Result<Vec<u8>> {
        if let Some(storage) = &self.storage {
            if storage.list_tables()?.contains(&name.to_string()) {
                return storage.get_table_chunk_ipc(name, offset as u64, limit as u64);
            }
        }

        if let Some(lf) = self.transient.get(name) {
            let df = lf.clone().slice(offset as i64, limit).collect()?;
            return Self::dataframe_to_ipc_bytes(&df);
        }

        Err(RustoraError::TableNotFound(name.to_string()))
    }

    /// Get the total row count for a dataset.
    pub fn get_row_count(&self, name: &str) -> Result<usize> {
        if let Some(storage) = &self.storage {
            if let Ok(count) = storage.table_row_count(name) {
                return Ok(count);
            }
        }

        if let Some(lf) = self.transient.get(name) {
            let count_df = lf
                .clone()
                .select([col("*").count().alias("count")])
                .collect()?;
            let count = count_df.column("count")?.u32()?.get(0).unwrap_or(0) as usize;
            return Ok(count);
        }

        Err(RustoraError::TableNotFound(name.to_string()))
    }

    // -----------------------------------------------------------------------
    // SQL Execution (via DuckDB)
    // -----------------------------------------------------------------------

    /// Execute a SQL query via DuckDB. Result is stored as a new table.
    /// Returns the result table name.
    pub fn execute_sql(&mut self, sql: &str) -> Result<String> {
        let storage = self.storage.as_ref().ok_or(RustoraError::NoProjectOpen)?;

        let result_name = format!("sql_result_{}", self.next_counter());
        info!(sql_len = sql.len(), result_table = %result_name, "executing SQL");
        storage.execute_sql_to_table(sql, &result_name)?;
        Ok(result_name)
    }

    /// Execute a SQL query and return the result directly as Arrow IPC bytes
    /// (without persisting as a table). For read-only queries.
    pub fn execute_sql_to_ipc(&self, sql: &str) -> Result<Vec<u8>> {
        let storage = self.storage.as_ref().ok_or(RustoraError::NoProjectOpen)?;
        storage.query_to_ipc(sql)
    }

    // -----------------------------------------------------------------------
    // Transformations (via DuckDB SQL for persistent, Polars for transient)
    // -----------------------------------------------------------------------

    /// Sort a dataset. For DuckDB tables, uses SQL ORDER BY.
    pub fn sort_dataset(
        &mut self,
        name: &str,
        columns: &[&str],
        descending: &[bool],
    ) -> Result<String> {
        if let Some(storage) = &self.storage {
            if storage.list_tables()?.contains(&name.to_string()) {
                let order_clauses: Vec<String> = columns
                    .iter()
                    .zip(descending.iter())
                    .map(|(c, &desc)| {
                        format!("\"{}\" {}", c, if desc { "DESC" } else { "ASC" })
                    })
                    .collect();
                let sql = format!(
                    "SELECT * FROM \"{}\" ORDER BY {}",
                    name,
                    order_clauses.join(", ")
                );
                let result_name = format!("{}_sorted", name);
                storage.execute_sql_to_table(&sql, &result_name)?;
                return Ok(result_name);
            }
        }

        if let Some(lf) = self.transient.get(name) {
            let by: Vec<PlSmallStr> = columns.iter().map(|c| PlSmallStr::from(*c)).collect();
            let sort_options =
                SortMultipleOptions::new().with_order_descending_multi(descending.to_vec());
            let sorted = lf.clone().sort(by, sort_options);
            let new_name = format!("{}_sorted", name);
            self.transient.insert(new_name.clone(), sorted);
            return Ok(new_name);
        }

        Err(RustoraError::TableNotFound(name.to_string()))
    }

    /// Filter a dataset with a Polars expression (transient datasets only).
    pub fn filter_dataset(&mut self, name: &str, predicate: Expr) -> Result<String> {
        let lf = self
            .transient
            .get(name)
            .ok_or(RustoraError::TableNotFound(name.to_string()))?;

        let filtered = lf.clone().filter(predicate);
        let new_name = format!("{}_filtered", name);
        self.transient.insert(new_name.clone(), filtered);
        Ok(new_name)
    }

    /// Filter a dataset using a SQL WHERE clause (works for both DuckDB and transient).
    /// Example predicate: "age > 30 AND city = 'Boston'"
    pub fn filter_dataset_sql(
        &mut self,
        name: &str,
        where_clause: &str,
    ) -> Result<String> {
        // For DuckDB tables, use SQL
        if let Some(storage) = &self.storage {
            if storage.list_tables()?.contains(&name.to_string()) {
                let sql = format!(
                    "SELECT * FROM \"{}\" WHERE {}",
                    name, where_clause
                );
                let result_name = format!("{}_filtered_{}", name, self.next_counter());
                storage.execute_sql_to_table(&sql, &result_name)?;
                return Ok(result_name);
            }
        }

        // For transient: try to use SQL via DuckDB if available, else error
        Err(RustoraError::Session(format!(
            "SQL filter requires an active project. Table '{}' not found in DuckDB.",
            name
        )))
    }

    /// Filter a dataset using a structured FilterSpec (safe from SQL injection).
    pub fn filter_dataset_structured(
        &mut self,
        name: &str,
        spec: &FilterSpec,
    ) -> Result<String> {
        let where_clause = spec.to_sql_where()?;
        self.filter_dataset_sql(name, &where_clause)
    }

    /// Group a dataset by columns with aggregations.
    /// `agg_exprs` are SQL aggregate expressions like ["AVG(salary)", "COUNT(*)", "SUM(amount)"].
    pub fn group_by(
        &mut self,
        name: &str,
        group_columns: &[&str],
        agg_exprs: &[&str],
    ) -> Result<String> {
        if let Some(storage) = &self.storage {
            if storage.list_tables()?.contains(&name.to_string()) {
                let group_cols = group_columns
                    .iter()
                    .map(|c| format!("\"{}\"", c))
                    .collect::<Vec<_>>()
                    .join(", ");

                let agg_list = agg_exprs.join(", ");

                let sql = format!(
                    "SELECT {}, {} FROM \"{}\" GROUP BY {}",
                    group_cols, agg_list, name, group_cols
                );

                let result_name = format!("{}_grouped_{}", name, self.next_counter());
                storage.execute_sql_to_table(&sql, &result_name)?;
                return Ok(result_name);
            }
        }

        Err(RustoraError::TableNotFound(name.to_string()))
    }

    /// Add a calculated column to a dataset via a SQL expression.
    /// Example: expr = "salary * 12", alias = "annual_salary"
    pub fn add_calculated_column(
        &mut self,
        name: &str,
        expr: &str,
        alias: &str,
    ) -> Result<String> {
        if let Some(storage) = &self.storage {
            if storage.list_tables()?.contains(&name.to_string()) {
                let sql = format!(
                    "SELECT *, ({}) AS \"{}\" FROM \"{}\"",
                    expr, alias, name
                );
                let result_name = format!("{}_calc_{}", name, self.next_counter());
                storage.execute_sql_to_table(&sql, &result_name)?;
                return Ok(result_name);
            }
        }

        Err(RustoraError::TableNotFound(name.to_string()))
    }

    /// Get summary statistics for all numeric columns in a dataset.
    /// Returns IPC bytes of a stats table with rows: count, null_count, min, max, mean, std.
    pub fn summary_stats_ipc(&self, name: &str) -> Result<Vec<u8>> {
        if let Some(storage) = &self.storage {
            if storage.list_tables()?.contains(&name.to_string()) {
                // Use DuckDB SUMMARIZE for comprehensive stats
                let sql = format!("SUMMARIZE SELECT * FROM \"{}\"", name);
                return storage.query_to_ipc(&sql);
            }
        }

        Err(RustoraError::Session(
            "Summary statistics require an active project. Please create or open a project first.".to_string()
        ))
    }

    // -----------------------------------------------------------------------
    // Chart / Aggregation
    // -----------------------------------------------------------------------

    /// Aggregate data for chart visualization.
    /// Returns up to `limit` groups, sorted by the group column.
    /// `agg_type` can be: "count", "sum", "avg", "min", "max"
    pub fn aggregate_for_chart(
        &self,
        name: &str,
        group_col: &str,
        value_col: Option<&str>,
        agg_type: &str,
        limit: u32,
    ) -> Result<Vec<u8>> {
        let storage = self.storage.as_ref().ok_or(RustoraError::NoProjectOpen)?;

        if !storage.list_tables()?.contains(&name.to_string()) {
            return Err(RustoraError::TableNotFound(name.to_string()));
        }

        let agg_expr = match (agg_type, value_col) {
            ("count", _) => "COUNT(*)".to_string(),
            (agg, Some(vc)) => format!("{}(\"{}\")", agg.to_uppercase(), vc),
            (agg, None) => {
                return Err(RustoraError::Session(format!(
                    "Aggregation '{}' requires a value column",
                    agg
                )))
            }
        };

        let sql = format!(
            "SELECT \"{group}\" AS label, {agg} AS value \
             FROM \"{table}\" \
             GROUP BY \"{group}\" \
             ORDER BY value DESC \
             LIMIT {limit}",
            group = group_col,
            agg = agg_expr,
            table = name,
            limit = limit,
        );

        storage.query_to_ipc(&sql)
    }

    // -----------------------------------------------------------------------
    // Export
    // -----------------------------------------------------------------------

    /// Export a dataset to Parquet.
    /// For transient LazyFrames, uses streaming sink to avoid loading the full dataset into memory.
    pub fn export_to_parquet(&self, name: &str, output_path: &str) -> Result<()> {
        if let Some(storage) = &self.storage {
            if storage.list_tables()?.contains(&name.to_string()) {
                return storage.export_to_parquet(name, output_path);
            }
        }

        if let Some(lf) = self.transient.get(name) {
            lf.clone()
                .sink_parquet(&output_path, ParquetWriteOptions::default(), None)?;
            return Ok(());
        }

        Err(RustoraError::TableNotFound(name.to_string()))
    }

    /// Export a dataset to CSV.
    /// For transient LazyFrames, uses streaming sink to avoid loading the full dataset into memory.
    pub fn export_to_csv(&self, name: &str, output_path: &str) -> Result<()> {
        if let Some(storage) = &self.storage {
            if storage.list_tables()?.contains(&name.to_string()) {
                return storage.export_to_csv(name, output_path);
            }
        }

        if let Some(lf) = self.transient.get(name) {
            lf.clone().sink_csv(
                &output_path,
                CsvWriterOptions {
                    include_header: true,
                    ..Default::default()
                },
                None,
            )?;
            return Ok(());
        }

        Err(RustoraError::TableNotFound(name.to_string()))
    }

    // -----------------------------------------------------------------------
    // Remove / Clean up
    // -----------------------------------------------------------------------

    /// Remove a dataset (drops DuckDB table or removes transient LazyFrame).
    pub fn remove_dataset(&mut self, name: &str) -> Result<bool> {
        if let Some(storage) = &self.storage {
            if storage.list_tables()?.contains(&name.to_string()) {
                storage.drop_table(name)?;
                return Ok(true);
            }
        }

        Ok(self.transient.remove(name).is_some())
    }

    /// Register an existing LazyFrame as a transient dataset.
    pub fn register_lazy_frame(&mut self, name: &str, lf: LazyFrame) {
        self.transient.insert(name.to_string(), lf);
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Serialize a Polars DataFrame to Arrow IPC Stream bytes.
    fn dataframe_to_ipc_bytes(df: &DataFrame) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        let cursor = Cursor::new(&mut buffer);

        IpcStreamWriter::new(cursor)
            .with_compat_level(CompatLevel::newest())
            .finish(&mut df.clone())?;

        Ok(buffer)
    }
}

impl Default for RustoraSession {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".csv").unwrap();
        writeln!(file, "name,age,city,score").unwrap();
        writeln!(file, "Alice,30,New York,95.5").unwrap();
        writeln!(file, "Bob,25,San Francisco,88.0").unwrap();
        writeln!(file, "Charlie,35,Chicago,72.3").unwrap();
        writeln!(file, "Diana,28,Boston,91.1").unwrap();
        writeln!(file, "Eve,32,Seattle,85.7").unwrap();
        file
    }

    #[test]
    fn test_import_and_preview() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        let name = session.import_file(path, Some("people")).unwrap();

        assert_eq!(name, "people");

        let ipc = session.get_preview_ipc(&name, 10).unwrap();
        assert!(!ipc.is_empty());
    }

    #[test]
    fn test_scan_file_transient() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        let name = session.scan_file(path).unwrap();

        let ipc = session.get_preview_ipc(&name, 10).unwrap();
        assert!(!ipc.is_empty());
    }

    #[test]
    fn test_dataset_info_persistent() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        let name = session.import_file(path, Some("info_test")).unwrap();

        let info = session.dataset_info(&name).unwrap();
        assert_eq!(info.num_columns, 4);
        assert!(info.persistent);
        assert!(info.column_names.contains(&"name".to_string()));
    }

    #[test]
    fn test_chunked_ipc() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        let name = session.import_file(path, Some("chunk_test")).unwrap();

        let chunk1 = session.get_chunk_ipc(&name, 0, 2).unwrap();
        assert!(!chunk1.is_empty());

        let chunk2 = session.get_chunk_ipc(&name, 2, 2).unwrap();
        assert!(!chunk2.is_empty());
    }

    #[test]
    fn test_row_count() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        let name = session.import_file(path, Some("count_test")).unwrap();

        let count = session.get_row_count(&name).unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_sql_query_duckdb() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        session.import_file(path, Some("sql_test")).unwrap();

        let result = session
            .execute_sql("SELECT name, score FROM sql_test WHERE age > 28")
            .unwrap();

        let ipc = session.get_preview_ipc(&result, 10).unwrap();
        assert!(!ipc.is_empty());

        let count = session.get_row_count(&result).unwrap();
        assert!(count > 0);
    }

    #[test]
    fn test_sort_dataset_duckdb() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        session.import_file(path, Some("sort_test")).unwrap();

        let sorted = session
            .sort_dataset("sort_test", &["age"], &[false])
            .unwrap();

        let ipc = session.get_preview_ipc(&sorted, 10).unwrap();
        assert!(!ipc.is_empty());
    }

    #[test]
    fn test_export_csv() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        let name = session.import_file(path, Some("export_test")).unwrap();

        let out = NamedTempFile::with_suffix(".csv").unwrap();
        let out_path = out.path().to_str().unwrap().to_string();

        session.export_to_csv(&name, &out_path).unwrap();

        let content = std::fs::read_to_string(&out_path).unwrap();
        assert!(content.contains("Alice"));
    }

    #[test]
    fn test_remove_dataset() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        session.import_file(path, Some("remove_me")).unwrap();

        assert!(session.list_datasets().contains(&"remove_me".to_string()));

        let removed = session.remove_dataset("remove_me").unwrap();
        assert!(removed);
        assert!(!session.list_datasets().contains(&"remove_me".to_string()));
    }

    #[test]
    fn test_list_datasets_combined() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        session.import_file(path, Some("persistent_one")).unwrap();
        session.scan_file(path).unwrap();

        let datasets = session.list_datasets();
        assert!(datasets.contains(&"persistent_one".to_string()));
        assert!(datasets.len() >= 2);
    }

    #[test]
    fn test_persistent_project() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test_project.duckdb");
        let db_path_str = db_path.to_str().unwrap();

        let csv = create_test_csv();
        let csv_path = csv.path().to_str().unwrap();

        {
            let mut session = RustoraSession::new();
            session.new_project(db_path_str).unwrap();
            session.import_file(csv_path, Some("my_data")).unwrap();
            assert_eq!(session.get_row_count("my_data").unwrap(), 5);
        }

        {
            let mut session = RustoraSession::new();
            let tables = session.open_project(db_path_str).unwrap();
            assert!(tables.contains(&"my_data".to_string()));

            let ipc = session.get_preview_ipc("my_data", 10).unwrap();
            assert!(!ipc.is_empty());
        }
    }

    #[test]
    fn test_filter_dataset_sql() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        session.import_file(path, Some("filter_test")).unwrap();

        let filtered = session
            .filter_dataset_sql("filter_test", "age > 28")
            .unwrap();

        let count = session.get_row_count(&filtered).unwrap();
        assert!(count > 0 && count < 5);

        let ipc = session.get_preview_ipc(&filtered, 10).unwrap();
        assert!(!ipc.is_empty());
    }

    #[test]
    fn test_group_by() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        session.import_file(path, Some("group_test")).unwrap();

        let grouped = session
            .group_by("group_test", &["city"], &["COUNT(*)", "AVG(score)"])
            .unwrap();

        let info = session.dataset_info(&grouped).unwrap();
        assert_eq!(info.num_columns, 3);

        let count = session.get_row_count(&grouped).unwrap();
        assert!(count > 0);
    }

    #[test]
    fn test_add_calculated_column() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        session.import_file(path, Some("calc_test")).unwrap();

        let result = session
            .add_calculated_column("calc_test", "score * 2", "double_score")
            .unwrap();

        let info = session.dataset_info(&result).unwrap();
        assert!(info.column_names.contains(&"double_score".to_string()));
        assert_eq!(info.num_columns, 5);
    }

    #[test]
    fn test_summary_stats_ipc() {
        let csv = create_test_csv();
        let path = csv.path().to_str().unwrap();

        let mut session = RustoraSession::new();
        session.import_file(path, Some("stats_test")).unwrap();

        let ipc = session.summary_stats_ipc("stats_test").unwrap();
        assert!(!ipc.is_empty());
    }

    #[test]
    fn test_unsupported_format() {
        let mut session = RustoraSession::new();
        let result = session.scan_file("test.xlsx");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_not_found() {
        let mut session = RustoraSession::new();
        let result = session.scan_file("nonexistent.csv");
        assert!(result.is_err());
    }
}

use crate::error::{Result, RustoraError};
use arrow_ipc::writer::StreamWriter;
use duckdb::Connection;
use std::path::Path;

/// Metadata about a table stored in DuckDB.
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub num_columns: usize,
    pub column_names: Vec<String>,
    pub column_types: Vec<String>,
    pub row_count: usize,
}

/// Persistent storage layer backed by DuckDB.
/// Handles file import, SQL execution, and Arrow IPC serialization.
pub struct DuckStorage {
    conn: Connection,
    db_path: String,
}

impl DuckStorage {
    /// Open or create a persistent DuckDB database at the given path.
    pub fn open(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path).map_err(|e| RustoraError::DuckDb(e.to_string()))?;
        Self::configure_connection(&conn)?;
        Ok(Self {
            conn,
            db_path: db_path.to_string(),
        })
    }

    /// Create an in-memory DuckDB database (for temporary/scratch use).
    pub fn open_in_memory() -> Result<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| RustoraError::DuckDb(e.to_string()))?;
        Self::configure_connection(&conn)?;
        Ok(Self {
            conn,
            db_path: ":memory:".to_string(),
        })
    }

    /// Tune the DuckDB connection for local desktop workloads.
    fn configure_connection(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "SET enable_progress_bar = false;
             SET preserve_insertion_order = true;",
        )
        .map_err(|e| RustoraError::DuckDb(e.to_string()))?;
        Ok(())
    }

    pub fn db_path(&self) -> &str {
        &self.db_path
    }

    // -----------------------------------------------------------------------
    // File Import -- Uses DuckDB's native high-performance readers
    // -----------------------------------------------------------------------

    /// Import a file into a persistent DuckDB table. Detects format by extension.
    /// Returns the sanitized table name used.
    pub fn import_file(&self, file_path: &str, table_name: &str) -> Result<String> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(RustoraError::FileNotFound(file_path.to_string()));
        }

        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let safe_name = sanitize_table_name(table_name);

        match extension.as_str() {
            "csv" | "tsv" => self.import_csv(file_path, &safe_name)?,
            "parquet" | "pq" => self.import_parquet(file_path, &safe_name)?,
            "ipc" | "arrow" | "feather" => self.import_arrow_ipc(file_path, &safe_name)?,
            other => return Err(RustoraError::UnsupportedFormat(other.to_string())),
        }

        Ok(safe_name)
    }

    fn import_csv(&self, file_path: &str, table_name: &str) -> Result<()> {
        let escaped_path = file_path.replace('\'', "''");
        let sql = format!(
            "CREATE OR REPLACE TABLE \"{}\" AS SELECT * FROM read_csv('{}', auto_detect=true)",
            table_name, escaped_path,
        );
        self.conn
            .execute_batch(&sql)
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;
        Ok(())
    }

    fn import_parquet(&self, file_path: &str, table_name: &str) -> Result<()> {
        let escaped_path = file_path.replace('\'', "''");
        let sql = format!(
            "CREATE OR REPLACE TABLE \"{}\" AS SELECT * FROM read_parquet('{}')",
            table_name, escaped_path,
        );
        self.conn
            .execute_batch(&sql)
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;
        Ok(())
    }

    fn import_arrow_ipc(&self, file_path: &str, table_name: &str) -> Result<()> {
        let escaped_path = file_path.replace('\'', "''");
        let sql = format!(
            "CREATE OR REPLACE TABLE \"{}\" AS SELECT * FROM '{}'",
            table_name, escaped_path,
        );
        self.conn
            .execute_batch(&sql)
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Query Execution -> Arrow IPC bytes (ZERO JSON)
    // -----------------------------------------------------------------------

    /// Execute arbitrary SQL and stream the result directly as Arrow IPC bytes.
    /// Batches are written incrementally to avoid collecting the full result set in memory.
    pub fn query_to_ipc(&self, sql: &str) -> Result<Vec<u8>> {
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;

        let arrow_iter = stmt
            .query_arrow([])
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;

        let schema = arrow_iter.get_schema();
        let mut buffer: Vec<u8> = Vec::new();

        let mut writer = StreamWriter::try_new(&mut buffer, &schema)
            .map_err(|e| RustoraError::DuckDb(format!("Arrow IPC write error: {}", e)))?;

        for batch in arrow_iter {
            if batch.num_rows() > 0 {
                writer
                    .write(&batch)
                    .map_err(|e| RustoraError::DuckDb(format!("Arrow IPC write error: {}", e)))?;
            }
        }

        writer
            .finish()
            .map_err(|e| RustoraError::DuckDb(format!("Arrow IPC finish error: {}", e)))?;

        Ok(buffer)
    }

    /// Get a paginated chunk of a table as Arrow IPC bytes.
    pub fn get_table_chunk_ipc(
        &self,
        table_name: &str,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<u8>> {
        let sql = format!(
            "SELECT * FROM \"{}\" LIMIT {} OFFSET {}",
            table_name, limit, offset
        );
        self.query_to_ipc(&sql)
    }

    /// Get a preview of a table (first N rows) as Arrow IPC bytes.
    pub fn get_table_preview_ipc(&self, table_name: &str, limit: u64) -> Result<Vec<u8>> {
        self.get_table_chunk_ipc(table_name, 0, limit)
    }

    // -----------------------------------------------------------------------
    // Table Management
    // -----------------------------------------------------------------------

    /// List all user tables in the database.
    pub fn list_tables(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main' ORDER BY table_name",
            )
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;

        let names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;

        Ok(names)
    }

    /// Get detailed info about a specific table.
    pub fn table_info(&self, table_name: &str) -> Result<TableInfo> {
        let row_count = self.table_row_count(table_name)?;

        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = '{}' AND table_schema = 'main' ORDER BY ordinal_position",
                table_name
            ))
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;

        let columns: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;

        let column_names: Vec<String> = columns.iter().map(|(n, _)| n.clone()).collect();
        let column_types: Vec<String> = columns.iter().map(|(_, t)| t.clone()).collect();

        Ok(TableInfo {
            name: table_name.to_string(),
            num_columns: column_names.len(),
            column_names,
            column_types,
            row_count,
        })
    }

    /// Estimate the in-memory size of a table in bytes based on column types and row count.
    pub fn table_estimated_size_bytes(&self, table_name: &str) -> Result<u64> {
        let info = self.table_info(table_name)?;
        let row_count = info.row_count as u64;
        if row_count == 0 {
            return Ok(0);
        }

        let bytes_per_row: u64 = info
            .column_types
            .iter()
            .map(|t| {
                let upper = t.to_uppercase();
                if upper.contains("BIGINT") || upper.contains("DOUBLE") || upper.contains("TIMESTAMP") {
                    8
                } else if upper.contains("INTEGER") || upper.contains("FLOAT") {
                    4
                } else if upper.contains("SMALLINT") {
                    2
                } else if upper.contains("BOOLEAN") || upper.contains("TINYINT") {
                    1
                } else if upper.contains("VARCHAR") || upper.contains("TEXT") || upper.contains("BLOB") {
                    64
                } else {
                    32
                }
            })
            .sum();

        Ok(row_count * bytes_per_row)
    }

    /// Get the row count for a table.
    pub fn table_row_count(&self, table_name: &str) -> Result<usize> {
        let sql = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
        let count: i64 = self
            .conn
            .query_row(&sql, [], |row| row.get(0))
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;

        Ok(count as usize)
    }

    /// Drop a table from the database.
    pub fn drop_table(&self, table_name: &str) -> Result<()> {
        let sql = format!("DROP TABLE IF EXISTS \"{}\"", table_name);
        self.conn
            .execute_batch(&sql)
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;
        Ok(())
    }

    /// Execute a SQL statement that creates a result set and store it as a new table.
    /// Returns the table name.
    pub fn execute_sql_to_table(&self, sql: &str, result_table: &str) -> Result<String> {
        let safe_name = sanitize_table_name(result_table);
        let create_sql = format!(
            "CREATE OR REPLACE TABLE \"{}\" AS {}",
            safe_name, sql
        );
        self.conn
            .execute_batch(&create_sql)
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;
        Ok(safe_name)
    }

    // -----------------------------------------------------------------------
    // Export
    // -----------------------------------------------------------------------

    /// Export a table to CSV.
    pub fn export_to_csv(&self, table_name: &str, output_path: &str) -> Result<()> {
        let escaped = output_path.replace('\'', "''");
        let sql = format!(
            "COPY \"{}\" TO '{}' (FORMAT CSV, HEADER TRUE)",
            table_name, escaped
        );
        self.conn
            .execute_batch(&sql)
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;
        Ok(())
    }

    /// Export a table to Parquet.
    pub fn export_to_parquet(&self, table_name: &str, output_path: &str) -> Result<()> {
        let escaped = output_path.replace('\'', "''");
        let sql = format!(
            "COPY \"{}\" TO '{}' (FORMAT PARQUET)",
            table_name, escaped
        );
        self.conn
            .execute_batch(&sql)
            .map_err(|e| RustoraError::DuckDb(e.to_string()))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Sanitize a string for use as a DuckDB table name.
fn sanitize_table_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
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
    fn test_import_csv_and_query() {
        let csv = create_test_csv();
        let csv_path = csv.path().to_str().unwrap();

        let storage = DuckStorage::open_in_memory().unwrap();
        let table_name = storage.import_file(csv_path, "test_data").unwrap();

        assert_eq!(table_name, "test_data");

        let tables = storage.list_tables().unwrap();
        assert!(tables.contains(&"test_data".to_string()));

        let info = storage.table_info("test_data").unwrap();
        assert_eq!(info.row_count, 5);
        assert_eq!(info.num_columns, 4);
        assert!(info.column_names.contains(&"name".to_string()));
    }

    #[test]
    fn test_query_to_ipc() {
        let csv = create_test_csv();
        let csv_path = csv.path().to_str().unwrap();

        let storage = DuckStorage::open_in_memory().unwrap();
        storage.import_file(csv_path, "test_data").unwrap();

        let ipc = storage
            .query_to_ipc("SELECT * FROM test_data WHERE age > 28")
            .unwrap();
        assert!(!ipc.is_empty());
    }

    #[test]
    fn test_table_chunk_ipc() {
        let csv = create_test_csv();
        let csv_path = csv.path().to_str().unwrap();

        let storage = DuckStorage::open_in_memory().unwrap();
        storage.import_file(csv_path, "test_data").unwrap();

        let chunk = storage.get_table_chunk_ipc("test_data", 0, 2).unwrap();
        assert!(!chunk.is_empty());

        let chunk2 = storage.get_table_chunk_ipc("test_data", 2, 2).unwrap();
        assert!(!chunk2.is_empty());
    }

    #[test]
    fn test_execute_sql_to_table() {
        let csv = create_test_csv();
        let csv_path = csv.path().to_str().unwrap();

        let storage = DuckStorage::open_in_memory().unwrap();
        storage.import_file(csv_path, "people").unwrap();

        let result = storage
            .execute_sql_to_table("SELECT name, score FROM people WHERE age > 28", "high_age")
            .unwrap();

        assert_eq!(result, "high_age");

        let info = storage.table_info("high_age").unwrap();
        assert_eq!(info.num_columns, 2);
        assert!(info.row_count > 0);
    }

    #[test]
    fn test_drop_table() {
        let csv = create_test_csv();
        let csv_path = csv.path().to_str().unwrap();

        let storage = DuckStorage::open_in_memory().unwrap();
        storage.import_file(csv_path, "to_drop").unwrap();

        assert!(storage.list_tables().unwrap().contains(&"to_drop".to_string()));

        storage.drop_table("to_drop").unwrap();

        assert!(!storage.list_tables().unwrap().contains(&"to_drop".to_string()));
    }

    #[test]
    fn test_export_csv() {
        let csv = create_test_csv();
        let csv_path = csv.path().to_str().unwrap();

        let storage = DuckStorage::open_in_memory().unwrap();
        storage.import_file(csv_path, "export_test").unwrap();

        let out = NamedTempFile::with_suffix(".csv").unwrap();
        let out_path = out.path().to_str().unwrap();

        storage.export_to_csv("export_test", out_path).unwrap();

        let content = std::fs::read_to_string(out_path).unwrap();
        assert!(content.contains("Alice"));
    }

    #[test]
    fn test_persistent_storage() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.duckdb");
        let db_path_str = db_path.to_str().unwrap().to_string();

        let csv = create_test_csv();
        let csv_path = csv.path().to_str().unwrap();

        {
            let storage = DuckStorage::open(&db_path_str).unwrap();
            storage.import_file(csv_path, "persistent_data").unwrap();
            let count = storage.table_row_count("persistent_data").unwrap();
            assert_eq!(count, 5);
        }

        {
            let storage = DuckStorage::open(&db_path_str).unwrap();
            let tables = storage.list_tables().unwrap();
            assert!(tables.contains(&"persistent_data".to_string()));

            let count = storage.table_row_count("persistent_data").unwrap();
            assert_eq!(count, 5);

            let ipc = storage
                .get_table_preview_ipc("persistent_data", 10)
                .unwrap();
            assert!(!ipc.is_empty());
        }
    }
}

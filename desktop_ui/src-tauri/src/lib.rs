use core_engine::{
    FilterCondition, FilterLogic, FilterOperator, FilterSpec, RustoraError, RustoraSession,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

/// Thread-safe wrapper around the core engine session.
/// Uses Arc so the mutex can be cloned into async spawn_blocking tasks
/// without holding a borrow on the Tauri State across await points.
struct AppState {
    session: Arc<Mutex<RustoraSession>>,
}

/// Structured error sent to the frontend as JSON so the UI can render
/// context-aware messages (e.g. distinguishing "file not found" from
/// "SQL syntax error").
#[derive(Debug, Serialize)]
struct CommandError {
    code: String,
    category: String,
    message: String,
}

impl CommandError {
    fn internal(message: String) -> Self {
        Self {
            code: "internal_error".to_string(),
            category: "internal".to_string(),
            message,
        }
    }
}

impl From<RustoraError> for CommandError {
    fn from(e: RustoraError) -> Self {
        let (code, category) = match &e {
            RustoraError::UnsupportedFormat(_) => ("unsupported_format", "file"),
            RustoraError::FileNotFound(_) => ("file_not_found", "file"),
            RustoraError::Polars(_) => ("polars_error", "data"),
            RustoraError::DuckDb(_) => ("duckdb_error", "data"),
            RustoraError::Io(_) => ("io_error", "file"),
            RustoraError::NoActiveDataFrame => ("no_active_dataframe", "session"),
            RustoraError::TableNotFound(_) => ("table_not_found", "data"),
            RustoraError::ColumnNotFound(_) => ("column_not_found", "data"),
            RustoraError::InvalidEdit(_) => ("invalid_edit", "data"),
            RustoraError::NoProjectOpen => ("no_project_open", "session"),
            RustoraError::Session(_) => ("session_error", "session"),
        };
        Self {
            code: code.to_string(),
            category: category.to_string(),
            message: e.to_string(),
        }
    }
}

/// Column info returned to the frontend for schema display.
#[derive(Serialize, Clone)]
struct ColumnInfo {
    name: String,
    dtype: String,
}

/// Metadata about an opened dataset returned to the frontend.
#[derive(Serialize)]
struct OpenResult {
    dataset_name: String,
    columns: Vec<ColumnInfo>,
    total_rows: usize,
    persistent: bool,
    size_bytes: Option<u64>,
}

/// Info about the current project.
#[derive(Serialize)]
struct ProjectInfo {
    path: String,
    tables: Vec<String>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_open_result(session: &RustoraSession, name: &str) -> Result<OpenResult, CommandError> {
    let info = session.dataset_info(name)?;
    let total_rows = info
        .estimated_rows
        .unwrap_or_else(|| session.get_row_count(name).unwrap_or(0));

    let columns: Vec<ColumnInfo> = info
        .column_names
        .iter()
        .zip(info.column_dtypes.iter())
        .map(|(n, d)| ColumnInfo {
            name: n.clone(),
            dtype: d.clone(),
        })
        .collect();

    Ok(OpenResult {
        dataset_name: name.to_string(),
        columns,
        total_rows,
        persistent: info.persistent,
        size_bytes: info.estimated_size_bytes,
    })
}

// ---------------------------------------------------------------------------
// Project Commands
// ---------------------------------------------------------------------------

/// Create a new project (.duckdb file).
#[tauri::command]
async fn new_project(state: State<'_, AppState>, path: String) -> Result<ProjectInfo, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        session.new_project(&path)?;
        Ok(ProjectInfo {
            path,
            tables: vec![],
        })
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Open an existing project (.duckdb file). Returns list of persistent tables.
#[tauri::command]
async fn open_project(state: State<'_, AppState>, path: String) -> Result<ProjectInfo, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        let tables = session.open_project(&path)?;
        Ok(ProjectInfo { path, tables })
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Get current project info.
#[tauri::command]
async fn get_project_info(state: State<'_, AppState>) -> Result<Option<ProjectInfo>, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        match session.project_path() {
            Some(path) => {
                let tables = session.list_datasets();
                Ok(Some(ProjectInfo {
                    path: path.to_string(),
                    tables,
                }))
            }
            None => Ok(None),
        }
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

// ---------------------------------------------------------------------------
// Data Import & File Open Commands
// ---------------------------------------------------------------------------

/// Import a file into the DuckDB project as a persistent table.
#[tauri::command]
async fn import_file(
    state: State<'_, AppState>,
    path: String,
    table_name: Option<String>,
) -> Result<OpenResult, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        let name = session.import_file(&path, table_name.as_deref())?;
        make_open_result(&session, &name)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Open a file (backwards-compatible: imports to DuckDB if project is open,
/// falls back to transient Polars scan).
#[tauri::command]
async fn open_file(state: State<'_, AppState>, path: String) -> Result<OpenResult, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        let name = if session.project_path().is_some() {
            session.import_file(&path, None)?
        } else {
            session.scan_file(&path)?
        };
        make_open_result(&session, &name)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

// ---------------------------------------------------------------------------
// Data Access Commands
// ---------------------------------------------------------------------------

/// Get a chunk of rows as Arrow IPC bytes for the virtualized grid.
#[tauri::command]
async fn get_chunk(
    state: State<'_, AppState>,
    dataset_name: String,
    offset: u32,
    limit: u32,
) -> Result<Vec<u8>, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        Ok(session.get_chunk_ipc(&dataset_name, offset, limit)?)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Sort a dataset and return new dataset metadata.
#[tauri::command]
async fn sort_dataset(
    state: State<'_, AppState>,
    dataset_name: String,
    columns: Vec<String>,
    descending: Vec<bool>,
) -> Result<OpenResult, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        let col_refs: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
        let new_name = session.sort_dataset(&dataset_name, &col_refs, &descending)?;
        make_open_result(&session, &new_name)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Execute a SQL query against DuckDB and return the result dataset metadata.
#[tauri::command]
async fn execute_sql(state: State<'_, AppState>, sql: String) -> Result<OpenResult, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        let new_name = session.execute_sql(&sql)?;
        make_open_result(&session, &new_name)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Export a dataset to a file (CSV or Parquet).
#[tauri::command]
async fn export_dataset(
    state: State<'_, AppState>,
    dataset_name: String,
    output_path: String,
    format: String,
) -> Result<(), CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        match format.as_str() {
            "csv" => Ok(session.export_to_csv(&dataset_name, &output_path)?),
            "parquet" => Ok(session.export_to_parquet(&dataset_name, &output_path)?),
            _ => Err(CommandError {
                code: "unsupported_format".to_string(),
                category: "file".to_string(),
                message: format!("Unsupported export format: {}", format),
            }),
        }
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// List all loaded datasets (persistent + transient).
#[tauri::command]
async fn list_datasets(state: State<'_, AppState>) -> Result<Vec<String>, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        Ok(session.list_datasets())
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Remove a dataset.
#[tauri::command]
async fn remove_dataset(state: State<'_, AppState>, dataset_name: String) -> Result<bool, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        Ok(session.remove_dataset(&dataset_name)?)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

// ---------------------------------------------------------------------------
// Transform & Analyze Commands
// ---------------------------------------------------------------------------

/// Filter a dataset using a SQL WHERE clause.
#[tauri::command]
async fn filter_dataset(
    state: State<'_, AppState>,
    dataset_name: String,
    where_clause: String,
) -> Result<OpenResult, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        let new_name = session.filter_dataset_sql(&dataset_name, &where_clause)?;
        make_open_result(&session, &new_name)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// A single filter condition from the frontend.
#[derive(Deserialize)]
struct FilterConditionInput {
    column: String,
    operator: String,
    value: String,
}

fn parse_operator(op: &str) -> Result<FilterOperator, CommandError> {
    match op {
        "equals" => Ok(FilterOperator::Equals),
        "not_equals" => Ok(FilterOperator::NotEquals),
        "greater_than" => Ok(FilterOperator::GreaterThan),
        "greater_than_or_equal" => Ok(FilterOperator::GreaterThanOrEqual),
        "less_than" => Ok(FilterOperator::LessThan),
        "less_than_or_equal" => Ok(FilterOperator::LessThanOrEqual),
        "contains" => Ok(FilterOperator::Contains),
        "not_contains" => Ok(FilterOperator::NotContains),
        "starts_with" => Ok(FilterOperator::StartsWith),
        "ends_with" => Ok(FilterOperator::EndsWith),
        "is_null" => Ok(FilterOperator::IsNull),
        "is_not_null" => Ok(FilterOperator::IsNotNull),
        _ => Err(CommandError {
            code: "invalid_operator".to_string(),
            category: "data".to_string(),
            message: format!("Unknown filter operator: {}", op),
        }),
    }
}

/// Filter a dataset using structured conditions (safe from SQL injection).
#[tauri::command]
async fn filter_dataset_structured(
    state: State<'_, AppState>,
    dataset_name: String,
    conditions: Vec<FilterConditionInput>,
    logic: String,
) -> Result<OpenResult, CommandError> {
    let parsed_conditions: Vec<FilterCondition> = conditions
        .into_iter()
        .map(|c| {
            Ok(FilterCondition {
                column: c.column,
                operator: parse_operator(&c.operator)?,
                value: c.value,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;

    let filter_logic = match logic.as_str() {
        "or" => FilterLogic::Or,
        _ => FilterLogic::And,
    };

    let spec = FilterSpec {
        conditions: parsed_conditions,
        logic: filter_logic,
    };

    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        let new_name = session.filter_dataset_structured(&dataset_name, &spec)?;
        make_open_result(&session, &new_name)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Group a dataset by columns with aggregate expressions.
#[tauri::command]
async fn group_by(
    state: State<'_, AppState>,
    dataset_name: String,
    group_columns: Vec<String>,
    agg_exprs: Vec<String>,
) -> Result<OpenResult, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        let col_refs: Vec<&str> = group_columns.iter().map(|s| s.as_str()).collect();
        let agg_refs: Vec<&str> = agg_exprs.iter().map(|s| s.as_str()).collect();
        let new_name = session.group_by(&dataset_name, &col_refs, &agg_refs)?;
        make_open_result(&session, &new_name)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Add a calculated column to a dataset.
#[tauri::command]
async fn add_calculated_column(
    state: State<'_, AppState>,
    dataset_name: String,
    expression: String,
    alias: String,
) -> Result<OpenResult, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        let new_name = session.add_calculated_column(&dataset_name, &expression, &alias)?;
        make_open_result(&session, &new_name)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Aggregate data for chart visualization. Returns Arrow IPC bytes.
#[tauri::command]
async fn aggregate_for_chart(
    state: State<'_, AppState>,
    dataset_name: String,
    group_col: String,
    value_col: Option<String>,
    agg_type: String,
    limit: u32,
) -> Result<Vec<u8>, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        Ok(session.aggregate_for_chart(
            &dataset_name,
            &group_col,
            value_col.as_deref(),
            &agg_type,
            limit,
        )?)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

/// Get summary statistics for a dataset as Arrow IPC bytes.
#[tauri::command]
async fn get_summary_stats(
    state: State<'_, AppState>,
    dataset_name: String,
) -> Result<Vec<u8>, CommandError> {
    let session = state.session.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let session = session.lock().map_err(|e| CommandError::internal(e.to_string()))?;
        Ok(session.summary_stats_ipc(&dataset_name)?)
    })
    .await
    .map_err(|e| CommandError::internal(e.to_string()))?
}

// ---------------------------------------------------------------------------
// Application Entry Point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            session: Arc::new(Mutex::new(RustoraSession::new())),
        })
        .invoke_handler(tauri::generate_handler![
            new_project,
            open_project,
            get_project_info,
            import_file,
            open_file,
            get_chunk,
            sort_dataset,
            execute_sql,
            export_dataset,
            list_datasets,
            remove_dataset,
            filter_dataset,
            filter_dataset_structured,
            group_by,
            add_calculated_column,
            aggregate_for_chart,
            get_summary_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Rustora");
}

use core_engine::{FilterCondition, FilterLogic, FilterOperator, FilterSpec, RustoraSession};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// Thread-safe wrapper around the core engine session.
struct AppState {
    session: Mutex<RustoraSession>,
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

fn make_open_result(session: &RustoraSession, name: &str) -> Result<OpenResult, String> {
    let info = session.dataset_info(name).map_err(|e| e.to_string())?;
    let total_rows = info.estimated_rows.unwrap_or_else(|| {
        session.get_row_count(name).unwrap_or(0)
    });

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
fn new_project(state: State<'_, AppState>, path: String) -> Result<ProjectInfo, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    session.new_project(&path).map_err(|e| e.to_string())?;
    Ok(ProjectInfo {
        path,
        tables: vec![],
    })
}

/// Open an existing project (.duckdb file). Returns list of persistent tables.
#[tauri::command]
fn open_project(state: State<'_, AppState>, path: String) -> Result<ProjectInfo, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let tables = session.open_project(&path).map_err(|e| e.to_string())?;
    Ok(ProjectInfo { path, tables })
}

/// Get current project info.
#[tauri::command]
fn get_project_info(state: State<'_, AppState>) -> Result<Option<ProjectInfo>, String> {
    let session = state.session.lock().map_err(|e| e.to_string())?;
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
}

// ---------------------------------------------------------------------------
// Data Import & File Open Commands
// ---------------------------------------------------------------------------

/// Import a file into the DuckDB project as a persistent table.
#[tauri::command]
fn import_file(
    state: State<'_, AppState>,
    path: String,
    table_name: Option<String>,
) -> Result<OpenResult, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let name = session
        .import_file(&path, table_name.as_deref())
        .map_err(|e| e.to_string())?;
    make_open_result(&session, &name)
}

/// Open a file (backwards-compatible: imports to DuckDB if project is open,
/// falls back to transient Polars scan).
#[tauri::command]
fn open_file(state: State<'_, AppState>, path: String) -> Result<OpenResult, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;

    let name = if session.project_path().is_some() {
        session
            .import_file(&path, None)
            .map_err(|e| e.to_string())?
    } else {
        session.scan_file(&path).map_err(|e| e.to_string())?
    };

    make_open_result(&session, &name)
}

// ---------------------------------------------------------------------------
// Data Access Commands
// ---------------------------------------------------------------------------

/// Get a chunk of rows as Arrow IPC bytes for the virtualized grid.
#[tauri::command]
fn get_chunk(
    state: State<'_, AppState>,
    dataset_name: String,
    offset: u32,
    limit: u32,
) -> Result<Vec<u8>, String> {
    let session = state.session.lock().map_err(|e| e.to_string())?;
    session
        .get_chunk_ipc(&dataset_name, offset, limit)
        .map_err(|e| e.to_string())
}

/// Sort a dataset and return new dataset metadata.
#[tauri::command]
fn sort_dataset(
    state: State<'_, AppState>,
    dataset_name: String,
    columns: Vec<String>,
    descending: Vec<bool>,
) -> Result<OpenResult, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;

    let col_refs: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
    let new_name = session
        .sort_dataset(&dataset_name, &col_refs, &descending)
        .map_err(|e| e.to_string())?;

    make_open_result(&session, &new_name)
}

/// Execute a SQL query against DuckDB and return the result dataset metadata.
#[tauri::command]
fn execute_sql(state: State<'_, AppState>, sql: String) -> Result<OpenResult, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let new_name = session.execute_sql(&sql).map_err(|e| e.to_string())?;
    make_open_result(&session, &new_name)
}

/// Export a dataset to a file (CSV or Parquet).
#[tauri::command]
fn export_dataset(
    state: State<'_, AppState>,
    dataset_name: String,
    output_path: String,
    format: String,
) -> Result<(), String> {
    let session = state.session.lock().map_err(|e| e.to_string())?;
    match format.as_str() {
        "csv" => session
            .export_to_csv(&dataset_name, &output_path)
            .map_err(|e| e.to_string()),
        "parquet" => session
            .export_to_parquet(&dataset_name, &output_path)
            .map_err(|e| e.to_string()),
        _ => Err(format!("Unsupported export format: {}", format)),
    }
}

/// List all loaded datasets (persistent + transient).
#[tauri::command]
fn list_datasets(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let session = state.session.lock().map_err(|e| e.to_string())?;
    Ok(session.list_datasets())
}

/// Remove a dataset.
#[tauri::command]
fn remove_dataset(state: State<'_, AppState>, dataset_name: String) -> Result<bool, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    session
        .remove_dataset(&dataset_name)
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Transform & Analyze Commands
// ---------------------------------------------------------------------------

/// Filter a dataset using a SQL WHERE clause.
#[tauri::command]
fn filter_dataset(
    state: State<'_, AppState>,
    dataset_name: String,
    where_clause: String,
) -> Result<OpenResult, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let new_name = session
        .filter_dataset_sql(&dataset_name, &where_clause)
        .map_err(|e| e.to_string())?;
    make_open_result(&session, &new_name)
}

/// A single filter condition from the frontend.
#[derive(Deserialize)]
struct FilterConditionInput {
    column: String,
    operator: String,
    value: String,
}

fn parse_operator(op: &str) -> Result<FilterOperator, String> {
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
        _ => Err(format!("Unknown filter operator: {}", op)),
    }
}

/// Filter a dataset using structured conditions (safe from SQL injection).
#[tauri::command]
fn filter_dataset_structured(
    state: State<'_, AppState>,
    dataset_name: String,
    conditions: Vec<FilterConditionInput>,
    logic: String,
) -> Result<OpenResult, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;

    let parsed_conditions: Vec<FilterCondition> = conditions
        .into_iter()
        .map(|c| {
            Ok(FilterCondition {
                column: c.column,
                operator: parse_operator(&c.operator)?,
                value: c.value,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let filter_logic = match logic.as_str() {
        "or" => FilterLogic::Or,
        _ => FilterLogic::And,
    };

    let spec = FilterSpec {
        conditions: parsed_conditions,
        logic: filter_logic,
    };

    let new_name = session
        .filter_dataset_structured(&dataset_name, &spec)
        .map_err(|e| e.to_string())?;
    make_open_result(&session, &new_name)
}

/// Group a dataset by columns with aggregate expressions.
#[tauri::command]
fn group_by(
    state: State<'_, AppState>,
    dataset_name: String,
    group_columns: Vec<String>,
    agg_exprs: Vec<String>,
) -> Result<OpenResult, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let col_refs: Vec<&str> = group_columns.iter().map(|s| s.as_str()).collect();
    let agg_refs: Vec<&str> = agg_exprs.iter().map(|s| s.as_str()).collect();
    let new_name = session
        .group_by(&dataset_name, &col_refs, &agg_refs)
        .map_err(|e| e.to_string())?;
    make_open_result(&session, &new_name)
}

/// Add a calculated column to a dataset.
#[tauri::command]
fn add_calculated_column(
    state: State<'_, AppState>,
    dataset_name: String,
    expression: String,
    alias: String,
) -> Result<OpenResult, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let new_name = session
        .add_calculated_column(&dataset_name, &expression, &alias)
        .map_err(|e| e.to_string())?;
    make_open_result(&session, &new_name)
}

/// Aggregate data for chart visualization. Returns Arrow IPC bytes.
#[tauri::command]
fn aggregate_for_chart(
    state: State<'_, AppState>,
    dataset_name: String,
    group_col: String,
    value_col: Option<String>,
    agg_type: String,
    limit: u32,
) -> Result<Vec<u8>, String> {
    let session = state.session.lock().map_err(|e| e.to_string())?;
    session
        .aggregate_for_chart(
            &dataset_name,
            &group_col,
            value_col.as_deref(),
            &agg_type,
            limit,
        )
        .map_err(|e| e.to_string())
}

/// Get summary statistics for a dataset as Arrow IPC bytes.
#[tauri::command]
fn get_summary_stats(
    state: State<'_, AppState>,
    dataset_name: String,
) -> Result<Vec<u8>, String> {
    let session = state.session.lock().map_err(|e| e.to_string())?;
    session
        .summary_stats_ipc(&dataset_name)
        .map_err(|e| e.to_string())
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
            session: Mutex::new(RustoraSession::new()),
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

use core_engine::{
    FilterCondition, FilterLogic, FilterOperator, FilterSpec, RustoraSession,
};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct StepDisplayEntry {
    pub label: String,
    pub result_table: String,
}

#[derive(Clone, Debug)]
pub struct ColumnInfo {
    pub name: String,
    pub dtype: String,
}

#[derive(Clone, Debug)]
pub struct OpenResult {
    pub dataset_name: String,
    pub columns: Vec<ColumnInfo>,
    pub total_rows: usize,
    pub persistent: bool,
    pub size_bytes: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct ProjectInfo {
    pub path: String,
    pub tables: Vec<String>,
}

pub struct SessionFacade {
    session: Arc<Mutex<RustoraSession>>,
}

impl SessionFacade {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(RustoraSession::new())),
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, RustoraSession>, String> {
        self.session
            .lock()
            .map_err(|e| format!("Session lock poisoned: {}", e))
    }

    fn make_open_result(
        session: &RustoraSession,
        name: &str,
    ) -> Result<OpenResult, String> {
        let info = session.dataset_info(name).map_err(|e| e.to_string())?;
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

    pub fn new_project(&self, path: &str) -> Result<ProjectInfo, String> {
        let mut session = self.lock()?;
        session.new_project(path).map_err(|e| e.to_string())?;
        Ok(ProjectInfo {
            path: path.to_string(),
            tables: vec![],
        })
    }

    pub fn open_project(&self, path: &str) -> Result<ProjectInfo, String> {
        let mut session = self.lock()?;
        let tables = session.open_project(path).map_err(|e| e.to_string())?;
        Ok(ProjectInfo {
            path: path.to_string(),
            tables,
        })
    }

    #[allow(dead_code)]
    pub fn project_path(&self) -> Option<String> {
        let session = self.lock().ok()?;
        session.project_path().map(|s| s.to_string())
    }

    pub fn import_file(
        &self,
        path: &str,
        table_name: Option<&str>,
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let name = session
            .import_file(path, table_name)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &name)
    }

    pub fn open_file(&self, path: &str) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let name = if session.project_path().is_some() {
            session.import_file(path, None).map_err(|e| e.to_string())?
        } else {
            session.scan_file(path).map_err(|e| e.to_string())?
        };
        Self::make_open_result(&session, &name)
    }

    pub fn get_chunk(
        &self,
        dataset_name: &str,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<u8>, String> {
        let session = self.lock()?;
        session
            .get_chunk_ipc(dataset_name, offset, limit)
            .map_err(|e| e.to_string())
    }

    pub fn sort_dataset(
        &self,
        dataset_name: &str,
        columns: &[&str],
        descending: &[bool],
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .sort_dataset(dataset_name, columns, descending)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn execute_sql(&self, sql: &str) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session.execute_sql(sql).map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn export_dataset(
        &self,
        dataset_name: &str,
        output_path: &str,
        format: &str,
    ) -> Result<(), String> {
        let session = self.lock()?;
        match format {
            "csv" => session
                .export_to_csv(dataset_name, output_path)
                .map_err(|e| e.to_string()),
            "parquet" => session
                .export_to_parquet(dataset_name, output_path)
                .map_err(|e| e.to_string()),
            _ => Err(format!("Unsupported export format: {}", format)),
        }
    }

    pub fn list_datasets(&self) -> Vec<String> {
        self.lock().map(|s| s.list_datasets()).unwrap_or_default()
    }

    pub fn get_row_count(&self, dataset_name: &str) -> Result<usize, String> {
        let session = self.lock()?;
        session.get_row_count(dataset_name).map_err(|e| e.to_string())
    }

    pub fn remove_dataset(&self, dataset_name: &str) -> Result<bool, String> {
        let mut session = self.lock()?;
        session
            .remove_dataset(dataset_name)
            .map_err(|e| e.to_string())
    }

    pub fn filter_dataset_sql(
        &self,
        dataset_name: &str,
        where_clause: &str,
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .filter_dataset_sql(dataset_name, where_clause)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn filter_dataset_structured(
        &self,
        dataset_name: &str,
        conditions: Vec<FilterCondition>,
        logic: FilterLogic,
    ) -> Result<OpenResult, String> {
        let spec = FilterSpec { conditions, logic };
        let mut session = self.lock()?;
        let new_name = session
            .filter_dataset_structured(dataset_name, &spec)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn group_by(
        &self,
        dataset_name: &str,
        group_columns: &[&str],
        agg_exprs: &[&str],
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .group_by(dataset_name, group_columns, agg_exprs)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn add_calculated_column(
        &self,
        dataset_name: &str,
        expression: &str,
        alias: &str,
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .add_calculated_column(dataset_name, expression, alias)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn aggregate_for_chart(
        &self,
        dataset_name: &str,
        group_col: &str,
        value_col: Option<&str>,
        agg_type: &str,
        limit: u32,
    ) -> Result<Vec<u8>, String> {
        let session = self.lock()?;
        session
            .aggregate_for_chart(dataset_name, group_col, value_col, agg_type, limit)
            .map_err(|e| e.to_string())
    }

    pub fn get_summary_stats(&self, dataset_name: &str) -> Result<Vec<u8>, String> {
        let session = self.lock()?;
        session
            .summary_stats_ipc(dataset_name)
            .map_err(|e| e.to_string())
    }

    pub fn get_steps(&self, dataset_name: &str) -> Vec<StepDisplayEntry> {
        let session = match self.lock() {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let history = session.get_history(dataset_name);
        history
            .entries()
            .iter()
            .map(|e| StepDisplayEntry {
                label: e.step.label(),
                result_table: e.result_table.clone(),
            })
            .collect()
    }

    pub fn get_columns(&self, dataset_name: &str) -> Result<Vec<ColumnInfo>, String> {
        let session = self.lock()?;
        let info = session.dataset_info(dataset_name).map_err(|e| e.to_string())?;
        Ok(info
            .column_names
            .iter()
            .zip(info.column_dtypes.iter())
            .map(|(n, d)| ColumnInfo {
                name: n.clone(),
                dtype: d.clone(),
            })
            .collect())
    }

    pub fn preview_file(
        &self,
        path: &str,
        delimiter: u8,
        has_header: bool,
        skip_rows: u32,
    ) -> Result<Vec<u8>, String> {
        let session = self.lock()?;
        session
            .preview_file(path, delimiter, has_header, skip_rows, 100)
            .map_err(|e| e.to_string())
    }

    pub fn import_file_with_options(
        &self,
        path: &str,
        table_name: Option<&str>,
        delimiter: u8,
        has_header: bool,
        skip_rows: u32,
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let name = session
            .import_file_with_options(path, table_name, delimiter, has_header, skip_rows)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &name)
    }

    pub fn remove_columns(
        &self,
        dataset_name: &str,
        columns: &[&str],
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .remove_columns(dataset_name, columns)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn keep_columns(
        &self,
        dataset_name: &str,
        columns: &[&str],
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .keep_columns(dataset_name, columns)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn change_column_type(
        &self,
        dataset_name: &str,
        column: &str,
        new_type: &str,
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .change_column_type(dataset_name, column, new_type)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn rename_column(
        &self,
        dataset_name: &str,
        old_col: &str,
        new_col: &str,
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .rename_column(dataset_name, old_col, new_col)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn pivot_dataset(
        &self,
        dataset_name: &str,
        index_cols: &[&str],
        pivot_col: &str,
        value_col: &str,
        agg: &str,
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .pivot_dataset(dataset_name, index_cols, pivot_col, value_col, agg)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn unpivot_dataset(
        &self,
        dataset_name: &str,
        value_cols: &[&str],
        var_name: &str,
        value_name: &str,
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .unpivot_dataset(dataset_name, value_cols, var_name, value_name)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn merge_datasets(
        &self,
        left: &str,
        right: &str,
        left_col: &str,
        right_col: &str,
        join_type: &str,
    ) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .merge_datasets(left, right, left_col, right_col, join_type)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }

    pub fn append_datasets(&self, tables: &[&str]) -> Result<OpenResult, String> {
        let mut session = self.lock()?;
        let new_name = session
            .append_datasets(tables)
            .map_err(|e| e.to_string())?;
        Self::make_open_result(&session, &new_name)
    }
}

pub fn parse_filter_operator(op: &str) -> Result<FilterOperator, String> {
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

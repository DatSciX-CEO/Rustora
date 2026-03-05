use crate::facade::{ColumnInfo, OpenResult, ProjectInfo, SessionFacade, StepDisplayEntry};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug)]
pub struct FeatureFlags {
    pub steps_panel: bool,
    pub ingest_preview: bool,
    pub column_ops: bool,
    pub pivot_unpivot: bool,
    pub merge_append: bool,
    pub formula_bar: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            steps_panel: true,
            ingest_preview: true,
            column_ops: true,
            pivot_unpivot: true,
            merge_append: true,
            formula_bar: true,
        }
    }
}

pub struct ParsedPage {
    pub columns: Vec<String>,
    pub dtypes: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
}

pub struct AppState {
    pub facade: SessionFacade,

    pub name: Option<String>,
    pub columns: Vec<ColumnInfo>,
    pub total_rows: usize,
    pub size_bytes: Option<u64>,
    pub current_page: Option<ParsedPage>,
    pub offset: usize,
    pub page_size: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub sort_column: Option<String>,
    pub sort_desc: bool,
    pub persistent: bool,
    pub project: Option<ProjectInfo>,
    pub tables: Vec<String>,

    pub sql_visible: bool,
    pub sql_text: String,
    pub chart_visible: bool,
    pub stats_visible: bool,
    pub stats_data: Option<ParsedPage>,

    pub chart_type: ChartType,
    pub chart_group_col: String,
    pub chart_value_col: String,
    pub chart_agg_type: String,
    pub chart_limit: u32,
    pub chart_data: Option<Vec<ChartRow>>,
    pub chart_error: Option<String>,

    pub filter_dialog: bool,
    pub filter_where: String,
    pub filter_mode_structured: bool,
    pub struct_filter_column: String,
    pub struct_filter_operator: String,
    pub struct_filter_value: String,
    pub group_dialog: bool,
    pub group_cols_input: String,
    pub group_aggs_input: String,
    pub calc_dialog: bool,
    pub calc_expr: String,
    pub calc_alias: String,

    pub features: FeatureFlags,

    // Phase 1: Applied Steps
    pub steps_visible: bool,
    pub steps_entries: Vec<StepDisplayEntry>,
    pub steps_active_index: Option<usize>,

    // Phase 2: Ingest Preview
    pub ingest_dialog: bool,
    pub ingest_file_path: String,
    pub ingest_preview: Option<ParsedPage>,
    pub ingest_delimiter: u8,
    pub ingest_has_header: bool,
    pub ingest_skip_rows: u32,
    pub ingest_table_name: String,

    // Phase 3: Pivot/Unpivot/Merge/Append
    pub pivot_dialog: bool,
    pub pivot_index_cols: String,
    pub pivot_col: String,
    pub pivot_value_col: String,
    pub pivot_agg: String,
    pub unpivot_dialog: bool,
    pub unpivot_value_cols: String,
    pub unpivot_var_name: String,
    pub unpivot_value_name: String,
    pub merge_dialog: bool,
    pub merge_right_table: String,
    pub merge_left_col: String,
    pub merge_right_col: String,
    pub merge_join_type: String,
    pub merge_right_columns: Vec<ColumnInfo>,
    pub append_dialog: bool,
    pub append_tables_input: String,

    // Phase 3b: Rename column
    pub rename_dialog: bool,
    pub rename_old_name: String,
    pub rename_new_name: String,

    // Phase 4: Cell selection & formula bar
    pub selected_cell: Option<(usize, usize)>,

    // Phase 6: Go to row
    pub goto_row_input: String,

    last_failed_action: Option<FailedAction>,
    #[allow(dead_code)]
    page_request_id: AtomicU64,
    #[allow(dead_code)]
    transform_request_id: AtomicU64,
}

#[derive(Clone, Debug)]
pub enum FailedAction {
    OpenFile(String),
    ImportFile(String),
    Sort(String),
    ExecuteSql(String),
    FilterSql(#[allow(dead_code)] String, String),
    GroupBy(String, String),
    CalcColumn(String, String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChartType {
    Bar,
    Line,
}

#[derive(Clone, Debug)]
pub struct ChartRow {
    pub label: String,
    pub value: f64,
}

const PAGE_SIZE: usize = 500;

impl AppState {
    pub fn new() -> Self {
        Self {
            facade: SessionFacade::new(),
            name: None,
            columns: vec![],
            total_rows: 0,
            size_bytes: None,
            current_page: None,
            offset: 0,
            page_size: PAGE_SIZE,
            loading: false,
            error: None,
            sort_column: None,
            sort_desc: false,
            persistent: false,
            project: None,
            tables: vec![],
            sql_visible: false,
            sql_text: String::new(),
            chart_visible: false,
            stats_visible: false,
            stats_data: None,
            chart_type: ChartType::Bar,
            chart_group_col: String::new(),
            chart_value_col: String::new(),
            chart_agg_type: "count".to_string(),
            chart_limit: 20,
            chart_data: None,
            chart_error: None,
            filter_dialog: false,
            filter_where: String::new(),
            filter_mode_structured: false,
            struct_filter_column: String::new(),
            struct_filter_operator: "equals".to_string(),
            struct_filter_value: String::new(),
            group_dialog: false,
            group_cols_input: String::new(),
            group_aggs_input: String::new(),
            calc_dialog: false,
            calc_expr: String::new(),
            calc_alias: String::new(),
            features: FeatureFlags::default(),
            steps_visible: false,
            steps_entries: vec![],
            steps_active_index: None,
            ingest_dialog: false,
            ingest_file_path: String::new(),
            ingest_preview: None,
            ingest_delimiter: b',',
            ingest_has_header: true,
            ingest_skip_rows: 0,
            ingest_table_name: String::new(),
            pivot_dialog: false,
            pivot_index_cols: String::new(),
            pivot_col: String::new(),
            pivot_value_col: String::new(),
            pivot_agg: "sum".to_string(),
            unpivot_dialog: false,
            unpivot_value_cols: String::new(),
            unpivot_var_name: "variable".to_string(),
            unpivot_value_name: "value".to_string(),
            merge_dialog: false,
            merge_right_table: String::new(),
            merge_left_col: String::new(),
            merge_right_col: String::new(),
            merge_join_type: "inner".to_string(),
            merge_right_columns: vec![],
            append_dialog: false,
            append_tables_input: String::new(),
            rename_dialog: false,
            rename_old_name: String::new(),
            rename_new_name: String::new(),
            selected_cell: None,
            goto_row_input: String::new(),
            last_failed_action: None,
            page_request_id: AtomicU64::new(0),
            transform_request_id: AtomicU64::new(0),
        }
    }

    #[allow(dead_code)]
    fn next_page_id(&self) -> u64 {
        self.page_request_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    #[allow(dead_code)]
    fn next_transform_id(&self) -> u64 {
        self.transform_request_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    #[allow(dead_code)]
    fn current_page_id(&self) -> u64 {
        self.page_request_id.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    fn current_transform_id(&self) -> u64 {
        self.transform_request_id.load(Ordering::Relaxed)
    }

    pub fn parse_ipc_to_page(ipc_bytes: &[u8]) -> Option<ParsedPage> {
        use arrow_array::Array;
        use arrow_ipc::reader::StreamReader;
        use std::io::Cursor;

        let cursor = Cursor::new(ipc_bytes);
        let reader = StreamReader::try_new(cursor, None).ok()?;
        let schema = reader.schema();

        let columns: Vec<String> = schema.fields().iter().map(|f| f.name().clone()).collect();
        let dtypes: Vec<String> = schema
            .fields()
            .iter()
            .map(|f| format!("{:?}", f.data_type()))
            .collect();

        let mut all_rows: Vec<Vec<String>> = Vec::new();

        for batch_result in reader {
            let batch = batch_result.ok()?;
            let num_rows = batch.num_rows();
            let num_cols = batch.num_columns();

            let string_cols: Vec<arrow_array::StringArray> = (0..num_cols)
                .map(|col_idx| {
                    let col = batch.column(col_idx);
                    arrow_cast::cast(col, &arrow_schema::DataType::Utf8)
                        .map(|arr| {
                            arr.as_any()
                                .downcast_ref::<arrow_array::StringArray>()
                                .cloned()
                                .unwrap_or_else(|| {
                                    arrow_array::StringArray::from(vec![""; col.len()])
                                })
                        })
                        .unwrap_or_else(|_| {
                            arrow_array::StringArray::from(vec![""; col.len()])
                        })
                })
                .collect();

            for row_idx in 0..num_rows {
                let mut row = Vec::with_capacity(num_cols);
                for col_idx in 0..num_cols {
                    let val = if string_cols[col_idx].is_null(row_idx) {
                        String::new()
                    } else {
                        string_cols[col_idx].value(row_idx).to_string()
                    };
                    row.push(val);
                }
                all_rows.push(row);
            }
        }

        let row_count = all_rows.len();
        Some(ParsedPage {
            columns,
            dtypes,
            rows: all_rows,
            row_count,
        })
    }

    fn apply_open_result(&mut self, result: OpenResult) {
        let page = self
            .facade
            .get_chunk(&result.dataset_name, 0, self.page_size as u32)
            .ok()
            .and_then(|bytes| Self::parse_ipc_to_page(&bytes));

        self.steps_entries = self.facade.get_steps(&result.dataset_name);
        self.steps_active_index = None;
        self.selected_cell = None;

        self.tables = self.facade.list_datasets();
        self.name = Some(result.dataset_name);
        self.columns = result.columns;
        self.total_rows = result.total_rows;
        self.size_bytes = result.size_bytes;
        self.current_page = page;
        self.offset = 0;
        self.sort_column = None;
        self.sort_desc = false;
        self.persistent = result.persistent;
        self.loading = false;
        self.error = None;
    }

    pub fn new_project(&mut self, path: &str) {
        self.loading = true;
        self.error = None;
        match self.facade.new_project(path) {
            Ok(info) => {
                self.project = Some(info.clone());
                self.tables = info.tables;
                self.name = None;
                self.columns = vec![];
                self.total_rows = 0;
                self.current_page = None;
                self.offset = 0;
                self.sort_column = None;
                self.sort_desc = false;
                self.loading = false;
            }
            Err(e) => {
                self.loading = false;
                self.error = Some(e);
            }
        }
    }

    pub fn open_project(&mut self, path: &str) {
        self.loading = true;
        self.error = None;
        match self.facade.open_project(path) {
            Ok(info) => {
                self.project = Some(info.clone());
                self.tables = info.tables;
                self.name = None;
                self.columns = vec![];
                self.total_rows = 0;
                self.current_page = None;
                self.offset = 0;
                self.sort_column = None;
                self.sort_desc = false;
                self.loading = false;
            }
            Err(e) => {
                self.loading = false;
                self.error = Some(e);
            }
        }
    }

    pub fn open_file(&mut self, path: &str) {
        self.loading = true;
        self.error = None;
        self.last_failed_action = None;
        match self.facade.open_file(path) {
            Ok(result) => self.apply_open_result(result),
            Err(e) => {
                self.loading = false;
                self.error = Some(e);
                self.last_failed_action = Some(FailedAction::OpenFile(path.to_string()));
            }
        }
    }

    pub fn import_file(&mut self, path: &str) {
        self.loading = true;
        self.error = None;
        self.last_failed_action = None;
        match self.facade.import_file(path, None) {
            Ok(result) => self.apply_open_result(result),
            Err(e) => {
                self.loading = false;
                self.error = Some(e);
                self.last_failed_action = Some(FailedAction::ImportFile(path.to_string()));
            }
        }
    }

    pub fn select_table(&mut self, table_name: &str) {
        self.loading = true;
        self.error = None;
        let page = self
            .facade
            .get_chunk(table_name, 0, self.page_size as u32)
            .ok()
            .and_then(|bytes| Self::parse_ipc_to_page(&bytes));

        if let Some(p) = page {
            self.name = Some(table_name.to_string());
            self.columns = p
                .columns
                .iter()
                .zip(p.dtypes.iter())
                .map(|(n, d)| ColumnInfo {
                    name: n.clone(),
                    dtype: d.clone(),
                })
                .collect();
            let fallback_count = p.row_count;
            self.current_page = Some(p);
            self.offset = 0;
            self.sort_column = None;
            self.sort_desc = false;
            self.persistent = true;
            self.loading = false;
            self.total_rows = self
                .facade
                .get_row_count(table_name)
                .unwrap_or(fallback_count);
        } else {
            self.loading = false;
            self.error = Some(format!("Failed to load table: {}", table_name));
        }
        self.tables = self.facade.list_datasets();
    }

    pub fn load_page(&mut self, offset: usize) {
        if let Some(ref name) = self.name.clone() {
            self.loading = true;
            match self
                .facade
                .get_chunk(name, offset as u32, self.page_size as u32)
            {
                Ok(bytes) => {
                    if let Some(page) = Self::parse_ipc_to_page(&bytes) {
                        self.current_page = Some(page);
                        self.offset = offset;
                    }
                    self.loading = false;
                }
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                }
            }
        }
    }

    pub fn sort_by(&mut self, column: &str) {
        if let Some(ref name) = self.name.clone() {
            let desc = if self.sort_column.as_deref() == Some(column) {
                !self.sort_desc
            } else {
                false
            };
            self.loading = true;
            self.error = None;
            self.last_failed_action = None;
            let col = column.to_string();
            match self.facade.sort_dataset(name, &[column], &[desc]) {
                Ok(result) => {
                    self.apply_open_result(result);
                    self.sort_column = Some(col);
                    self.sort_desc = desc;
                }
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                    self.last_failed_action = Some(FailedAction::Sort(column.to_string()));
                }
            }
        }
    }

    pub fn execute_sql(&mut self) {
        let sql = self.sql_text.trim().to_string();
        if sql.is_empty() {
            return;
        }
        self.loading = true;
        self.error = None;
        self.last_failed_action = None;
        match self.facade.execute_sql(&sql) {
            Ok(result) => self.apply_open_result(result),
            Err(e) => {
                self.loading = false;
                self.error = Some(e);
                self.last_failed_action = Some(FailedAction::ExecuteSql(sql));
            }
        }
    }

    pub fn export_dataset(&mut self, output_path: &str, format: &str) {
        if let Some(ref name) = self.name.clone() {
            if let Err(e) = self.facade.export_dataset(name, output_path, format) {
                self.error = Some(e);
            }
        }
    }

    pub fn remove_dataset(&mut self, dataset_name: &str) {
        if let Err(e) = self.facade.remove_dataset(dataset_name) {
            self.error = Some(e);
            return;
        }
        self.tables = self.facade.list_datasets();
        if self.name.as_deref() == Some(dataset_name) {
            self.name = None;
            self.columns = vec![];
            self.total_rows = 0;
            self.current_page = None;
        }
    }

    pub fn apply_structured_filter(&mut self) {
        use core_engine::{FilterCondition, FilterLogic};
        use crate::facade::parse_filter_operator;

        let op = match parse_filter_operator(&self.struct_filter_operator) {
            Ok(op) => op,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        let condition = FilterCondition {
            column: self.struct_filter_column.clone(),
            operator: op,
            value: self.struct_filter_value.clone(),
        };
        if let Some(ref name) = self.name.clone() {
            self.loading = true;
            self.error = None;
            self.last_failed_action = None;
            match self.facade.filter_dataset_structured(
                name,
                vec![condition],
                FilterLogic::And,
            ) {
                Ok(result) => {
                    self.apply_open_result(result);
                    self.filter_dialog = false;
                    self.struct_filter_value.clear();
                }
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                }
            }
        }
    }

    pub fn filter_dataset_sql(&mut self) {
        let where_clause = self.filter_where.trim().to_string();
        if where_clause.is_empty() {
            return;
        }
        if let Some(ref name) = self.name.clone() {
            let dataset = name.clone();
            self.loading = true;
            self.error = None;
            self.last_failed_action = None;
            match self.facade.filter_dataset_sql(name, &where_clause) {
                Ok(result) => {
                    self.apply_open_result(result);
                    self.filter_dialog = false;
                    self.filter_where.clear();
                }
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                    self.last_failed_action =
                        Some(FailedAction::FilterSql(dataset, where_clause));
                }
            }
        }
    }

    pub fn group_by_dataset(&mut self) {
        let cols_str = self.group_cols_input.trim().to_string();
        let aggs_str = self.group_aggs_input.trim().to_string();
        if cols_str.is_empty() || aggs_str.is_empty() {
            return;
        }
        if let Some(ref name) = self.name.clone() {
            let group_cols: Vec<&str> = cols_str.split(',').map(|s| s.trim()).collect();
            let agg_exprs: Vec<&str> = aggs_str.split(',').map(|s| s.trim()).collect();
            self.loading = true;
            self.error = None;
            self.last_failed_action = None;
            match self.facade.group_by(name, &group_cols, &agg_exprs) {
                Ok(result) => {
                    self.apply_open_result(result);
                    self.group_dialog = false;
                    self.group_cols_input.clear();
                    self.group_aggs_input.clear();
                }
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                    self.last_failed_action =
                        Some(FailedAction::GroupBy(cols_str, aggs_str));
                }
            }
        }
    }

    pub fn add_calculated_column(&mut self) {
        let expr = self.calc_expr.trim().to_string();
        let alias = self.calc_alias.trim().to_string();
        if expr.is_empty() || alias.is_empty() {
            return;
        }
        if let Some(ref name) = self.name.clone() {
            self.loading = true;
            self.error = None;
            self.last_failed_action = None;
            match self.facade.add_calculated_column(name, &expr, &alias) {
                Ok(result) => {
                    self.apply_open_result(result);
                    self.calc_dialog = false;
                    self.calc_expr.clear();
                    self.calc_alias.clear();
                }
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                    self.last_failed_action =
                        Some(FailedAction::CalcColumn(expr, alias));
                }
            }
        }
    }

    pub fn generate_chart(&mut self) {
        if let Some(ref name) = self.name.clone() {
            let value_col = if self.chart_agg_type == "count" {
                None
            } else {
                Some(self.chart_value_col.as_str())
            };
            self.chart_error = None;
            match self.facade.aggregate_for_chart(
                name,
                &self.chart_group_col,
                value_col,
                &self.chart_agg_type,
                self.chart_limit,
            ) {
                Ok(bytes) => {
                    if let Some(page) = Self::parse_ipc_to_page(&bytes) {
                        let data: Vec<ChartRow> = page
                            .rows
                            .iter()
                            .map(|r| ChartRow {
                                label: r.first().cloned().unwrap_or_default(),
                                value: r
                                    .get(1)
                                    .and_then(|v| v.parse::<f64>().ok())
                                    .unwrap_or(0.0),
                            })
                            .collect();
                        self.chart_data = Some(data);
                    }
                }
                Err(e) => {
                    self.chart_error = Some(e);
                }
            }
        }
    }

    pub fn load_summary_stats(&mut self) {
        if let Some(ref name) = self.name.clone() {
            match self.facade.get_summary_stats(name) {
                Ok(bytes) => {
                    self.stats_data = Self::parse_ipc_to_page(&bytes);
                    self.stats_visible = true;
                }
                Err(e) => {
                    self.error = Some(e);
                }
            }
        }
    }

    // Phase 1: Step navigation
    pub fn select_step(&mut self, index: usize) {
        if let Some(entry) = self.steps_entries.get(index) {
            let table_name = entry.result_table.clone();
            self.steps_active_index = Some(index);
            self.selected_cell = None;

            let page = self
                .facade
                .get_chunk(&table_name, 0, self.page_size as u32)
                .ok()
                .and_then(|bytes| Self::parse_ipc_to_page(&bytes));

            if let Some(p) = page {
                self.columns = p
                    .columns
                    .iter()
                    .zip(p.dtypes.iter())
                    .map(|(n, d)| ColumnInfo {
                        name: n.clone(),
                        dtype: d.clone(),
                    })
                    .collect();
                let fallback = p.row_count;
                self.current_page = Some(p);
                self.offset = 0;
                self.name = Some(table_name.clone());
                self.total_rows = self
                    .facade
                    .get_row_count(&table_name)
                    .unwrap_or(fallback);
            }
        }
    }

    // Phase 2: Ingest preview
    pub fn start_ingest(&mut self, file_path: &str) {
        self.ingest_file_path = file_path.to_string();
        self.ingest_table_name.clear();
        self.ingest_dialog = true;

        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext == "tsv" {
            self.ingest_delimiter = b'\t';
        } else {
            self.ingest_delimiter = b',';
        }
        self.ingest_has_header = true;
        self.ingest_skip_rows = 0;
        self.preview_ingest();
    }

    pub fn preview_ingest(&mut self) {
        match self.facade.preview_file(
            &self.ingest_file_path,
            self.ingest_delimiter,
            self.ingest_has_header,
            self.ingest_skip_rows,
        ) {
            Ok(bytes) => {
                self.ingest_preview = Self::parse_ipc_to_page(&bytes);
            }
            Err(e) => {
                self.error = Some(e);
                self.ingest_preview = None;
            }
        }
    }

    pub fn commit_ingest(&mut self) {
        let table_name = if self.ingest_table_name.trim().is_empty() {
            None
        } else {
            Some(self.ingest_table_name.trim())
        };
        self.loading = true;
        self.error = None;
        match self.facade.import_file_with_options(
            &self.ingest_file_path.clone(),
            table_name,
            self.ingest_delimiter,
            self.ingest_has_header,
            self.ingest_skip_rows,
        ) {
            Ok(result) => {
                self.ingest_dialog = false;
                self.ingest_preview = None;
                self.apply_open_result(result);
            }
            Err(e) => {
                self.loading = false;
                self.error = Some(e);
            }
        }
    }

    // Phase 3: Column operations
    pub fn remove_column(&mut self, col_name: &str) {
        if let Some(ref name) = self.name.clone() {
            self.loading = true;
            self.error = None;
            match self.facade.remove_columns(name, &[col_name]) {
                Ok(result) => self.apply_open_result(result),
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                }
            }
        }
    }

    pub fn keep_columns_action(&mut self, cols: Vec<String>) {
        if let Some(ref name) = self.name.clone() {
            let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
            self.loading = true;
            self.error = None;
            match self.facade.keep_columns(name, &col_refs) {
                Ok(result) => self.apply_open_result(result),
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                }
            }
        }
    }

    pub fn change_column_type_action(&mut self, column: &str, new_type: &str) {
        if let Some(ref name) = self.name.clone() {
            self.loading = true;
            self.error = None;
            match self.facade.change_column_type(name, column, new_type) {
                Ok(result) => self.apply_open_result(result),
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                }
            }
        }
    }

    pub fn rename_column_action(&mut self) {
        let old = self.rename_old_name.trim().to_string();
        let new = self.rename_new_name.trim().to_string();
        if old.is_empty() || new.is_empty() {
            self.error = Some("Both old and new column names are required".into());
            return;
        }
        if let Some(ref name) = self.name.clone() {
            self.loading = true;
            self.error = None;
            self.rename_dialog = false;
            match self.facade.rename_column(name, &old, &new) {
                Ok(result) => self.apply_open_result(result),
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                }
            }
        }
    }

    pub fn pivot_dataset(&mut self) {
        let idx_str = self.pivot_index_cols.trim().to_string();
        let pcol = self.pivot_col.trim().to_string();
        let vcol = self.pivot_value_col.trim().to_string();
        let agg = self.pivot_agg.trim().to_string();
        if idx_str.is_empty() || pcol.is_empty() || vcol.is_empty() {
            return;
        }
        if let Some(ref name) = self.name.clone() {
            let index_cols: Vec<&str> = idx_str.split(',').map(|s| s.trim()).collect();
            self.loading = true;
            self.error = None;
            match self
                .facade
                .pivot_dataset(name, &index_cols, &pcol, &vcol, &agg)
            {
                Ok(result) => {
                    self.pivot_dialog = false;
                    self.apply_open_result(result);
                }
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                }
            }
        }
    }

    pub fn unpivot_dataset(&mut self) {
        let vcols_str = self.unpivot_value_cols.trim().to_string();
        let var = self.unpivot_var_name.trim().to_string();
        let val = self.unpivot_value_name.trim().to_string();
        if vcols_str.is_empty() {
            return;
        }
        if let Some(ref name) = self.name.clone() {
            let value_cols: Vec<&str> = vcols_str.split(',').map(|s| s.trim()).collect();
            self.loading = true;
            self.error = None;
            match self.facade.unpivot_dataset(name, &value_cols, &var, &val) {
                Ok(result) => {
                    self.unpivot_dialog = false;
                    self.apply_open_result(result);
                }
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                }
            }
        }
    }

    // Phase 3: Merge / Append
    pub fn merge_datasets(&mut self) {
        let right = self.merge_right_table.trim().to_string();
        let lcol = self.merge_left_col.trim().to_string();
        let rcol = self.merge_right_col.trim().to_string();
        let jtype = self.merge_join_type.clone();
        if right.is_empty() || lcol.is_empty() || rcol.is_empty() {
            return;
        }
        if let Some(ref name) = self.name.clone() {
            self.loading = true;
            self.error = None;
            match self
                .facade
                .merge_datasets(name, &right, &lcol, &rcol, &jtype)
            {
                Ok(result) => {
                    self.merge_dialog = false;
                    self.apply_open_result(result);
                }
                Err(e) => {
                    self.loading = false;
                    self.error = Some(e);
                }
            }
        }
    }

    pub fn append_datasets(&mut self) {
        let input = self.append_tables_input.trim().to_string();
        if input.is_empty() {
            return;
        }
        let table_names: Vec<&str> = input.split(',').map(|s| s.trim()).collect();
        self.loading = true;
        self.error = None;
        match self.facade.append_datasets(&table_names) {
            Ok(result) => {
                self.append_dialog = false;
                self.apply_open_result(result);
            }
            Err(e) => {
                self.loading = false;
                self.error = Some(e);
            }
        }
    }

    pub fn load_merge_right_columns(&mut self) {
        if !self.merge_right_table.is_empty() {
            self.merge_right_columns = self
                .facade
                .get_columns(&self.merge_right_table)
                .unwrap_or_default();
        }
    }

    pub fn dismiss_error(&mut self) {
        self.error = None;
    }

    pub fn retry_last_action(&mut self) {
        let action = match self.last_failed_action.take() {
            Some(a) => a,
            None => return,
        };
        match action {
            FailedAction::OpenFile(path) => self.open_file(&path),
            FailedAction::ImportFile(path) => self.import_file(&path),
            FailedAction::Sort(col) => self.sort_by(&col),
            FailedAction::ExecuteSql(sql) => {
                self.sql_text = sql;
                self.execute_sql();
            }
            FailedAction::FilterSql(_, where_clause) => {
                self.filter_where = where_clause;
                self.filter_dataset_sql();
            }
            FailedAction::GroupBy(cols, aggs) => {
                self.group_cols_input = cols;
                self.group_aggs_input = aggs;
                self.group_by_dataset();
            }
            FailedAction::CalcColumn(expr, alias) => {
                self.calc_expr = expr;
                self.calc_alias = alias;
                self.add_calculated_column();
            }
        }
    }

    pub fn has_retry(&self) -> bool {
        self.last_failed_action.is_some()
    }
}

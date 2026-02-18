# RUSTORA MASTER ENHANCEMENT PLAN
## Unified Data Platform: Excel + Power BI + Power Query + SSMS

**Date:** February 17, 2026  
**Version:** 2.0
**Status:** PLANNING - Ready for Implementation

---

## VISION

Build a high-performance, local-first unified data platform combining:
- **Excel** - Data exploration and spreadsheet-like interface  
- **Power Query** - ETL and data transformation
- **Power BI** - Analytics and visualizations
- **SSMS** - SQL querying and database management

---

## PART 1: DATA CONNECTIVITY

### 1.1 File Sources (HIGH Priority)

| Source | Format | Status | Implementation |
|--------|--------|--------|----------------|
| CSV/TSV | Text | ✅ EXISTS | DuckDB read_csv |
| Parquet | Columnar | ✅ EXISTS | DuckDB read_parquet |
| Arrow IPC | Binary | ✅ EXISTS | DuckDB/Polars |
| Feather | Binary | ✅ EXISTS | Polars |
| JSON | Text | ❌ | DuckDB read_json |
| JSON Lines | Text | ❌ | DuckDB read_ndjson |
| Excel (.xlsx) | Spreadsheet | ❌ | calamine crate |
| Excel (.xls) | Spreadsheet | ❌ | calamine crate |
| Avro | Binary | ❌ | apache_avro crate |
| ORC | Binary | ❌ | polars-orc plugin |
| XML | Text | ❌ | polars-xml |
| HDF5 | Binary | ❌ | hdf5 crate |

**Implementation Notes:**
- JSON: Use DuckDB's `read_json` and `read_ndjson` functions
- Excel: Add `calamine` crate for .xlsx/.xls parsing via Polars
- Avro: Use `apache_avro` crate or Polars plugin
- ORC: Use `polars-orc` crate

### 1.2 Database Sources (HIGH Priority)

| Database | Status | Implementation | Priority |
|----------|--------|----------------|----------|
| SQLite | ✅ EXISTS | DuckDB native | HIGH |
| DuckDB | ✅ EXISTS | Native | HIGH |
| PostgreSQL | ❌ | postgres + sqlx | HIGH |
| MySQL/MariaDB | ❌ | mysql crate | HIGH |
| SQL Server | ❌ | tds crate | MEDIUM |
| Oracle | ❌ | oracle-db crate | LOW |
| BigQuery | ❌ | google-cloud-bigquery | LOW |
| Snowflake | ❌ | snowflake-sdk | LOW |
| Redshift | ❌ | postgres (Redshift mode) | LOW |
| SQLite (external) | ❌ | rusqlite + connection | MEDIUM |

**Implementation Notes:**
- Use connection pool pattern for database connections
- Store connection configs encrypted in project file
- Support both read and write operations where applicable

### 1.3 API Sources (MEDIUM Priority)

| Source | Status | Implementation | Priority |
|--------|--------|----------------|----------|
| REST API (generic) | ❌ | reqwest + Polars | MEDIUM |
| GraphQL | ❌ | graphql-client | LOW |
| WebSocket (streaming) | ❌ | tokio-tungstenite | LOW |
| OData | ❌ | odata crate | LOW |

---

## PART 2: DATA EXPLORATION (Excel-like)

### 2.1 Grid Enhancements (HIGH Priority)

| Feature | Status | Description |
|---------|--------|-------------|
| **Cell Selection** | ❌ | Click to select, shift+click for range |
| **Row Selection** | ❌ | Click row header to select |
| **Column Selection** | ❌ | Click column header to select |
| **Multi-cell Selection** | ❌ | Click and drag for range |
| **Copy/Paste** | ❌ | Ctrl+C/V with clipboard integration |
| **Find/Replace** | ❌ | Ctrl+F for find, Ctrl+H for replace |
| **Go To Row** | ❌ | Ctrl+G to jump to row number |
| **Freeze Panes** | ❌ | Freeze rows/columns |
| **Auto-fit Column Width** | ❌ | Double-click column border |
| **Manual Column Resize** | ❌ | Drag column borders |
| **Column Reorder** | ❌ | Drag-drop columns |
| **Row Numbers** | ✅ EXISTS | Already implemented |
| **Column Type Indicators** | ❌ | Icons for string/int/date/bool |
| **NULL Value Display** | ✅ EXISTS | Shows "NULL" |
| **Quick Info Tooltips** | ❌ | Hover for column stats |
| **Keyboard Navigation** | ❌ | Arrow keys in grid |
| **Scroll to Selection** | ❌ | Scroll to selected cell |

### 2.2 Navigation (HIGH Priority)

| Feature | Status | Description |
|---------|--------|-------------|
| **Keyboard Navigation** | ❌ | Arrow keys move cell focus |
| **Go To Row** | ❌ | Go to specific row number |
| **Scroll to Selection** | ❌ | Auto-scroll to selected cell |
| **Jump to First/Last Row** | ❌ | Ctrl+Home/End |
| **Page Up/Down** | ❌ | Standard paging |

### 2.3 Multi-Sheet Workbook (HIGH Priority)

| Feature | Status | Description |
|---------|--------|-------------|
| **Sheet Tabs** | ❌ | Bottom tab bar with sheet names |
| **Add Sheet** | ❌ | + button creates new sheet |
| **Rename Sheet** | ❌ | Double-click tab to rename |
| **Delete Sheet** | ❌ | Right-click menu with confirmation |
| **Reorder Sheets** | ❌ | Drag-drop tabs |
| **Copy/Move Sheet** | ❌ | Right-click options |
| **Sheet Protection** | ❌ | Password protection |
| **Duplicate Sheet** | ❌ | Copy entire sheet |

**Data Model:**
```typescript
interface Sheet {
  id: string;
  name: string;
  sourceTable: string | null;
  transformSteps: TransformStep[];
  pivotConfig?: PivotConfig;
  chartConfigs?: ChartConfig[];
  columnWidths: Record<string, number>;
  frozenColumns: number;
  frozenRows: number;
}
```

### 2.4 Data Entry (MEDIUM Priority)

| Feature | Status | Description |
|---------|--------|-------------|
| **Edit Cells** | ❌ | Double-click to edit |
| **Add Rows** | ❌ | Insert new rows |
| **Delete Rows** | ❌ | Delete selected rows |
| **Auto-fill** | ❌ | Drag handle for series |
| **Undo/Redo Edit** | ❌ | Ctrl+Z/Y for cell edits |

---

## PART 3: DATA TRANSFORMATION (Power Query-like)

### 3.1 Transform Panel UI (HIGH Priority)

Create a "Power Query Editor" panel with step-by-step transformations:

```
┌─────────────────────────────────────────────────────────────┐
│  Query Settings                            Applied Steps     │
│  ┌─────────────────────────────┐          ┌──────────────┐  │
│  │ Name: customers             │          │ ◉ Source    │  │
│  │ Description: Imported from │          │ ◉ Filter     │  │
│  │           CSV file          │          │ ◉ Type Change│  │
│  │ Source: customers.csv      │          │ ◉ Group By   │  │
│  └─────────────────────────────┘          │ ◉ Sort      │  │
│                                           └──────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Grid Preview (data changes with each selected step)        │
│  ┌─────┬─────────┬─────┬────────┬─────────────────────────┐ │
│  │ ID  │ Name    │ Age │ Sales │ City                   │ │
│  ├─────┼─────────┼─────┼────────┼─────────────────────────┤ │
│  │ 1   │ Alice   │ 30  │ 1000  │ New York               │ │
│  │ 2   │ Bob     │ 25  │ 1500  │ San Francisco          │ │
│  └─────┴─────────┴─────┴────────┴─────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  Transform Tab: [Filter] [Sort] [Group] [Merge] [Pivot]    │
└─────────────────────────────────────────────────────────────┘
```

**Key Features:**
- Applied Steps panel (left sidebar)
- Each step is clickable and reversible
- Preview updates in real-time
- Step reordering via drag-drop
- Disable/enable individual steps
- Step descriptions

### 3.2 Transformation Operations (HIGH Priority)

| Operation | Status | SQL Equivalent | UI Element |
|-----------|--------|----------------|------------|
| **Filter Rows** | ❌ | WHERE | Dropdown with conditions |
| **Remove Duplicates** | ❌ | DISTINCT | Column selector |
| **Remove Columns** | ❌ | SELECT col1, col2 | Multi-select list |
| **Keep Columns** | ❌ | SELECT | Multi-select list |
| **Rename Column** | ❌ | AS | Text input |
| **Reorder Columns** | ❌ | SELECT (ordered) | Drag-drop list |
| **Change Type** | ❌ | CAST | Dropdown (Int/String/Date/etc) |
| **Split Column** | ❌ | SUBSTRING, SPLIT | Delimiter input |
| **Merge Columns** | ❌ | CONCATENATE | Column selector + separator |
| **Replace Values** | ❌ | REPLACE | Find/Replace inputs |
| **Fill Down/Up** | ❌ | LAST_VALUE | Direction selector |
| **Trim/Clean** | ❌ | TRIM | Checkbox options |
| **Extract** | ❌ | SUBSTRING | Pattern input |
| **Pivot** | ❌ | PIVOT | Column selectors |
| **Unpivot** | ❌ | UNPIVOT | Column selectors |
| **Transpose** | ❌ | Manual matrix | Button |
| **Reverse Rows** | ❌ | ORDER BY DESC | Button |
| **Add Index Column** | ❌ | ROW_NUMBER | Start number input |

### 3.3 Aggregation Operations (HIGH Priority)

| Operation | Status | Description | UI |
|-----------|--------|-------------|----|
| **Group By** | ❌ | GROUP BY with aggregations | Column + function selector |
| **Sum** | ❌ | SUM(column) | Part of Group By |
| **Average** | ❌ | AVG(column) | Part of Group By |
| **Count** | ❌ | COUNT(column) | Part of Group By |
| **Count Distinct** | ❌ | COUNT(DISTINCT) | Part of Group By |
| **Min/Max** | ❌ | MIN, MAX | Part of Group By |
| **Standard Deviation** | ❌ | STDDEV | Part of Group By |
| **Custom Aggregation** | ❌ | Any aggregate expression | Expression input |

### 3.4 Table Operations (HIGH Priority)

| Operation | Status | Join Type | UI |
|-----------|--------|-----------|-----|
| **Join Tables** | ❌ | INNER | Table selector + key columns + type |
| **Join Tables** | ❌ | LEFT | Table selector + key columns + type |
| **Join Tables** | ❌ | RIGHT | Table selector + key columns + type |
| **Join Tables** | ❌ | FULL | Table selector + key columns + type |
| **Join Tables** | ❌ | CROSS | Table selector |
| **Append Tables** | ❌ | UNION ALL | Multi-table selector |
| **Union** | ❌ | UNION | Multi-table selector |
| **Difference** | ❌ | EXCEPT | Table selector |
| **Intersection** | ❌ | INTERSECT | Table selector |

### 3.5 Advanced Transforms (MEDIUM Priority)

| Operation | Status | Description |
|-----------|--------|-------------|
| **Custom Column** | ❌ | Calculated columns with expressions |
| **Conditional Column** | ❌ | IF-THEN-ELSE UI |
| **Date Operations** | ❌ | Extract year/month/day/hour |
| **Text Operations** | ❌ | Upper, Lower, Proper case, Trim |
| **Normalize** | ❌ | Min-max scaling to 0-1 |
| **One-Hot Encode** | ❌ | Create dummy variables |
| **Bin/Quantile** | ❌ | Create bins from continuous values |

---

## PART 4: DATA ANALYSIS (Power BI-like)

### 4.1 Calculated Fields (MEDIUM Priority)

| Feature | Status | Description |
|---------|--------|-------------|
| **Calculated Columns** | ❌ | New columns with formulas |
| **DAX-like Expressions** | ❌ | Expression parser |
| **Quick Measures** | ❌ | Pre-built calculations |
| **Measures** | ❌ | Aggregations that calculate dynamically |

### 4.2 Visualizations (MEDIUM Priority)

| Chart Type | Status | Library | Priority |
|------------|--------|---------|----------|
| **Bar Chart** | ❌ | recharts | HIGH |
| **Horizontal Bar** | ❌ | recharts | HIGH |
| **Line Chart** | ❌ | recharts | HIGH |
| **Area Chart** | ❌ | recharts | MEDIUM |
| **Scatter Plot** | ❌ | recharts | HIGH |
| **Bubble Chart** | ❌ | recharts | MEDIUM |
| **Pie Chart** | ❌ | recharts | MEDIUM |
| **Donut Chart** | ❌ | recharts | MEDIUM |
| **Histogram** | ❌ | visx | MEDIUM |
| **Box Plot** | ❌ | visx | LOW |
| **Heatmap** | ❌ | visx | LOW |
| **Treemap** | ❌ | visx | LOW |
| **Sankey** | ❌ | d3-sankey | LOW |

### 4.3 Analytics (LOW Priority)

| Feature | Status | Description |
|---------|--------|-------------|
| **Trend Lines** | ❌ | Linear regression overlay |
| **Forecasting** | ❌ | Time series prediction |
| **Clustering** | ❌ | K-means clustering |
| **Correlation Matrix** | ❌ | Heatmap of correlations |

---

## PART 5: SQL EDITOR (SSMS-like)

### 5.1 Query Editor Enhancements (HIGH Priority)

| Feature | Status | Description |
|---------|--------|-------------|
| **Syntax Highlighting** | ❌ | Monaco Editor integration |
| **Auto-complete** | ❌ | Table/column names |
| **Error Highlighting** | ❌ | SQL syntax errors |
| **Query Formatting** | ❌ | Pretty print SQL |
| **Multiple Tabs** | ❌ | Multiple query tabs |
| **Query History** | ❌ | Recent queries list |
| **Save Queries** | ❌ | Save to .sql files |
| **Load Queries** | ❌ | Load from .sql files |
| **Query Templates** | ❌ | Reusable snippets |

### 5.2 Query Execution (HIGH Priority)

| Feature | Status | Description |
|---------|--------|-------------|
| **Execute Query** | ✅ EXISTS | Run SQL |
| **Execute Selection** | ❌ | Run only selected text |
| **Execution Plan** | ❌ | EXPLAIN ANALYZE |
| **Query Timing** | ❌ | Execution duration display |
| **Row Count Limit** | ❌ | SET LIMIT option |
| **Query Cancellation** | ❌ | Cancel running query |
| **Batch Separator** | ❌ | GO statement support |

### 5.3 Database Management (MEDIUM Priority)

| Feature | Status | Description |
|---------|--------|-------------|
| **Schema Browser** | ❌ | Tree view of tables/views |
| **Table Designer** | ❌ | Create/modify tables UI |
| **Index Management** | ❌ | CREATE INDEX UI |
| **View Editor** | ❌ | CREATE VIEW UI |
| **Foreign Keys** | ❌ | Relationship editor UI |
| **Stored Procedures** | ❌ | List and execute |

---

## PART 6: PROJECT & STATE MANAGEMENT

### 6.1 Data Model (HIGH Priority)

```typescript
interface Project {
  id: string;
  name: string;
  path: string;
  version: string;
  created: Date;
  modified: Date;
  settings: ProjectSettings;
  sheets: Sheet[];
  connections: Connection[];
  queries: SavedQuery[];
  transformPipelines: TransformPipeline[];
}

interface Sheet {
  id: string;
  name: string;
  order: number;
  sourceTable: string | null;
  sourceConnection: string | null;
  transformSteps: TransformStep[];
  pivotConfig?: PivotConfig;
  chartConfigs?: ChartConfig[];
  columnWidths: Record<string, number>;
  columnOrder: string[];
  frozenColumns: number;
  frozenRows: number;
  visible: boolean;
}

interface TransformStep {
  id: string;
  name: string;
  type: TransformType;
  params: Record<string, any>;
  enabled: boolean;
  order: number;
}

interface Connection {
  id: string;
  name: string;
  type: 'file' | 'database' | 'api';
  config: ConnectionConfig;
  encryptedCredentials?: string;
}

interface SavedQuery {
  id: string;
  name: string;
  sql: string;
  connectionId?: string;
  created: Date;
  modified: Date;
}
```

### 6.2 Undo/Redo System (HIGH Priority)

| Feature | Status | Description |
|---------|--------|-------------|
| **Transformation Steps** | ❌ | Edit applied steps |
| **Undo** | ❌ | Ctrl+Z |
| **Redo** | ❌ | Ctrl+Y |
| **Step Reordering** | ❌ | Drag steps to reorder |
| **Step Deletion** | ❌ | Remove transformation step |
| **Step Modification** | ❌ | Edit step parameters |

### 6.3 State Management (HIGH Priority)

**Current:** React useState + useCallback

**Recommended:** Zustand + TanStack Query

| Package | Purpose | Priority |
|---------|---------|----------|
| `zustand` | Global UI state | HIGH |
| `@tanstack/react-query` | Server state, caching | HIGH |
| `immer` | Immutable updates | MEDIUM |

**Why Zustand:**
- Simpler than Context API
- No providers needed
- Built-in devtools
- TypeScript-friendly

**Why TanStack Query:**
- Automatic caching
- Background refetch
- Optimistic updates
- Query invalidation

---

## PART 7: PERFORMANCE OPTIMIZATIONS

### 7.1 Critical Fixes (CRITICAL)

| Issue | Location | Impact | Fix |
|-------|----------|--------|-----|
| **O(n²) Arrow Parsing** | arrow.ts:22-28 | CRITICAL | Column-wise processing |
| **Mutex Panic** | session.rs:76 | CRITICAL | Proper error handling |
| **Repeated list_tables()** | session.rs | HIGH | Add caching |
| **No Error Boundaries** | Frontend | HIGH | React error boundaries |

**Arrow Parsing Fix:**
```typescript
// Current (O(n²)): Row-by-row
for (let i = 0; i < table.numRows; i++) {
  const row = {};
  for (const col of columns) {
    row[col] = table.getChild(col)?.get(i);
  }
  rows.push(row);
}

// Fixed (O(n)): Column-wise
const rows = [];
for (let i = 0; i < table.numRows; i++) {
  rows.push({});
}
for (const col of columns) {
  const vector = table.getChild(col);
  for (let i = 0; i < table.numRows; i++) {
    rows[i][col] = vector?.get(i);
  }
}
```

### 7.2 Advanced Optimizations (MEDIUM)

| Optimization | Status | Description |
|--------------|--------|-------------|
| **Web Workers** | ❌ | Background data processing |
| Virtual Scrolling | ✅ EXISTS | TanStack Virtual |
| **Query Caching** | ❌ | Cache SQL results |
| **IndexedDB Cache** | ❌ | Browser-side cache |
| **Lazy Column Load** | ❌ | Load columns on demand |
| **WASM Processing** | ❌ | Rust in browser |
| **Connection Pooling** | ❌ | DuckDB connection pool |

---

## PART 8: IMPLEMENTATION ROADMAP

### Phase 1: Foundation (Week 1-2) 🔴 CRITICAL
- [ ] Fix Arrow parsing O(n²) → O(n) performance
- [ ] Fix mutex panic risk in session.rs
- [ ] Add React error boundaries
- [ ] Add Zustand for state management
- [ ] Add TanStack Query for server state
- [ ] Add basic error handling improvements

**Files to modify:**
- `desktop_ui/src/lib/arrow.ts`
- `core_engine/src/session.rs`
- `desktop_ui/src/App.tsx` (wrap in error boundary)

### Phase 2: Excel-like Features (Week 3-4)
- [ ] Multi-sheet workbook
  - Add sheet state to data model
  - Add sheet tabs UI
  - Add/rename/delete sheet functionality
- [ ] Cell selection and ranges
- [ ] Copy/paste support
- [ ] Column resize/reorder (drag-drop)
- [ ] Quick filter dropdowns in column headers
- [ ] Keyboard navigation (arrows, Ctrl+Home/End)

**New files:**
- `desktop_ui/src/components/SheetTabs.tsx`
- `desktop_ui/src/components/FilterDropdown.tsx`

### Phase 3: Power Query Features (Week 5-8)
- [ ] Transform panel UI
  - Applied Steps sidebar
  - Step selection and preview
- [ ] Filter rows UI
  - Column dropdown with unique values
  - Condition builder (equals, contains, greater than, etc.)
- [ ] Group By UI
  - Column selector for grouping
  - Aggregation function selector
- [ ] Join tables UI
  - Table selector
  - Key column mapping
  - Join type selector (inner/left/right/full)
- [ ] Append tables UI
  - Multi-table selector
  - Column alignment options
- [ ] Pivot/Unpivot UI
  - Column selectors for pivot/unpivot
- [ ] Undo/redo for transformation steps

**New files:**
- `desktop_ui/src/components/TransformPanel.tsx`
- `desktop_ui/src/components/AppliedSteps.tsx`
- `desktop_ui/src/components/FilterDialog.tsx`
- `desktop_ui/src/components/GroupByDialog.tsx`
- `desktop_ui/src/components/JoinDialog.tsx`
- `desktop_ui/src/components/AppendDialog.tsx`
- `desktop_ui/src/components/PivotDialog.tsx`

### Phase 4: SQL Editor (Week 9-10)
- [ ] Monaco Editor integration
  - SQL syntax highlighting
  - Auto-complete for table/column names
- [ ] Query tabs (multiple queries)
- [ ] Query history panel
- [ ] Save/load queries
- [ ] Execution plan view

**New files:**
- `desktop_ui/src/components/QueryEditor.tsx`
- `desktop_ui/src/components/QueryHistory.tsx`

### Phase 5: Visualizations (Week 11-12)
- [ ] Chart container component
- [ ] Bar chart
- [ ] Line chart
- [ ] Scatter plot
- [ ] Pie chart
- [ ] Chart configuration panel

**New files:**
- `desktop_ui/src/components/Charts.tsx`
- `desktop_ui/src/components/ChartBuilder.tsx`

### Phase 6: Data Sources (Week 13+)
- [ ] JSON import
- [ ] Excel import (.xlsx)
- [ ] Database connectors (PostgreSQL, MySQL)
- [ ] REST API connector
- [ ] Connection manager UI

**Backend changes:**
- Add `calamine` crate for Excel
- Add database connector crates

---

## PART 9: TECHNICAL STACK ADDITIONS

### Frontend Dependencies

| Package | Purpose | Version | Priority |
|---------|---------|---------|----------|
| `@monaco-editor/react` | SQL editor | ^5.0.0 | HIGH |
| `zustand` | State management | ^5.0.0 | HIGH |
| `@tanstack/react-query` | Server state | ^5.0.0 | HIGH |
| `recharts` | Charts | ^2.0.0 | MEDIUM |
| `visx` | Low-level charts | ^3.0.0 | MEDIUM |
| `react-dnd` | Drag and drop | ^17.0.0 | MEDIUM |
| `immer` | Immutable state | ^10.0.0 | MEDIUM |
| `react-error-boundary` | Error boundaries | ^5.0.0 | HIGH |
| `date-fns` | Date handling | ^4.0.0 | LOW |

### Backend New Commands

```rust
// core_engine/src/session.rs - New methods

// Transformations
fn filter_dataset_sql(&self, name: &str, predicate: &str) -> Result<String>
fn deduplicate(&self, name: &str, columns: &[&str]) -> Result<String>
fn split_column(&self, name: &str, column: &str, delimiter: &str) -> Result<String>
fn merge_columns(&self, name: &str, columns: &[&str], separator: &str) -> Result<String>
fn pivot_table(&self, name: &str, index: &str, columns: &str, values: &str) -> Result<String>
fn unpivot_table(&self, name: &str, id_columns: &[&str], value_columns: &[&str]) -> Result<String>
fn join_tables(&self, left: &str, right: &str, on_left: &str, on_right: &str, how: &str) -> Result<String>
fn union_tables(&self, names: &[&str]) -> Result<String>
fn group_by_agg(&self, name: &str, group_cols: &[&str], aggs: HashMap<&str, &str>) -> Result<String>
fn change_type(&self, name: &str, column: &str, dtype: &str) -> Result<String>
fn fill_down(&self, name: &str, columns: &[&str]) -> Result<String>
fn add_index(&self, name: &str, offset: u64) -> Result<String>

// Data Import
fn import_json(&mut self, file_path: &str, table_name: Option<&str>) -> Result<String>
fn import_excel(&mut self, file_path: &str, sheet: Option<&str>, table_name: Option<&str>) -> Result<String>

// Project
fn save_transform_pipeline(&self, pipeline: &TransformPipeline) -> Result<()>
fn load_transform_pipeline(&self, id: &str) -> Result<TransformPipeline>
```

### Tauri Commands (src-tauri/src/lib.rs)

```rust
#[tauri::command]
fn filter_dataset(state: State<'_, AppState>, name: String, predicate: String) -> Result<OpenResult, String>

#[tauri::command]
fn deduplicate(state: State<'_, AppState>, name: String, columns: Vec<String>) -> Result<OpenResult, String>

#[tauri::command]
fn join_tables(state: State<'_, AppState>, left: String, right: String, on_left: String, on_right: String, how: String) -> Result<OpenResult, String>

#[tauri::command]
fn union_tables(state: State<'_, AppState>, names: Vec<String>) -> Result<OpenResult, String>

#[tauri::command]
fn group_by(state: State<'_, AppState>, name: String, group_cols: Vec<String>, aggs: HashMap<String, String>) -> Result<OpenResult, String>

#[tauri::command]
fn pivot(state: State<'_, AppState>, name: String, index: String, columns: String, values: String) -> Result<OpenResult, String>

// Add to invoke_handler!
```

---

## PART 10: SUCCESS CRITERIA

### Performance Targets

| Metric | Target | Test Method |
|--------|--------|-------------|
| Load 1M row CSV | < 2 seconds | Benchmark |
| Filter 10M rows | < 500ms | Benchmark |
| UI scrolling | 60fps | Chrome DevTools |
| Memory usage | < 2GB for 10M rows | Chrome DevTools |
| Startup time | < 3 seconds | Manual |

### Feature Completeness Targets

| Category | Target |
|----------|--------|
| File formats | CSV, Parquet, JSON, Excel |
| Transforms | All basic transforms |
| SQL Editor | Syntax highlighting + autocomplete |
| Charts | Bar, Line, Scatter, Pie |
| Database | PostgreSQL, MySQL support |

### User Experience Targets

| Feature | Requirement |
|---------|-------------|
| Keyboard shortcuts | Ctrl+C/V/F/Z/Y throughout |
| Error messages | Clear, actionable |
| Loading states | Skeleton/spinner for all operations |
| Tooltips | Help text for all features |
| Accessibility | Keyboard navigable |

---

## PART 11: FILE STRUCTURE CHANGES

### New Directory Structure

```
desktop_ui/src/
├── components/
│   ├── DataGrid/
│   │   ├── DataGrid.tsx
│   │   ├── Cell.tsx
│   │   ├── ColumnHeader.tsx
│   │   ├── FilterDropdown.tsx
│   │   └── SelectionManager.ts
│   ├── Sheets/
│   │   ├── SheetTabs.tsx
│   │   └── SheetManager.ts
│   ├── Transform/
│   │   ├── TransformPanel.tsx
│   │   ├── AppliedSteps.tsx
│   │   ├── FilterDialog.tsx
│   │   ├── GroupByDialog.tsx
│   │   ├── JoinDialog.tsx
│   │   ├── AppendDialog.tsx
│   │   └── PivotDialog.tsx
│   ├── Query/
│   │   ├── QueryEditor.tsx
│   │   ├── QueryTabs.tsx
│   │   └── QueryHistory.tsx
│   ├── Charts/
│   │   ├── ChartContainer.tsx
│   │   ├── BarChart.tsx
│   │   ├── LineChart.tsx
│   │   └── ChartConfig.tsx
│   └── common/
│       ├── Modal.tsx
│       ├── Button.tsx
│       └── Input.tsx
├── hooks/
│   ├── useDataset.ts (existing)
│   ├── useProject.ts
│   ├── useTransform.ts
│   ├── useQuery.ts
│   └── useKeyboard.ts
├── store/
│   ├── projectStore.ts
│   ├── queryClient.ts
│   └── transformStore.ts
├── lib/
│   ├── arrow.ts (fix)
│   ├── transformations.ts
│   └── expressions.ts
└── types/
    └── index.ts
```

---

*Document Version: 2.0*
*Last Updated: February 17, 2026*
*Status: READY FOR IMPLEMENTATION*

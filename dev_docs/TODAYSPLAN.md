# Rustora MVP Review & Enhancement Plan

This document serves as a comprehensive review of the Rustora application and outlines a plan to elevate it from an MVP to an extremely polished, high-performance, and robust product.

## General Architecture Assessment
The architecture is solid:
- **`core_engine`**: The use of DuckDB for persistent storage and Polars for transient/in-memory computation is an excellent choice for a data analysis tool. Returning Arrow IPC bytes ensures zero-copy (or minimized copy) data serialization, avoiding the massive overhead of JSON stringification.
- **`desktop_ui`**: Wrapping the tool in Tauri with a React frontend utilizing `@tanstack/react-virtual` ensures the data grid can scale to large datasets without choking the DOM.
- **`python_api`**: Exposing the core engine via PyO3 allows for great interoperability with the Python data science ecosystem.

## Proposed Enhancements & Best Practices

> **Priority Order**: Sections are numbered by recommended execution order. Security and correctness (1, 4) should precede cosmetic and ergonomic work (2, 3), which should precede operational maturity (5–8). Estimated effort per section: **S** (small, < half day), **M** (medium, ~1 day), **L** (large, 1–2 days).

### 1. Core Engine — SQL Hardening `[Priority: HIGH | Effort: M]`
#### [MODIFY] `core_engine/src/filter.rs` and `core_engine/src/storage.rs`
- **Current State**: SQL queries are generated via extensive string formatting (e.g., `format!("SELECT * FROM {} WHERE {}", table, condition)`). While some basic escaping is performed via `escape_sql_string`, this approach is susceptible to SQL injection and does not take advantage of DuckDB's query plan caching for parameterized queries.
- **Note**: The desktop UI already mitigates this via the `filter_dataset_structured` Tauri command, which builds filters programmatically without raw SQL. The raw SQL path remains exposed through the SQL panel in the desktop UI and the Python API's `filter_sql()` / `execute_sql()` methods — those are the primary hardening targets.
- **Enhancement**: 
  - Refactor data access methods to use **parameterized queries** for values (e.g., `WHERE age > ?`). 
  - Implement a more rigorous validation for table and column names since they cannot be parameterized as easily.
  - Switch from manual string escaping to structural SQL building or DuckDB's prepared statements.
  - Add new unit tests in `filter.rs` and `storage.rs` specifically covering the parameterized query paths and edge cases (empty strings, Unicode, extremely long values).

### 2. Desktop UI — Visual & UX Overhaul `[Priority: MEDIUM | Effort: L]`
#### [MODIFY] `desktop_ui/src/App.css` and UI Components
- **Current State**: The UI is functional but relies on rudimentary CSS. The styling is basic and lacks the "premium vibe" expected from modern professional tools.
- **Enhancement**: 
  - **Aesthetics & Theme**: Overhaul `App.css` to introduce a sleek, modern design system. Implement a refined color palette (e.g., elegant dark mode with glassmorphism effects), smooth gradients, modern typography (like Inter or Roboto), and dynamic micro-animations for hover states and transitions.
  - **UX Polish**: Improve the "Empty State" UI when no data is loaded, add better visual feedback when loading large files or executing long-running SQL queries.
  - **Accessibility (a11y)**: Ensure full keyboard navigation for the data grid, menus, and `FilterPopover`. Add proper ARIA attributes (`role`, `aria-label`, `aria-live` for status updates) to all interactive elements. Verify sufficient color contrast ratios (WCAG AA minimum), especially if introducing a dark theme with glassmorphism.

### 3. Python API — Ergonomics & Type Safety `[Priority: MEDIUM | Effort: M]`
#### [NEW] `python_api/python/rustora/__init__.py` and `rustora.pyi`
- **Current State**: The PyO3 module exposes Rust classes directly to Python. While perfectly functional, Python users heavily rely on type linting and autocompletion in their IDEs (VSCode/PyCharm), which is missing. The API returns raw Arrow IPC bytes, requiring the user to know how to parse them.
- **Enhancement**:
  - Add native Python wrapper code or a `.pyi` type stub file to provide robust type hints and docstrings.
  - Introduce helper utilities in Python that automatically parse the Arrow IPC bytes into a Polars DataFrame (`polars.read_ipc`) or PyArrow Table, drastically improving usability for data scientists.
  - Expand `test_smoke.py` into a proper test suite covering sort, filter, structured filter, export (CSV and Parquet), edge cases (empty datasets, missing columns), and the new helper utilities.

### 4. Tauri Layer — Structured Error Propagation `[Priority: HIGH | Effort: S]`
#### [MODIFY] `desktop_ui/src-tauri/src/lib.rs` and Frontend Error Handling
- **Current State**: Tauri commands convert `RustoraError` into opaque strings before sending them to the frontend. The React side receives a plain string and can only display it verbatim — there is no way to distinguish a "file not found" from a "SQL syntax error" programmatically.
- **Enhancement**:
  - Implement `serde::Serialize` on a Tauri-facing error struct (or directly on `RustoraError`) so the frontend receives structured JSON errors with fields like `code`, `category`, and `message`.
  - Update `ErrorBoundary.tsx` and `StatusBar.tsx` to render contextual error UI based on error category (e.g., recoverable vs. fatal, user-facing vs. internal).

### 5. Observability — Structured Logging `[Priority: MEDIUM | Effort: S]`
#### [NEW] `tracing` integration across `core_engine` and `desktop_ui/src-tauri`
- **Current State**: No structured logging exists anywhere in the codebase. Diagnosing slow imports, failed queries, or memory issues requires manual debugging.
- **Enhancement**:
  - Add the `tracing` and `tracing-subscriber` crates to the workspace dependencies.
  - Instrument key operations in `core_engine` (file import, SQL execution, export) with `tracing::info!` / `tracing::warn!` spans that include timing and row counts.
  - Initialize a subscriber in the Tauri entry point (`main.rs`) with configurable log levels (e.g., `RUST_LOG=rustora=debug`).

### 6. CI/CD Pipeline `[Priority: HIGH | Effort: M]`
#### [NEW] `.github/workflows/ci.yml`
- **Current State**: No CI/CD configuration exists. All testing is manual.
- **Enhancement**:
  - Create a GitHub Actions workflow that runs on every push and PR:
    - `cargo fmt --check` and `cargo clippy -- -D warnings` across all workspace members.
    - `cargo test` for `core_engine` (unit + integration tests).
    - `maturin develop && python test_smoke.py` for `python_api`.
    - `npm ci && npm run build` in `desktop_ui` to verify the frontend compiles.
  - Optionally add a matrix build for Windows, macOS, and Linux to catch platform-specific issues early.

### 7. Build & Distribution `[Priority: LOW | Effort: M]`
#### [MODIFY] `desktop_ui/src-tauri/tauri.conf.json`
- **Current State**: The Tauri config is development-oriented. No production build configuration, app icons, code signing, or auto-updater setup exists.
- **Enhancement**:
  - Configure `tauri.conf.json` for production builds with proper app metadata (name, version, description, icons).
  - Set up platform-specific bundling: MSI (Windows), DMG (macOS), AppImage/deb (Linux).
  - Investigate Tauri's built-in updater plugin for future OTA update support.

### 8. Performance Baselines `[Priority: MEDIUM | Effort: S]`
#### [NEW] Benchmark suite
- **Current State**: No performance benchmarks exist. There is no way to detect performance regressions introduced by refactoring.
- **Enhancement**:
  - Add a `benches/` directory in `core_engine` using Criterion.rs with benchmarks for:
    - CSV/Parquet import at varying row counts (10k, 100k, 1M rows).
    - SQL query execution (simple filter, aggregation, join).
    - Arrow IPC serialization for large result sets.
  - Record baseline numbers before starting any refactoring work.
  - Integrate benchmark runs into CI as a non-blocking informational step.

---

## Verification Plan

### Automated Tests — Per Section

| Section | Command | What to Verify |
|---------|---------|----------------|
| 1. SQL Hardening | `cargo test -p core_engine` | All existing tests pass + new parameterized query tests pass. |
| 3. Python API | `cd python_api && maturin develop && python -m pytest test_smoke.py -v` | Smoke tests + new expanded tests for sort, filter, export, edge cases. |
| 4. Error Propagation | `cargo test -p desktop-ui-tauri` | Structured error serialization round-trips correctly. |
| 5. Logging | `RUST_LOG=rustora=debug cargo test -p core_engine` | Tracing output appears during test runs without panics. |
| 8. Benchmarks | `cargo bench -p core_engine` | Criterion baselines are recorded; no panics or timeouts. |

### Frontend Tests
- Add Vitest component tests for `DataGrid.tsx`, `FilterPopover.tsx`, and `SqlPanel.tsx` (`npm run test` in `desktop_ui`).
- After CSS changes, verify the virtual scroller still renders correctly at 100k+ rows (no layout thrashing or blank cells).

### Security Regression
- Execute SQL injection payloads through both the SQL panel and the Python API's `execute_sql()`:
  - `'; DROP TABLE users; --`
  - `1; SELECT * FROM information_schema.tables`
  - Unicode/null-byte edge cases: `\0`, `%00`, multi-byte sequences.
- Confirm all are rejected or safely escaped.

### Manual Verification
- Launch the Tauri app (`npm run tauri dev` in `desktop_ui`).
- Import a test CSV and verify:
  - New aesthetics render correctly (theme, typography, animations).
  - Keyboard navigation works through the grid, menus, and filter popover.
  - Error states display structured messages (e.g., import a nonexistent file, run invalid SQL).
  - Status bar and logs reflect `tracing` output where applicable.
- Test on at least **two platforms** (e.g., Windows + one of macOS/Linux) to catch platform-specific rendering or path issues.

### Performance Regression Checklist
- [ ] Import 1M-row CSV: confirm time is within ±10% of pre-refactor baseline.
- [ ] Render first visible grid chunk: confirm < 200ms after import completes.
- [ ] SQL aggregation on 1M rows: confirm time is within ±10% of baseline.
- [ ] Memory footprint during import: confirm no unexpected growth vs. baseline.

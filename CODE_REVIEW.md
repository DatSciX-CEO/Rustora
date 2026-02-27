# Rustora Code Review & Technical Guide

Welcome to the Rustora codebase! This document provides a comprehensive overview of the architecture, logic, and file-by-file responsibilities of the Rustora application. It's designed to help new developers onboard quickly and understand how the different pieces of the stack interact.

## 🏗️ Architecture Overview

Rustora is built on a high-performance, local-first stack consisting of three primary layers:

1.  **Core Engine (Rust):** The brain of the application. Handles data ingestion, SQL execution, transformation, and Arrow IPC serialization. It utilizes DuckDB for persistent project storage and SQL querying, and Polars for fast transient memory operations.
2.  **Desktop UI (Tauri + React/TypeScript):** The presentation layer. Tauri v2 provides the native desktop shell and bridges the frontend to the Rust core engine. The React 19 frontend uses a virtualized grid to render potentially millions of rows without performance degradation.
3.  **Python API (PyO3):** A Python wrapper allowing programmatic access to the `core_engine` functionality via Python bindings.

Crucially, **no data is passed between the backend and frontend as JSON**. Instead, the application relies on **Zero-JSON Arrow IPC** binary transfer (`Vec<u8>`), which allows zero-copy parsing in the browser and maintains high performance for massive datasets.

---

## 🗺️ Codebase Matrix: Files, Logic, and Responsibilities

The following matrix details every critical folder and file in the application, specifically outlining its location, what the code does, and its core logic. This serves as a rapid lookup map for humans and AI agents navigating the repository.

### 1. The Core Engine (`core_engine/`)
The pure Rust backend. UI-agnostic data engine responsible for ingestion, querying, and serializing data to Arrow IPC.

| Location | File Name | Purpose & Details | Core Logic & Responsibilities |
| :--- | :--- | :--- | :--- |
| `core_engine/` | `Cargo.toml` | **Dependency & Build Config** | Defines crate dependencies: `duckdb`, `polars`, `arrow-ipc`, `thiserror`, `tracing`. |
| `core_engine/src/` | `lib.rs` | **API Exporter** | Crate facade. Re-exports core structs like `RustoraSession` and `DuckStorage` for external use (like Tauri and Python bindings). |
| `core_engine/src/` | `session.rs` | **Session Orchestrator** | **Logic:** Manages dual backends (`DuckStorage` for persistent `.duckdb` projects and `Polars LazyFrames` for transient memory).<br>**Roles:** Project creation, file ingestion delegation, routing SQL queries, data grouping/filtering, and converting `Polars DataFrame` into Arrow IPC bytes `Vec<u8>`. |
| `core_engine/src/` | `storage.rs` | **DuckDB Persistence Layer** | **Logic:** Wraps a `duckdb::Connection`. Executes native SQL queries and utilizes DuckDB's high-speed readers (CSV, Parquet).<br>**Roles:** Importing files, creating persistent tables, streaming `query_arrow()` results directly into an Arrow IPC `StreamWriter` without hitting JSON. |
| `core_engine/src/` | `filter.rs` | **Type-safe SQL Filter Builder** | **Logic:** Takes structured filter specs (e.g., Column `Age` > 30) and transforms them into secure, escaped SQL `WHERE` clauses.<br>**Roles:** Preventing SQL injection vulnerabilities when UI filters are translated into DuckDB queries. |
| `core_engine/src/` | `error.rs` | **Standardized Error Handling** | **Logic:** Implements `thiserror` mapping for graceful error cascades.<br>**Roles:** Translates deep system errors (Polars, DuckDB, IO) into clean, handleable `RustoraError` enums for the UI/API. |

### 2. Desktop UI & Backend Bridge (`desktop_ui/`)
The native desktop shell and React presentation layer. Uses Tauri for system access and frontend hosting.

| Location | File Name | Purpose & Details | Core Logic & Responsibilities |
| :--- | :--- | :--- | :--- |
| `desktop_ui/src-tauri/src/` | `main.rs` | **Tauri Entry Point** | **Logic:** Bootstraps the Tauri runtime, registers plugins, and starts the app window. |
| `desktop_ui/src-tauri/src/` | `lib.rs` | **Tauri Command Layer (IPC Bridge)** | **Logic:** Exposes ~18 `#[tauri::command]` asynchronous functions.<br>**Roles:** Wraps `RustoraSession` in a thread-safe `Arc<Mutex<>>`. Deserializes UI requests, spawns blocking threads (`spawn_blocking`) to run Rust heavy lifting without freezing Tauri, and passes `Vec<u8>` or structured responses back. |
| `desktop_ui/src/` | `App.tsx` & `main.tsx` | **React App Root** | **Logic:** Renders the main shell, initializing the application layout, CSS, and primary component tree. |
| `desktop_ui/src/hooks/` | `useDataset.ts` | **Central State Management** | **Logic:** The primary hook that talks to Tauri. Controls active datasets, current page chunks, and UI loading states.<br>**Roles:** Manages `invoke()` calls, coordinates offset pagination for the virtualized grid, and employs request deduplication to prevent stale-data race conditions. |
| `desktop_ui/src/lib/` | `arrow.ts` | **Arrow IPC Decoder** | **Logic:** Intercepts the raw `Uint8Array` binary streams coming from Tauri.<br>**Roles:** Parses the bytes using `apache-arrow` JS and converts them into an iterable, columnar format that the React DataGrid can consume instantly. |
| `desktop_ui/src/lib/` | `error.ts` | **Frontend Error Parser** | **Logic:** Contains discriminated union types to parse structured error JSON from Tauri.<br>**Roles:** Translates backend failures into user-friendly UI toast/banner messages. |
| `desktop_ui/src/components/` | `DataGrid.tsx` | **Virtualized Data Table** | **Logic:** Utilizes `@tanstack/react-virtual` for Row & Column virtualization.<br>**Roles:** Renders only visible rows on the screen. Ensures 60fps scrolling even if the underlying table has 50 Million rows. |
| `desktop_ui/src/components/` | `SqlPanel.tsx` | **Interactive SQL Editor** | **Logic:** Text area that sends raw strings to the `execute_sql` Tauri command and triggers grid reloads with the resulting table data. |
| `desktop_ui/src/components/` | `MenuBar.tsx` | **Action Menu & Dialogs** | **Logic:** Houses the main toolbar. Also contains embedded logic/dialogs for triggering Filters, Group By, Column Math, and Export operations. |
| `desktop_ui/src/components/` | `TableList.tsx` | **Sidebar Table Browser** | **Logic:** Lists imported datasets (both transient and persistent) allowing users to switch active context. |
| `desktop_ui/src/components/` | `ChartPanel.tsx` | **Visualization Builder** | **Logic:** Uses `recharts` to render visual aggregates generated via DuckDB `aggregate_for_chart` logic. |

### 3. Python API Bindings (`python_api/`)
Maturin/PyO3 crate allowing native Python scripts to directly call the `core_engine`.

| Location | File Name | Purpose & Details | Core Logic & Responsibilities |
| :--- | :--- | :--- | :--- |
| `python_api/src/` | `lib.rs` | **PyO3 Bindings** | **Logic:** Wraps `RustoraSession` using `#[pyclass]` and `#[pymethods]`.<br>**Roles:** Exposes Rust functions to Python. Translates native Rust types into zero-copy `PyBytes` objects, drastically speeding up data handoffs to Pandas or PyArrow. |
| `python_api/` | `rustora.pyi` | **Python Type Stubs** | **Logic:** Python signature file.<br>**Roles:** Provides IDE autocomplete, strict typing, and documentation for the Python interface without needing to parse the Rust library. |
| `python_api/` | `test_smoke.py` | **Validation Scripts** | **Logic:** Minimal scripts to ensure the compiled `.whl` bindings load correctly and basic functions (import, query, IPC) execute natively. |
| `python_api/` | `pyproject.toml` | **Maturin Build Config** | **Logic:** Configures how the Rust crate compiles into a Python extension module. |


---

## 🔄 Data Flow Summary

To understand how everything connects, trace a standard operation (e.g., viewing a dataset):

1.  **User Action:** User clicks a dataset in `TableList.tsx`.
2.  **Frontend State:** `useDataset.ts` dispatches a `get_chunk` Tauri command with the offset/limit of visible rows.
3.  **Tauri Bridge:** `src-tauri/src/lib.rs` receives the call and spawns a blocking thread to execute `session.get_chunk_ipc()`.
4.  **Core Engine:** `DuckStorage` prepares a `SELECT ... LIMIT ... OFFSET` SQL statement, executes it via DuckDB, and serializes the result into an Arrow IPC byte array (`Vec<u8>`).
5.  **Return Path:** The binary array is passed back through Tauri to the frontend.
6.  **Frontend Render:** `lib/arrow.ts` parses the binary array. `DataGrid.tsx` consumes the parsed table and updates the virtualized DOM.

This end-to-end pipeline operates efficiently with absolutely zero JSON serialization of table row data, representing Rustora's defining architectural feature.
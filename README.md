<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust"/>
  <img src="https://img.shields.io/badge/Tauri_v2-24C8D8?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri"/>
  <img src="https://img.shields.io/badge/React_19-61DAFB?style=for-the-badge&logo=react&logoColor=black" alt="React"/>
  <img src="https://img.shields.io/badge/DuckDB-FFF000?style=for-the-badge&logo=duckdb&logoColor=black" alt="DuckDB"/>
  <img src="https://img.shields.io/badge/Apache_Arrow-0052CC?style=for-the-badge&logo=apache&logoColor=white" alt="Arrow"/>
</p>

<h1 align="center">🦀 Rustora</h1>

<p align="center">
  <strong>Blazingly fast, 100% local data analysis on your desktop.</strong><br/>
  <em>Think Excel + Power Query + SSMS — but built in Rust and designed for datasets that don't fit in RAM.</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-active-brightgreen?style=flat-square" alt="Status"/>
  <img src="https://img.shields.io/badge/tests-20%20passing-brightgreen?style=flat-square" alt="Tests"/>
  <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License"/>
  <img src="https://img.shields.io/badge/telemetry-none-critical?style=flat-square" alt="No Telemetry"/>
</p>

---

## 🚀 Why Rustora?

Most data tools force you into one of two bad tradeoffs: **cloud-dependent** SaaS that ships your data to someone else's servers, or **slow desktop apps** that choke on anything over a few hundred thousand rows.

Rustora takes a different path:

| Problem | Rustora's Answer |
|:---|:---|
| Excel crashes on 1M+ rows | Processes **10GB–50GB+** datasets without breaking a sweat |
| Power BI requires cloud sync | **100% air-gapped** — your data never leaves your machine |
| Python/Pandas loads everything into RAM | **Out-of-core processing** via lazy evaluation and streaming |
| JSON serialization bottlenecks | **Zero-copy Arrow IPC** binary transfer — no serialization overhead |
| SSMS requires a dedicated server | **Embedded DuckDB** — full SQL engine, zero configuration |

---

## ✨ Key Features

🗄️ **Hybrid Storage Engine**
DuckDB for persistent, ACID-compliant project storage. Polars for lightning-fast transient computations. The best of both worlds.

⚡ **Handles Massive Datasets**
Lazy evaluation and chunked streaming mean you can query a 50GB Parquet file on a laptop with 16GB of RAM.

🔒 **Zero-Copy Data Pipeline**
All tabular data moves between Rust and the UI as raw Apache Arrow IPC bytes (`Vec<u8>`). **JSON is banned** for data transfer — this is what makes Rustora feel native.

🐘 **Full SQL Engine**
Write SQL queries directly against your tables powered by DuckDB's production-grade query engine. `SELECT`, `JOIN`, `GROUP BY`, window functions — it's all there.

💾 **Project Persistence**
Save your work as `.duckdb` project files. Tables, query results, and imported data persist across sessions.

🖥️ **Modern Desktop UI**
Tauri v2 shell with a React 19 frontend. Virtualized data grid renders only the rows you can see — smooth scrolling through millions of rows.

📤 **Export Anywhere**
One-click export to **CSV** or **Parquet** formats.

🛡️ **100% Local & Private**
No cloud APIs. No telemetry. No analytics. No phone-home. Completely air-gapped by design.

---

## 🏗️ Architecture

Rustora is built as a **Rust Cargo workspace** with strict layer separation. The core data engine knows nothing about the UI — it can be reused independently by other Rust projects or future Python bindings.

```
┌─────────────────────────────────────────────────────────────┐
│                    React Frontend (Vite)                     │
│        Virtualized Grid  ·  SQL Panel  ·  Toolbar           │
└────────────────────────────┬────────────────────────────────┘
                             │ Tauri invoke()
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                   Tauri v2 Command Layer                     │
│           11 commands · State management · IPC bridge        │
└────────────────────────────┬────────────────────────────────┘
                             │ &mut RustoraSession
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                 core_engine (Pure Rust)                      │
│  ┌──────────────────┐      ┌───────────────────────────┐    │
│  │  DuckDB Storage   │      │  Polars LazyFrame Engine   │    │
│  │  • Persistent     │      │  • Transient scans         │    │
│  │  • SQL engine     │      │  • Sort / Filter / Slice   │    │
│  │  • ACID-compliant │      │  • Out-of-core streaming   │    │
│  └────────┬─────────┘      └─────────────┬─────────────┘    │
│           │                              │                   │
│           ▼                              ▼                   │
│       Arrow RecordBatch          Polars DataFrame            │
│           │                              │                   │
│           └──────────┐    ┌──────────────┘                   │
│                      ▼    ▼                                  │
│              Arrow IPC Stream Bytes (Vec<u8>)                │
│                    ZERO JSON · ZERO COPY                     │
└─────────────────────────────────────────────────────────────┘
```

### Workspace Members

| Crate | Role | Key Dependencies |
|:---|:---|:---|
| **`core_engine`** | Pure Rust data engine — storage, compute, serialization | Polars 0.46, DuckDB 1.4, Arrow IPC |
| **`desktop_ui`** | Tauri v2 desktop application with React frontend | Tauri 2, React 19, Vite, @tanstack/react-virtual |
| **`python_api`** | Future PyO3/Maturin Python bindings | *(placeholder)* |

---

## 🛠️ Project Structure & Error Handling

### Why are there multiple `Cargo.toml` files?
Rustora is organized as a **Rust Workspace**. This is the idiomatic way to manage a multi-component project where different parts have different build requirements:
- **`core_engine`**: A library crate containing the core logic.
- **`desktop_ui/src-tauri`**: A binary crate that builds the desktop application. Tauri expects its own configuration here.
- **`python_api`**: A special `cdylib` crate for Python bindings.
- **Root `Cargo.toml`**: The "single source of truth" that manages shared dependency versions (like `polars` or `serde`) across all members.

### Robust Error Handling
You may notice that SQL syntax errors or missing files do not crash the application. This is due to a multi-layered defensive strategy:
1. **Rust Result Pattern**: The backend uses the `Result` type to return errors as data rather than panicking.
2. **Tauri Bridge**: Errors are serialized into structured JSON and sent to the frontend.
3. **React State**: The `useDataset` hook captures these errors in a `try...catch` block and updates the UI state, allowing the app to remain interactive while displaying a helpful message to the user.

---

## 📦 Data Flow

### Ingest → Query → Render

```
CSV / Parquet / Arrow file
        │
        ▼
   DuckDB read_csv() / read_parquet()    ←── Persistent table created
        │
        ▼
   SQL Query (DuckDB engine)
        │
        ▼
   Arrow RecordBatch[]
        │
        ▼
   IPC Stream Bytes (binary)             ←── Sent to frontend via Tauri
        │
        ▼
   apache-arrow (JS) tableFromIPC()
        │
        ▼
   Virtualized Grid (only visible rows)  ←── Smooth 60fps scrolling
```

### Two Modes of Operation

| Mode | When | Storage | Persistence |
|:---|:---|:---|:---|
| **Project Mode** | Create/open a `.duckdb` file | DuckDB tables | ✅ Data survives across sessions |
| **Scratch Mode** | Open files without a project | Polars LazyFrames | ❌ Data discarded on close |

---

## 🛠️ Getting Started

### Prerequisites

| Requirement | Version | Install |
|:---|:---|:---|
| **Rust** | stable | [rustup.rs](https://www.rust-lang.org/tools/install) |
| **Node.js** | 18+ | [nodejs.org](https://nodejs.org/) |
| **C++ Build Tools** | — | Windows: VS Build Tools ("Desktop development with C++" workload) · macOS: Xcode CLI · Linux: `build-essential` |
| **Tauri Dependencies** | — | [Tauri v2 Prerequisites](https://v2.tauri.app/start/prerequisites/) |

### Quick Start (Windows)

The fastest way to get up and running:

```powershell
git clone https://github.com/protoxx/rustora.git
cd rustora
.\setup.ps1
```

The `setup.ps1` script checks prerequisites, installs dependencies, and launches the app. Pass `-Build` to produce a release `.exe` instead of starting the dev server.

### Manual Install & Run

```bash
# Clone the repository
git clone https://github.com/protoxx/rustora.git
cd rustora

# Install frontend dependencies
cd desktop_ui
npm install

# Launch in development mode
# (Compiles the Rust backend + starts Vite dev server + opens the app)
npm run tauri dev
```

> **First launch note:** The initial build compiles the entire Rust backend including DuckDB (C++ source) and Polars. This can take **10–15 minutes** depending on your machine. Subsequent launches are fast thanks to incremental compilation.

### Build for Release

To create the optimized `Rustora.exe` executable:

```bash
cd desktop_ui
npm run tauri build
```

The build process will produce:
- **Executable:** `../target/release/Rustora.exe`
- **Installer (MSI):** `../target/release/bundle/msi/`

> **Note:** The `target` directory is excluded from version control (git-ignored). When cloning this repository on a new machine, you **must run the build command** to generate the executable.

### Build Without npm (Cargo-Only)

If your environment doesn't have npm/Node.js, you can build the release `.exe` using **only Rust and Cargo**. The repository includes a pre-built frontend (`desktop_ui/dist/`) so no JavaScript tooling is required.

```powershell
# One command — builds Rustora.exe + MSI installer
.\build_no_npm.ps1
```

Or manually:

```bash
# 1. Install the Tauri CLI (one-time)
cargo install tauri-cli

# 2. Build using the standalone config (skips npm, uses pre-built frontend)
cd desktop_ui
cargo tauri build --config src-tauri/tauri.standalone.conf.json
```

The standalone config tells Tauri to use the committed `dist/` folder directly instead of invoking npm. The resulting `.exe` bundles the frontend into the binary — **no localhost, no dev server, no Node.js needed at runtime**.

> **Requirements:** Rust (stable) and C++ Build Tools. Node.js is **not** needed.

### Alternative: Using a Different JS Package Manager

If you want the full dev workflow but prefer an npm alternative:

```bash
# 1. Install the Tauri CLI (one-time)
cargo install tauri-cli

# 2. Install frontend dependencies with an npm alternative:
cd desktop_ui
yarn install        # https://yarnpkg.com
pnpm install        # https://pnpm.io
bun install         # https://bun.sh

# 3. Launch in development mode
cargo tauri dev

# 4. Build the release .exe
cargo tauri build
```

### Troubleshooting

| Symptom | Cause | Fix |
|:---|:---|:---|
| `localhost refused to connect` | Vite dev server didn't start (missing `node_modules`) | Use `.\build_no_npm.ps1` for cargo-only builds, or run `npm install` inside `desktop_ui/` for dev mode |
| `npm install` at repo root does nothing | There is no `package.json` at the root — this is a Rust workspace | `cd desktop_ui` then `npm install` |
| Build fails with missing icons | Icons directory not generated | Icons are committed to the repo; if missing, run `npx tauri icon <source.png>` inside `desktop_ui/` |
| Link errors on Windows | Missing C++ build tools | Install VS Build Tools with "Desktop development with C++" workload |
| Exe shows blank window | Frontend not bundled | Use `.\build_no_npm.ps1` or ensure `desktop_ui/dist/` exists before `cargo tauri build` |

### Run Tests

```bash
# From the workspace root — runs all 20 core_engine tests
cargo test -p core_engine
```

**Test coverage includes:**
- DuckDB storage operations (import, query, export, persistence across sessions)
- Session management (project lifecycle, persistent + transient dataset listing)
- Arrow IPC serialization (chunked pagination, previews)
- SQL execution and result materialization
- Sort, filter, and export workflows
- Error handling (unsupported formats, missing files)

---

## 💡 Usage Guide

### Create a Project
Click **New** → choose a location → Rustora creates a `.duckdb` project file. All imported data persists here.

### Import Data
Click **Import** → select CSV, TSV, Parquet, or Arrow IPC files → data is ingested into DuckDB as a persistent table.

### Browse & Navigate
Imported tables appear in the **sidebar**. Click any table to load it into the virtualized grid. Scroll through millions of rows seamlessly.

### Write SQL
Toggle the **SQL panel** → write queries against your tables → press **Ctrl+Enter** to execute.

```sql
SELECT department, AVG(salary) as avg_salary, COUNT(*) as headcount
FROM employees
WHERE hire_date > '2020-01-01'
GROUP BY department
ORDER BY avg_salary DESC
```

### Sort
Click any **column header** to sort ascending. Click again to toggle descending.

### Export
Use the **CSV** or **Parquet** toolbar buttons to export the current dataset to disk.

---

## 🔧 Tech Stack

| Layer | Technology | Why |
|:---|:---|:---|
| **Storage** | DuckDB 1.4 (bundled) | ACID-compliant, embedded OLAP database with native Parquet/CSV readers |
| **Compute** | Polars 0.46 | Lazy evaluation, out-of-core streaming, multi-threaded execution |
| **Serialization** | Apache Arrow IPC | Zero-copy binary transfer — the industry standard for columnar data |
| **Desktop Shell** | Tauri v2 | Lightweight native shell (~5MB), no Electron bloat |
| **Frontend** | React 19 + TypeScript + Vite | Modern, fast, type-safe UI development |
| **Grid** | @tanstack/react-virtual | Virtualizes rows so only visible data is rendered |
| **Arrow Parsing** | apache-arrow (JS) | Decodes IPC bytes directly in the browser — column-wise, zero-copy |

---

## 📁 Project Structure

```
rustora/
├── Cargo.toml                      # Workspace root
├── CODE_REVIEW.md                  # 📖 Deep-dive into application architecture, file logic, and code
├── core_engine/                    # 🧠 THE BRAIN — Pure Rust data engine
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Public API exports
│       ├── session.rs              # RustoraSession: orchestrates all operations
│       ├── storage.rs              # DuckStorage: DuckDB persistence layer
│       └── error.rs                # Error types (Polars, DuckDB, IO)
├── desktop_ui/                     # 🖥️ THE GUI — Tauri v2 + React
│   ├── src-tauri/
│   │   └── src/
│   │       ├── lib.rs              # Tauri commands bridging UI ↔ core_engine
│   │       └── main.rs             # Entry point
│   ├── src/
│   │   ├── App.tsx                 # Root component
│   │   ├── App.css                 # Application styles
│   │   ├── components/
│   │   │   ├── DataGrid.tsx        # Virtualized data grid
│   │   │   ├── Toolbar.tsx         # File/project/export actions
│   │   │   ├── SqlPanel.tsx        # SQL query editor
│   │   │   ├── TableList.tsx       # Sidebar table browser
│   │   │   ├── StatusBar.tsx       # Row count, dataset info
│   │   │   └── ErrorBoundary.tsx   # React error boundary
│   │   ├── hooks/
│   │   │   └── useDataset.ts       # Central state management hook
│   │   └── lib/
│   │       └── arrow.ts            # Arrow IPC → row parser (column-wise)
│   └── package.json
└── python_api/                     # 🐍 THE BRIDGE — Future Python bindings
    └── Cargo.toml
```

---

## 🗺️ Roadmap

- [ ] Column filtering and search UI
- [ ] Undo/redo for transformations
- [ ] Charts and data visualization
- [ ] Python API via PyO3/Maturin bindings
- [ ] Multi-tab workspaces
- [ ] Drag-and-drop column reordering
- [ ] Custom CSV delimiter configuration
- [ ] Dark/light theme toggle

---

## 📄 License

[MIT](LICENSE)

---

<p align="center">
  <strong>Built with 🦀 Rust · ⚡ Polars · 🦆 DuckDB · 🏹 Arrow</strong><br/>
  <em>Your data. Your machine. No compromises.</em>
</p>

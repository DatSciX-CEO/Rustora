<div align="center">
  <h1>🛡️ Rustora</h1>
  <p><strong>High-Performance Desktop Data Analytics Engine</strong></p>
  <p><i>Powered by DuckDB, Polars, and Arrow. Built for strict infosec environments.</i></p>
</div>

<hr />

Rustora is a native, highly-optimized desktop application designed to handle large-scale data ingestion, transformation, and analysis. It operates entirely locally natively on your machine, ensuring zero data exfiltration—making it the premier choice for secure, air-gapped, and strict Information Security (InfoSec) environments.

## 🚀 Quick Start (Strict Environment / Air-Gapped)

For maximum security and portability, Rustora provides pure-Rust compilation paths that require **zero web dependencies** (No Node.js, No NPM, No WebView). 

### Option 1: Native UI Build (Maximum Portability)
This path builds the application using `egui`, a pure-Rust immediate mode GUI. It is completely native, extremely lightweight, and has no external JavaScript engine requirements.

```powershell
# Build and run the native egui desktop app
.\setup.ps1 -Egui

# Alternatively, manual cargo build:
cargo build --release -p desktop_egui
```

### Option 2: Cargo-Only Tauri Build (Pre-built UI)
If you prefer the standard rich UI but cannot execute Node/NPM in your environment, use this path. It leverages pre-compiled frontend assets shipped directly with the repository.

```powershell
# Build the Tauri app without relying on NPM/Node.js
.\setup.ps1 -Build -NoNpm

# Or run the standalone build script directly:
.\build_no_npm.ps1
```

**Prerequisites**: [Rust (stable)](https://rustup.rs/) and C++ Build Tools (required for compiling the DuckDB engine from source).

---

## 💻 Developer Installation

For environments where Node.js and NPM are permitted, you can use the full developer workflow which supports hot-reloading and UI development.

```powershell
# Install frontend dependencies and launch the dev server
.\setup.ps1

# Build the release executable and MSI installer
.\setup.ps1 -Build
```

---

## 🏗️ Architecture Layers

Rustora is designed with a strict separation of concerns, ensuring maximum performance at the data layer and flexibility at the presentation layer.

- **`core_engine`**: The pure Rust data engine. Blends **DuckDB** for persistent analytical storage, **Polars** for lightning-fast transient compute, and **Arrow IPC** for zero-copy serialization.
- **`desktop_egui`**: A purely native, extremely lightweight frontend compiled directly to machine code using the egui framework.
- **`desktop_ui`**: A rich, modern frontend built with Tauri.
- **`python_api`**: Hooks for programmatic data manipulation and pipeline integration.

---

## ✨ Key Capabilities

| Feature Category | Description |
|-----------------|-------------|
| **Blazing Fast Ingestion** | Natively load CSV, TSV, Parquet, and Arrow files with granular delimiter configs and instant preview. |
| **Advanced Transforms** | Execute complex filtering (SQL + structured), Group By, Sort, and computed columns at native speed. |
| **Data Manipulation** | Keep, remove, rename, and change column data types seamlessly. |
| **Reshaping** | Leverage DuckDB-powered PIVOT and UNPIVOT logic through an intuitive UI. |
| **Combining** | Perform robust JOINs (Inner, Left, Right, Full) and UNION ALL operations. |
| **Lineage Tracking** | Navigate through Power Query-style "Applied Steps" for full reproducibility of transformations. |
| **Interactive Grid** | Resizable columns, direct cell selection, formula bar, and precise row navigation. |
| **Analytics & Viz** | Instantly generate Bar and Line charts with aggregations, alongside a comprehensive stats panel. |
| **Direct SQL** | Execute your own DuckDB SQL queries directly against your data model. |
| **Export** | Serialize and export your final transformed data to CSV or Parquet. |

## 📜 License

This software is licensed under the **MIT License**.

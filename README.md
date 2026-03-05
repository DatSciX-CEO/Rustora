# Rustora

High-performance desktop data analytics, powered by DuckDB + Polars + Arrow.

## Architecture

- **core_engine** -- Pure Rust data engine (DuckDB persistent storage, Polars transient compute, Arrow IPC serialization)
- **desktop_egui** -- Native egui desktop application (no npm/Node/webview required)

## Features

| Category | Features |
|----------|----------|
| Ingestion | CSV, TSV, Parquet, Arrow with preview & delimiter config |
| Transforms | Filter (SQL + structured), Group By, Sort, Add Calculated Column |
| Column Ops | Remove, Keep, Change Type, Rename |
| Pivot/Unpivot | DuckDB PIVOT/UNPIVOT with dialogs |
| Merge/Append | JOIN (inner/left/right/full) and UNION ALL |
| Applied Steps | Power Query-style transformation lineage with step navigation |
| Grid | Resizable columns, cell selection, formula bar, go-to-row |
| Charts | Bar and Line charts with aggregation |
| Stats | Summary statistics panel |
| SQL | Direct SQL query execution |
| Export | CSV and Parquet export |

## Quick Start

```powershell
# Build the native egui desktop app
.\build_egui.ps1

# Or build manually
cargo build --release -p desktop_egui
```

**Prerequisites**: Rust (stable), C++ build tools (for DuckDB compilation).

## License

MIT

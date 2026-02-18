import { useState, useRef, useEffect } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { ProjectInfo, ColumnInfo } from "../hooks/useDataset";
import type { ParsedTable } from "../lib/arrow";

// ── Types ──────────────────────────────────────────────────────────────────

interface MenuBarProps {
    onNewProject: (path: string) => void;
    onOpenProject: (path: string) => void;
    onImportFile: (path: string) => void;
    onOpenFile: (path: string) => void;
    onExport: (path: string, format: string) => void;
    onToggleSql: () => void;
    onToggleChart: () => void;
    onFilter: (whereClause: string) => void;
    onGroupBy: (groupColumns: string[], aggExprs: string[]) => void;
    onAddColumn: (expression: string, alias: string) => void;
    onSummaryStats: () => Promise<ParsedTable | null>;
    sqlVisible: boolean;
    datasetName: string | null;
    columns: ColumnInfo[];
    project: ProjectInfo | null;
    loading: boolean;
}

const DATA_FILTERS = [
    {
        name: "Data Files",
        extensions: ["csv", "tsv", "parquet", "pq", "ipc", "arrow", "feather"],
    },
    { name: "CSV", extensions: ["csv", "tsv"] },
    { name: "Parquet", extensions: ["parquet", "pq"] },
    { name: "Arrow IPC", extensions: ["ipc", "arrow", "feather"] },
];

// ── Component ──────────────────────────────────────────────────────────────

export function MenuBar({
    onNewProject,
    onOpenProject,
    onImportFile,
    onOpenFile,
    onExport,
    onToggleSql,
    onToggleChart,
    onFilter,
    onGroupBy,
    onAddColumn,
    onSummaryStats,
    sqlVisible,
    datasetName,
    columns,
    project,
    loading,
}: MenuBarProps) {
    const [showFilterDialog, setShowFilterDialog] = useState(false);
    const [showGroupByDialog, setShowGroupByDialog] = useState(false);
    const [showCalcDialog, setShowCalcDialog] = useState(false);
    const [showStatsPanel, setShowStatsPanel] = useState(false);
    const [statsData, setStatsData] = useState<ParsedTable | null>(null);
    const [showExportMenu, setShowExportMenu] = useState(false);
    const exportRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handler = (e: MouseEvent) => {
            if (exportRef.current && !exportRef.current.contains(e.target as Node)) {
                setShowExportMenu(false);
            }
        };
        document.addEventListener("mousedown", handler);
        return () => document.removeEventListener("mousedown", handler);
    }, []);

    // ── File actions ───────────────────────────────────────────────────────

    const handleNewProject = async () => {
        const result = await save({
            filters: [{ name: "Rustora Project", extensions: ["duckdb"] }],
            title: "Create New Rustora Project",
        });
        if (result) onNewProject(result);
    };

    const handleOpenProject = async () => {
        const result = await open({
            multiple: false,
            filters: [{ name: "Rustora Project", extensions: ["duckdb"] }],
            title: "Open Rustora Project",
        });
        if (result) onOpenProject(result);
    };

    const handleOpenFile = async () => {
        const result = await open({
            multiple: false,
            filters: DATA_FILTERS,
            title: "Open Data File",
        });
        if (result) {
            if (project) {
                onImportFile(result);
            } else {
                onOpenFile(result);
            }
        }
    };

    const handleExport = async (format: string) => {
        setShowExportMenu(false);
        const ext = format === "parquet" ? "parquet" : "csv";
        const result = await save({
            filters: [{ name: format.toUpperCase(), extensions: [ext] }],
            title: `Export as ${format.toUpperCase()}`,
        });
        if (result) onExport(result, format);
    };

    // ── Summary Stats ──────────────────────────────────────────────────────

    const handleSummaryStats = async () => {
        setShowStatsPanel(true);
        const data = await onSummaryStats();
        setStatsData(data);
    };

    const hasData = !!datasetName;

    return (
        <>
            {/* ── Brand Bar ─────────────────────────────────────────────── */}
            <div className="menu-bar">
                <span className="app-title">
                    <span className="app-title-icon">🦀</span>
                    RUSTORA
                </span>

                <div className="menu-bar-right">
                    {loading && <span className="loading-indicator">Processing…</span>}
                    {project && (
                        <span className="project-badge" title={project.path}>
                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                                <ellipse cx="12" cy="5" rx="9" ry="3" />
                                <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
                                <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
                            </svg>
                            {project.path.split(/[\\/]/).pop()}
                        </span>
                    )}
                    {datasetName && (
                        <span className="dataset-badge">{datasetName}</span>
                    )}
                </div>
            </div>

            {/* ── Always-Visible Toolbar ─────────────────────────────────── */}
            <div className="toolbar-ribbon-persistent">
                {/* Project */}
                <div className="ribbon-group">
                    <span className="ribbon-group-title">PROJECT</span>
                    <div className="ribbon-group-items">
                        <button className="ribbon-btn-sm" onClick={handleNewProject} disabled={loading} title="Create a new .duckdb project">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 5v14M5 12h14" /></svg>
                            <span>New</span>
                        </button>
                        <button className="ribbon-btn-sm" onClick={handleOpenProject} disabled={loading} title="Open an existing .duckdb project">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><ellipse cx="12" cy="5" rx="9" ry="3" /><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" /><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" /></svg>
                            <span>Open</span>
                        </button>
                    </div>
                </div>

                <div className="ribbon-sep-sm" />

                {/* Data */}
                <div className="ribbon-group">
                    <span className="ribbon-group-title">DATA</span>
                    <div className="ribbon-group-items">
                        <button className="ribbon-btn-sm ribbon-btn-accent" onClick={handleOpenFile} disabled={loading} title="Open or import a data file (CSV, Parquet, Arrow)">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
                            <span>{project ? "Import" : "Open File"}</span>
                        </button>
                    </div>
                </div>

                <div className="ribbon-sep-sm" />

                {/* Transform */}
                <div className="ribbon-group">
                    <span className="ribbon-group-title">TRANSFORM</span>
                    <div className="ribbon-group-items">
                        <button
                            className="ribbon-btn-sm"
                            disabled={!hasData || loading}
                            onClick={() => setShowFilterDialog(true)}
                            title="Filter rows with a SQL WHERE clause"
                        >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" /></svg>
                            <span>Filter</span>
                        </button>
                        <button
                            className={`ribbon-btn-sm ${sqlVisible ? "ribbon-btn-active" : ""}`}
                            onClick={onToggleSql}
                            title="Toggle the SQL query panel"
                        >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" /></svg>
                            <span>SQL</span>
                        </button>
                        <button
                            className="ribbon-btn-sm"
                            disabled={!hasData || loading}
                            onClick={() => setShowGroupByDialog(true)}
                            title="Group by columns with aggregations"
                        >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /></svg>
                            <span>Group By</span>
                        </button>
                        <button
                            className="ribbon-btn-sm"
                            disabled={!hasData || loading}
                            onClick={() => setShowCalcDialog(true)}
                            title="Add a calculated column"
                        >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></svg>
                            <span>Add Col</span>
                        </button>
                    </div>
                </div>

                <div className="ribbon-sep-sm" />

                {/* Analyze */}
                <div className="ribbon-group">
                    <span className="ribbon-group-title">ANALYZE</span>
                    <div className="ribbon-group-items">
                        <button
                            className="ribbon-btn-sm"
                            disabled={!hasData || loading}
                            onClick={handleSummaryStats}
                            title="Show summary statistics for all columns"
                        >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 20V10" /><path d="M12 20V4" /><path d="M6 20v-6" /></svg>
                            <span>Stats</span>
                        </button>
                        <button
                            className="ribbon-btn-sm"
                            disabled={!hasData || loading}
                            onClick={onToggleChart}
                            title="Open the chart visualization panel"
                        >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="18" y1="20" x2="18" y2="10" /><line x1="12" y1="20" x2="12" y2="4" /><line x1="6" y1="20" x2="6" y2="14" /></svg>
                            <span>Chart</span>
                        </button>
                    </div>
                </div>

                <div className="ribbon-sep-sm" />

                {/* Export */}
                <div className="ribbon-group" ref={exportRef}>
                    <span className="ribbon-group-title">EXPORT</span>
                    <div className="ribbon-group-items" style={{ position: "relative" }}>
                        <button
                            className="ribbon-btn-sm"
                            disabled={!hasData || loading}
                            onClick={() => setShowExportMenu((v) => !v)}
                            title="Export data to CSV or Parquet"
                        >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M14 3v4a1 1 0 001 1h4" /><path d="M17 21H7a2 2 0 01-2-2V5a2 2 0 012-2h7l5 5v11a2 2 0 01-2 2z" /></svg>
                            <span>Export ▾</span>
                        </button>
                        {showExportMenu && (
                            <div className="export-dropdown">
                                <button className="export-dropdown-item" onClick={() => handleExport("csv")}>
                                    Export as CSV
                                </button>
                                <button className="export-dropdown-item" onClick={() => handleExport("parquet")}>
                                    Export as Parquet
                                </button>
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* ── Filter Dialog ───────────────────────────────────────────────── */}
            {showFilterDialog && (
                <FilterDialog
                    columns={columns}
                    onSubmit={(clause) => { setShowFilterDialog(false); onFilter(clause); }}
                    onClose={() => setShowFilterDialog(false)}
                />
            )}

            {/* ── Group By Dialog ─────────────────────────────────────────────── */}
            {showGroupByDialog && (
                <GroupByDialog
                    columns={columns}
                    onSubmit={(groupCols, aggs) => { setShowGroupByDialog(false); onGroupBy(groupCols, aggs); }}
                    onClose={() => setShowGroupByDialog(false)}
                />
            )}

            {/* ── Calculated Column Dialog ────────────────────────────────────── */}
            {showCalcDialog && (
                <CalcColumnDialog
                    onSubmit={(expr, alias) => { setShowCalcDialog(false); onAddColumn(expr, alias); }}
                    onClose={() => setShowCalcDialog(false)}
                />
            )}

            {/* ── Stats Panel ─────────────────────────────────────────────────── */}
            {showStatsPanel && (
                <StatsPanel
                    data={statsData}
                    onClose={() => { setShowStatsPanel(false); setStatsData(null); }}
                />
            )}
        </>
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Dialogs
// ═══════════════════════════════════════════════════════════════════════════

function FilterDialog({
    columns,
    onSubmit,
    onClose,
}: {
    columns: ColumnInfo[];
    onSubmit: (clause: string) => void;
    onClose: () => void;
}) {
    const [clause, setClause] = useState("");
    return (
        <div className="dialog-overlay" onClick={onClose}>
            <div className="dialog" onClick={(e) => e.stopPropagation()}>
                <h3>Filter Rows</h3>
                <p className="dialog-help">
                    Enter a SQL WHERE clause. Available columns:{" "}
                    <span className="dialog-cols">{columns.map((c) => c.name).join(", ")}</span>
                </p>
                <textarea
                    className="dialog-input"
                    placeholder='e.g. age > 30 AND city = &#39;Boston&#39;'
                    value={clause}
                    onChange={(e) => setClause(e.target.value)}
                    onKeyDown={(e) => {
                        if (e.key === "Enter" && (e.ctrlKey || e.metaKey) && clause.trim()) {
                            onSubmit(clause.trim());
                        }
                    }}
                    autoFocus
                />
                <div className="dialog-actions">
                    <button className="dialog-btn-secondary" onClick={onClose}>Cancel</button>
                    <button
                        className="dialog-btn-primary"
                        disabled={!clause.trim()}
                        onClick={() => onSubmit(clause.trim())}
                    >
                        Apply Filter
                    </button>
                </div>
            </div>
        </div>
    );
}

function GroupByDialog({
    columns,
    onSubmit,
    onClose,
}: {
    columns: ColumnInfo[];
    onSubmit: (groupCols: string[], aggs: string[]) => void;
    onClose: () => void;
}) {
    const [selectedCols, setSelectedCols] = useState<Set<string>>(new Set());
    const [aggExpr, setAggExpr] = useState("COUNT(*)");

    const toggleCol = (name: string) => {
        setSelectedCols((prev) => {
            const next = new Set(prev);
            if (next.has(name)) next.delete(name);
            else next.add(name);
            return next;
        });
    };

    return (
        <div className="dialog-overlay" onClick={onClose}>
            <div className="dialog" onClick={(e) => e.stopPropagation()}>
                <h3>Group By</h3>
                <p className="dialog-help">Select columns to group by:</p>
                <div className="dialog-col-grid">
                    {columns.map((c) => (
                        <label key={c.name} className="dialog-col-check">
                            <input
                                type="checkbox"
                                checked={selectedCols.has(c.name)}
                                onChange={() => toggleCol(c.name)}
                            />
                            <span>{c.name}</span>
                            <span className="dialog-col-dtype">{c.dtype}</span>
                        </label>
                    ))}
                </div>
                <p className="dialog-help" style={{ marginTop: 12 }}>
                    Aggregation expressions (comma-separated):
                </p>
                <textarea
                    className="dialog-input"
                    placeholder="COUNT(*), AVG(salary), SUM(amount)"
                    value={aggExpr}
                    onChange={(e) => setAggExpr(e.target.value)}
                />
                <div className="dialog-actions">
                    <button className="dialog-btn-secondary" onClick={onClose}>Cancel</button>
                    <button
                        className="dialog-btn-primary"
                        disabled={selectedCols.size === 0 || !aggExpr.trim()}
                        onClick={() =>
                            onSubmit(
                                Array.from(selectedCols),
                                aggExpr.split(",").map((s) => s.trim()).filter(Boolean)
                            )
                        }
                    >
                        Apply Group By
                    </button>
                </div>
            </div>
        </div>
    );
}

function CalcColumnDialog({
    onSubmit,
    onClose,
}: {
    onSubmit: (expr: string, alias: string) => void;
    onClose: () => void;
}) {
    const [expr, setExpr] = useState("");
    const [alias, setAlias] = useState("");

    return (
        <div className="dialog-overlay" onClick={onClose}>
            <div className="dialog" onClick={(e) => e.stopPropagation()}>
                <h3>Add Calculated Column</h3>
                <p className="dialog-help">Column Name:</p>
                <input
                    className="dialog-input-single"
                    placeholder="e.g. annual_salary"
                    value={alias}
                    onChange={(e) => setAlias(e.target.value)}
                    autoFocus
                />
                <p className="dialog-help" style={{ marginTop: 8 }}>SQL Expression:</p>
                <textarea
                    className="dialog-input"
                    placeholder="e.g. salary * 12"
                    value={expr}
                    onChange={(e) => setExpr(e.target.value)}
                />
                <div className="dialog-actions">
                    <button className="dialog-btn-secondary" onClick={onClose}>Cancel</button>
                    <button
                        className="dialog-btn-primary"
                        disabled={!expr.trim() || !alias.trim()}
                        onClick={() => onSubmit(expr.trim(), alias.trim())}
                    >
                        Add Column
                    </button>
                </div>
            </div>
        </div>
    );
}

function StatsPanel({
    data,
    onClose,
}: {
    data: ParsedTable | null;
    onClose: () => void;
}) {
    return (
        <div className="stats-panel-overlay" onClick={onClose}>
            <div className="stats-panel" onClick={(e) => e.stopPropagation()}>
                <div className="stats-panel-header">
                    <h3>Summary Statistics</h3>
                    <button className="stats-close-btn" onClick={onClose}>&times;</button>
                </div>
                {!data ? (
                    <div className="stats-loading">
                        <div className="grid-loading-spinner" />
                        <span>Calculating…</span>
                    </div>
                ) : (
                    <div className="stats-table-wrap">
                        <table className="stats-table">
                            <thead>
                                <tr>
                                    {data.columns.map((c) => (
                                        <th key={c}>{c}</th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {data.rows.map((row, i) => (
                                    <tr key={i}>
                                        {data.columns.map((c) => (
                                            <td key={c}>{row[c] != null ? String(row[c]) : "—"}</td>
                                        ))}
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>
        </div>
    );
}

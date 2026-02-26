import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseIpcBytes, type ParsedTable } from "../lib/arrow";
import { errorMessage } from "../lib/error";
import type { ColumnInfo } from "../hooks/useDataset";
import {
  BarChart,
  Bar,
  LineChart,
  Line,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import type { PieLabelRenderProps } from "recharts";

interface ChartPanelProps {
  datasetName: string;
  columns: ColumnInfo[];
  onClose: () => void;
}

type ChartType = "bar" | "line" | "pie";

const AGG_TYPES = [
  { value: "count", label: "Count" },
  { value: "sum", label: "Sum" },
  { value: "avg", label: "Average" },
  { value: "min", label: "Min" },
  { value: "max", label: "Max" },
];

const CHART_COLORS = [
  "#c45a2c",
  "#3a7cbf",
  "#3a8a5c",
  "#b8860b",
  "#7b5ea7",
  "#c73e3a",
  "#2e6aa8",
  "#e07c3e",
  "#5ba582",
  "#9c6644",
];

export function ChartPanel({
  datasetName,
  columns,
  onClose,
}: ChartPanelProps) {
  const [chartType, setChartType] = useState<ChartType>("bar");
  const [groupCol, setGroupCol] = useState(columns[0]?.name ?? "");
  const [valueCol, setValueCol] = useState(columns.length > 1 ? columns[1].name : "");
  const [aggType, setAggType] = useState("count");
  const [limit, setLimit] = useState(20);
  const [chartData, setChartData] = useState<Record<string, unknown>[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const needsValueCol = aggType !== "count";

  const generateChart = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const bytes = await invoke<number[]>("aggregate_for_chart", {
        datasetName,
        groupCol,
        valueCol: needsValueCol ? valueCol : null,
        aggType,
        limit,
      });
      const parsed: ParsedTable = parseIpcBytes(bytes);
      setChartData(parsed.rows);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setLoading(false);
    }
  }, [datasetName, groupCol, valueCol, aggType, limit, needsValueCol]);

  return (
    <div className="chart-panel-overlay" onClick={onClose}>
      <div className="chart-panel" onClick={(e) => e.stopPropagation()}>
        <div className="chart-panel-header">
          <h3>Chart Builder</h3>
          <button className="stats-close-btn" onClick={onClose}>
            &times;
          </button>
        </div>

        <div className="chart-panel-body">
          {/* Config sidebar */}
          <div className="chart-config">
            <div className="chart-config-section">
              <label className="chart-config-label">Chart Type</label>
              <div className="chart-type-picker">
                {(["bar", "line", "pie"] as ChartType[]).map((t) => (
                  <button
                    key={t}
                    className={`chart-type-btn ${chartType === t ? "active" : ""}`}
                    onClick={() => setChartType(t)}
                  >
                    {t === "bar" && (
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="M18 20V10" /><path d="M12 20V4" /><path d="M6 20v-6" />
                      </svg>
                    )}
                    {t === "line" && (
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
                      </svg>
                    )}
                    {t === "pie" && (
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="M21.21 15.89A10 10 0 118 2.83" /><path d="M22 12A10 10 0 0012 2v10z" />
                      </svg>
                    )}
                    <span>{t.charAt(0).toUpperCase() + t.slice(1)}</span>
                  </button>
                ))}
              </div>
            </div>

            <div className="chart-config-section">
              <label className="chart-config-label">Group By (X Axis)</label>
              <select
                className="chart-config-select"
                value={groupCol}
                onChange={(e) => setGroupCol(e.target.value)}
              >
                {columns.map((c) => (
                  <option key={c.name} value={c.name}>
                    {c.name} ({c.dtype})
                  </option>
                ))}
              </select>
            </div>

            <div className="chart-config-section">
              <label className="chart-config-label">Aggregation</label>
              <select
                className="chart-config-select"
                value={aggType}
                onChange={(e) => setAggType(e.target.value)}
              >
                {AGG_TYPES.map((a) => (
                  <option key={a.value} value={a.value}>
                    {a.label}
                  </option>
                ))}
              </select>
            </div>

            {needsValueCol && (
              <div className="chart-config-section">
                <label className="chart-config-label">Value Column (Y Axis)</label>
                <select
                  className="chart-config-select"
                  value={valueCol}
                  onChange={(e) => setValueCol(e.target.value)}
                >
                  {columns.map((c) => (
                    <option key={c.name} value={c.name}>
                      {c.name} ({c.dtype})
                    </option>
                  ))}
                </select>
              </div>
            )}

            <div className="chart-config-section">
              <label className="chart-config-label">Max Groups</label>
              <input
                className="chart-config-input"
                type="number"
                min={5}
                max={100}
                value={limit}
                onChange={(e) => setLimit(Number(e.target.value) || 20)}
              />
            </div>

            <button
              className="chart-generate-btn"
              onClick={generateChart}
              disabled={loading || !groupCol}
            >
              {loading ? "Generating..." : "Generate Chart"}
            </button>

            {error && <div className="chart-error">{error}</div>}
          </div>

          {/* Chart display area */}
          <div className="chart-display">
            {!chartData && !loading && (
              <div className="chart-placeholder">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.3">
                  <path d="M18 20V10" /><path d="M12 20V4" /><path d="M6 20v-6" />
                </svg>
                <span>Configure options and click Generate</span>
              </div>
            )}

            {loading && (
              <div className="chart-placeholder">
                <div className="grid-loading-spinner" />
                <span>Aggregating data...</span>
              </div>
            )}

            {chartData && !loading && (
              <ResponsiveContainer width="100%" height="100%">
                {chartType === "bar" ? (
                  <BarChart data={chartData} margin={{ top: 20, right: 30, left: 20, bottom: 60 }}>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--border-subtle)" />
                    <XAxis
                      dataKey="label"
                      tick={{ fontSize: 11, fill: "var(--text-secondary)" }}
                      angle={-35}
                      textAnchor="end"
                      height={60}
                    />
                    <YAxis tick={{ fontSize: 11, fill: "var(--text-secondary)" }} />
                    <Tooltip
                      contentStyle={{
                        background: "var(--bg-surface)",
                        border: "1px solid var(--border-default)",
                        borderRadius: 6,
                        fontSize: 12,
                      }}
                    />
                    <Bar dataKey="value" fill={CHART_COLORS[0]} radius={[3, 3, 0, 0]} />
                  </BarChart>
                ) : chartType === "line" ? (
                  <LineChart data={chartData} margin={{ top: 20, right: 30, left: 20, bottom: 60 }}>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--border-subtle)" />
                    <XAxis
                      dataKey="label"
                      tick={{ fontSize: 11, fill: "var(--text-secondary)" }}
                      angle={-35}
                      textAnchor="end"
                      height={60}
                    />
                    <YAxis tick={{ fontSize: 11, fill: "var(--text-secondary)" }} />
                    <Tooltip
                      contentStyle={{
                        background: "var(--bg-surface)",
                        border: "1px solid var(--border-default)",
                        borderRadius: 6,
                        fontSize: 12,
                      }}
                    />
                    <Line
                      type="monotone"
                      dataKey="value"
                      stroke={CHART_COLORS[0]}
                      strokeWidth={2}
                      dot={{ fill: CHART_COLORS[0], r: 3 }}
                    />
                  </LineChart>
                ) : (
                  <PieChart>
                    <Pie
                      data={chartData}
                      dataKey="value"
                      nameKey="label"
                      cx="50%"
                      cy="50%"
                      outerRadius="75%"
                      label={(props: PieLabelRenderProps) =>
                        `${props.name ?? ""} (${(((props.percent as number | undefined) ?? 0) * 100).toFixed(0)}%)`
                      }
                      labelLine={{ strokeWidth: 1 }}
                    >
                      {chartData.map((_, idx) => (
                        <Cell
                          key={idx}
                          fill={CHART_COLORS[idx % CHART_COLORS.length]}
                        />
                      ))}
                    </Pie>
                    <Tooltip
                      contentStyle={{
                        background: "var(--bg-surface)",
                        border: "1px solid var(--border-default)",
                        borderRadius: 6,
                        fontSize: 12,
                      }}
                    />
                    <Legend />
                  </PieChart>
                )}
              </ResponsiveContainer>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

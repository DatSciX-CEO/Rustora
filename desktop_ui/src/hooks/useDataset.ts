import { useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseIpcBytes, type ParsedTable } from "../lib/arrow";
import { errorMessage } from "../lib/error";

export interface ColumnInfo {
  name: string;
  dtype: string;
}

export interface ProjectInfo {
  path: string;
  tables: string[];
}

export interface DatasetState {
  name: string | null;
  columns: ColumnInfo[];
  totalRows: number;
  sizeBytes: number | null;
  currentPage: ParsedTable | null;
  offset: number;
  pageSize: number;
  loading: boolean;
  error: string | null;
  sortColumn: string | null;
  sortDesc: boolean;
  persistent: boolean;
  project: ProjectInfo | null;
  tables: string[];
}

const PAGE_SIZE = 500;

export function useDataset() {
  const [state, setState] = useState<DatasetState>({
    name: null,
    columns: [],
    totalRows: 0,
    sizeBytes: null,
    currentPage: null,
    offset: 0,
    pageSize: PAGE_SIZE,
    loading: false,
    error: null,
    sortColumn: null,
    sortDesc: false,
    persistent: false,
    project: null,
    tables: [],
  });

  const activeDataset = useRef<string | null>(null);
  const pageRequestId = useRef(0);

  const fetchChunk = useCallback(
    async (datasetName: string, offset: number, limit: number) => {
      const bytes = await invoke<number[]>("get_chunk", {
        datasetName,
        offset,
        limit,
      });
      return parseIpcBytes(bytes);
    },
    []
  );

  const refreshTableList = useCallback(async () => {
    try {
      const tables = await invoke<string[]>("list_datasets");
      setState((s) => ({ ...s, tables }));
    } catch {
      // ignore
    }
  }, []);

  // ── Project Management ─────────────────────────────────────────────────

  const newProject = useCallback(
    async (path: string) => {
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const result = await invoke<{ path: string; tables: string[] }>(
          "new_project",
          { path }
        );
        setState((s) => ({
          ...s,
          project: result,
          tables: result.tables,
          name: null,
          columns: [],
          totalRows: 0,
          currentPage: null,
          offset: 0,
          loading: false,
          sortColumn: null,
          sortDesc: false,
        }));
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    []
  );

  const openProject = useCallback(
    async (path: string) => {
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const result = await invoke<{ path: string; tables: string[] }>(
          "open_project",
          { path }
        );
        setState((s) => ({
          ...s,
          project: result,
          tables: result.tables,
          name: null,
          columns: [],
          totalRows: 0,
          currentPage: null,
          offset: 0,
          loading: false,
          sortColumn: null,
          sortDesc: false,
        }));
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    []
  );

  // ── File Import / Open ─────────────────────────────────────────────────

  const applyOpenResult = useCallback(
    async (result: {
      dataset_name: string;
      columns: ColumnInfo[];
      total_rows: number;
      persistent: boolean;
      size_bytes?: number | null;
    }) => {
      activeDataset.current = result.dataset_name;
      const page = await fetchChunk(result.dataset_name, 0, PAGE_SIZE);
      await refreshTableList();

      setState((s) => ({
        ...s,
        name: result.dataset_name,
        columns: result.columns,
        totalRows: result.total_rows,
        sizeBytes: result.size_bytes ?? null,
        currentPage: page,
        offset: 0,
        pageSize: PAGE_SIZE,
        loading: false,
        error: null,
        sortColumn: null,
        sortDesc: false,
        persistent: result.persistent,
      }));
    },
    [fetchChunk, refreshTableList]
  );

  const openFile = useCallback(
    async (path: string) => {
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const result = await invoke<{
          dataset_name: string;
          columns: ColumnInfo[];
          total_rows: number;
          persistent: boolean;
          size_bytes: number | null;
        }>("open_file", { path });
        await applyOpenResult(result);
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    [applyOpenResult]
  );

  const importFile = useCallback(
    async (path: string, tableName?: string) => {
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const result = await invoke<{
          dataset_name: string;
          columns: ColumnInfo[];
          total_rows: number;
          persistent: boolean;
          size_bytes: number | null;
        }>("import_file", { path, tableName: tableName ?? null });
        await applyOpenResult(result);
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    [applyOpenResult]
  );

  const selectTable = useCallback(
    async (tableName: string) => {
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const [bytes, datasets] = await Promise.all([
          invoke<number[]>("get_chunk", {
            datasetName: tableName,
            offset: 0,
            limit: PAGE_SIZE,
          }),
          invoke<string[]>("list_datasets"),
        ]);
        const page = parseIpcBytes(bytes);
        activeDataset.current = tableName;

        setState((s) => ({
          ...s,
          name: tableName,
          columns: page.columns.map((c, i) => ({
            name: c,
            dtype: page.dtypes[i] || "Unknown",
          })),
          totalRows: page.rowCount,
          currentPage: page,
          offset: 0,
          pageSize: PAGE_SIZE,
          loading: false,
          error: null,
          sortColumn: null,
          sortDesc: false,
          persistent: true,
          tables: datasets,
        }));
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    []
  );

  // ── Data Operations ────────────────────────────────────────────────────

  const loadPage = useCallback(
    async (offset: number) => {
      if (!activeDataset.current) return;
      const requestId = ++pageRequestId.current;
      setState((s) => ({ ...s, loading: true }));
      try {
        const page = await fetchChunk(
          activeDataset.current,
          offset,
          PAGE_SIZE
        );
        if (requestId !== pageRequestId.current) return;
        setState((s) => ({
          ...s,
          currentPage: page,
          offset,
          loading: false,
        }));
      } catch (e) {
        if (requestId !== pageRequestId.current) return;
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    [fetchChunk]
  );

  const sortBy = useCallback(
    async (column: string) => {
      if (!activeDataset.current) return;
      setState((s) => ({ ...s, loading: true, error: null }));

      const desc = state.sortColumn === column ? !state.sortDesc : false;

      try {
        const result = await invoke<{
          dataset_name: string;
          columns: ColumnInfo[];
          total_rows: number;
          persistent: boolean;
          size_bytes: number | null;
        }>("sort_dataset", {
          datasetName: activeDataset.current,
          columns: [column],
          descending: [desc],
        });
        await applyOpenResult({ ...result, persistent: result.persistent });
        setState((s) => ({ ...s, sortColumn: column, sortDesc: desc }));
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    [applyOpenResult, state.sortColumn, state.sortDesc]
  );

  const executeSql = useCallback(
    async (sql: string) => {
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const result = await invoke<{
          dataset_name: string;
          columns: ColumnInfo[];
          total_rows: number;
          persistent: boolean;
          size_bytes: number | null;
        }>("execute_sql", { sql });
        await applyOpenResult(result);
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    [applyOpenResult]
  );

  const exportDataset = useCallback(
    async (outputPath: string, format: string) => {
      if (!activeDataset.current) return;
      try {
        await invoke("export_dataset", {
          datasetName: activeDataset.current,
          outputPath,
          format,
        });
      } catch (e) {
        setState((s) => ({ ...s, error: errorMessage(e) }));
      }
    },
    []
  );

  const removeDataset = useCallback(
    async (name: string) => {
      try {
        await invoke("remove_dataset", { datasetName: name });
        await refreshTableList();
        if (activeDataset.current === name) {
          activeDataset.current = null;
          setState((s) => ({
            ...s,
            name: null,
            columns: [],
            totalRows: 0,
            currentPage: null,
          }));
        }
      } catch (e) {
        setState((s) => ({ ...s, error: errorMessage(e) }));
      }
    },
    [refreshTableList]
  );

  // ── Transform Operations ─────────────────────────────────────────────

  const filterDataset = useCallback(
    async (whereClause: string) => {
      if (!activeDataset.current) return;
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const result = await invoke<{
          dataset_name: string;
          columns: ColumnInfo[];
          total_rows: number;
          persistent: boolean;
          size_bytes: number | null;
        }>("filter_dataset", {
          datasetName: activeDataset.current,
          whereClause,
        });
        await applyOpenResult(result);
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    [applyOpenResult]
  );

  const filterDatasetStructured = useCallback(
    async (
      conditions: {
        column: string;
        operator: string;
        value: string;
      }[],
      logic: string = "and"
    ) => {
      if (!activeDataset.current) return;
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const result = await invoke<{
          dataset_name: string;
          columns: ColumnInfo[];
          total_rows: number;
          persistent: boolean;
          size_bytes: number | null;
        }>("filter_dataset_structured", {
          datasetName: activeDataset.current,
          conditions,
          logic,
        });
        await applyOpenResult(result);
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    [applyOpenResult]
  );

  const groupByDataset = useCallback(
    async (groupColumns: string[], aggExprs: string[]) => {
      if (!activeDataset.current) return;
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const result = await invoke<{
          dataset_name: string;
          columns: ColumnInfo[];
          total_rows: number;
          persistent: boolean;
          size_bytes: number | null;
        }>("group_by", {
          datasetName: activeDataset.current,
          groupColumns,
          aggExprs,
        });
        await applyOpenResult(result);
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    [applyOpenResult]
  );

  const addCalculatedColumn = useCallback(
    async (expression: string, alias: string) => {
      if (!activeDataset.current) return;
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const result = await invoke<{
          dataset_name: string;
          columns: ColumnInfo[];
          total_rows: number;
          persistent: boolean;
          size_bytes: number | null;
        }>("add_calculated_column", {
          datasetName: activeDataset.current,
          expression,
          alias,
        });
        await applyOpenResult(result);
      } catch (e) {
        setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
      }
    },
    [applyOpenResult]
  );

  const getSummaryStats = useCallback(async (): Promise<ParsedTable | null> => {
    if (!activeDataset.current) return null;
    try {
      const bytes = await invoke<number[]>("get_summary_stats", {
        datasetName: activeDataset.current,
      });
      return parseIpcBytes(bytes);
    } catch (e) {
      setState((s) => ({ ...s, error: errorMessage(e) }));
      return null;
    }
  }, []);

  return {
    state,
    newProject,
    openProject,
    openFile,
    importFile,
    selectTable,
    loadPage,
    sortBy,
    executeSql,
    exportDataset,
    removeDataset,
    filterDataset,
    filterDatasetStructured,
    groupByDataset,
    addCalculatedColumn,
    getSummaryStats,
  };
}


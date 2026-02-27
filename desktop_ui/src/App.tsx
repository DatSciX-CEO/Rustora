/**
 * App — root application component for the Rustora desktop application.
 *
 * Wires together the useDataset hook with all UI components. Manages SQL panel
 * and chart panel visibility state locally, and delegates all data operations
 * to the useDataset hook. File dialog interactions for the welcome screen are
 * handled here to avoid importing Tauri dialog APIs into child components.
 */
import { useState, useCallback } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useDataset } from "./hooks/useDataset";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { MenuBar } from "./components/MenuBar";
import { SqlPanel } from "./components/SqlPanel";
import { DataGrid } from "./components/DataGrid";
import { StatusBar } from "./components/StatusBar";
import { TableList } from "./components/TableList";
import { ChartPanel } from "./components/ChartPanel";
import "./App.css";

const DATA_FILTERS = [
  { name: "Data Files", extensions: ["csv", "tsv", "parquet", "pq", "ipc", "arrow", "feather"] },
  { name: "CSV", extensions: ["csv", "tsv"] },
  { name: "Parquet", extensions: ["parquet", "pq"] },
  { name: "Arrow IPC", extensions: ["ipc", "arrow", "feather"] },
];

function App() {
  const {
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
    retryLastAction,
  } = useDataset();
  const [sqlVisible, setSqlVisible] = useState(false);
  const [chartVisible, setChartVisible] = useState(false);

  const showSidebar = state.tables.length > 0;

  const handleWelcomeNewProject = useCallback(async () => {
    const result = await save({
      filters: [{ name: "Rustora Project", extensions: ["duckdb"] }],
      title: "Create New Rustora Project",
    });
    if (result) newProject(result);
  }, [newProject]);

  const handleWelcomeOpenProject = useCallback(async () => {
    const result = await open({
      multiple: false,
      filters: [{ name: "Rustora Project", extensions: ["duckdb"] }],
      title: "Open Rustora Project",
    });
    if (result) openProject(result);
  }, [openProject]);

  const handleWelcomeOpenFile = useCallback(async () => {
    const result = await open({
      multiple: false,
      filters: DATA_FILTERS,
      title: "Open Data File",
    });
    if (result) {
      if (state.project) {
        importFile(result);
      } else {
        openFile(result);
      }
    }
  }, [openFile, importFile, state.project]);

  return (
    <ErrorBoundary>
      <div className="app">
        <MenuBar
          onNewProject={newProject}
          onOpenProject={openProject}
          onImportFile={importFile}
          onOpenFile={openFile}
          onExport={exportDataset}
          onToggleSql={() => setSqlVisible(!sqlVisible)}
          onToggleChart={() => setChartVisible(!chartVisible)}
          onFilter={filterDataset}
          onGroupBy={groupByDataset}
          onAddColumn={addCalculatedColumn}
          onSummaryStats={getSummaryStats}
          sqlVisible={sqlVisible}
          datasetName={state.name}
          columns={state.columns}
          project={state.project}
          loading={state.loading}
        />
        <SqlPanel
          onExecute={executeSql}
          loading={state.loading}
          visible={sqlVisible}
        />
        <div className="main-content">
          <TableList
            tables={state.tables}
            activeTable={state.name}
            onSelectTable={selectTable}
            onRemoveTable={removeDataset}
            visible={showSidebar}
          />
          <DataGrid
            page={state.currentPage}
            columns={state.columns}
            totalRows={state.totalRows}
            offset={state.offset}
            pageSize={state.pageSize}
            sortColumn={state.sortColumn}
            sortDesc={state.sortDesc}
            onSort={sortBy}
            onPageChange={loadPage}
            onFilterStructured={filterDatasetStructured}
            onNewProject={handleWelcomeNewProject}
            onOpenProject={handleWelcomeOpenProject}
            onOpenFile={handleWelcomeOpenFile}
            loading={state.loading}
          />
        </div>
        {chartVisible && state.name && (
          <ChartPanel
            datasetName={state.name}
            columns={state.columns}
            onClose={() => setChartVisible(false)}
          />
        )}
        <StatusBar
          totalRows={state.totalRows}
          totalColumns={state.columns.length}
          offset={state.offset}
          pageSize={state.pageSize}
          datasetName={state.name}
          sizeBytes={state.sizeBytes}
          persistent={state.persistent}
          projectPath={state.project?.path ?? null}
          error={state.error}
          loading={state.loading}
          onRetry={retryLastAction}
        />
      </div>
    </ErrorBoundary>
  );
}

export default App;

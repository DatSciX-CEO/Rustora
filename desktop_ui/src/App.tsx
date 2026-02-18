import { useState } from "react";
import { useDataset } from "./hooks/useDataset";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { MenuBar } from "./components/MenuBar";
import { SqlPanel } from "./components/SqlPanel";
import { DataGrid } from "./components/DataGrid";
import { StatusBar } from "./components/StatusBar";
import { TableList } from "./components/TableList";
import "./App.css";

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
    groupByDataset,
    addCalculatedColumn,
    getSummaryStats,
  } = useDataset();
  const [sqlVisible, setSqlVisible] = useState(false);

  const showSidebar = state.tables.length > 0;

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
            loading={state.loading}
          />
        </div>
        <StatusBar
          totalRows={state.totalRows}
          totalColumns={state.columns.length}
          offset={state.offset}
          pageSize={state.pageSize}
          datasetName={state.name}
          persistent={state.persistent}
          projectPath={state.project?.path ?? null}
          error={state.error}
          loading={state.loading}
        />
      </div>
    </ErrorBoundary>
  );
}

export default App;

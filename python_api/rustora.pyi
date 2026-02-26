"""Type stubs for the rustora native extension module."""

from typing import Optional

class Session:
    """Core session managing all data operations.

    Usage::

        import rustora

        session = rustora.Session()
        session.new_project("analysis.duckdb")
        session.import_file("data.csv", "my_table")
        ipc_bytes = session.get_preview("my_table", 100)
    """

    def __init__(self) -> None:
        """Create a new session with an in-memory scratch database."""
        ...

    def new_project(self, path: str) -> None:
        """Create a new persistent project (.duckdb file).

        Args:
            path: Filesystem path for the new DuckDB database file.

        Raises:
            RuntimeError: If the file cannot be created.
        """
        ...

    def open_project(self, path: str) -> list[str]:
        """Open an existing project. Returns the list of table names.

        Args:
            path: Path to an existing .duckdb file.

        Returns:
            List of persistent table names already in the database.

        Raises:
            RuntimeError: If the file does not exist or is not a valid DuckDB database.
        """
        ...

    def import_file(self, path: str, table_name: Optional[str] = None) -> str:
        """Import a file into the DuckDB project as a persistent table.

        Supported formats: CSV, TSV, Parquet, Arrow IPC / Feather.

        Args:
            path: Path to the data file.
            table_name: Optional name for the table. Auto-generated from filename if omitted.

        Returns:
            The table name used in the database.

        Raises:
            RuntimeError: If no project is open, or the file format is unsupported.
        """
        ...

    def scan_file(self, path: str) -> str:
        """Lazily scan a file via Polars (transient, not persisted).

        Args:
            path: Path to the data file.

        Returns:
            A generated dataset name.

        Raises:
            RuntimeError: If the file does not exist or has an unsupported format.
        """
        ...

    def list_datasets(self) -> list[str]:
        """List all available datasets (persistent tables + transient scans)."""
        ...

    def get_row_count(self, name: str) -> int:
        """Get the total row count for a dataset.

        Args:
            name: Dataset / table name.

        Raises:
            RuntimeError: If the dataset is not found.
        """
        ...

    def get_preview(self, name: str, limit: int) -> bytes:
        """Get a preview of a dataset as Arrow IPC stream bytes.

        Args:
            name: Dataset / table name.
            limit: Maximum number of rows to return.

        Returns:
            Arrow IPC stream bytes. Parse with ``pyarrow.ipc.open_stream``
            or ``polars.read_ipc(io.BytesIO(data))``.

        Raises:
            RuntimeError: If the dataset is not found.
        """
        ...

    def get_chunk(self, name: str, offset: int, limit: int) -> bytes:
        """Get a paginated chunk of rows as Arrow IPC stream bytes.

        Args:
            name: Dataset / table name.
            offset: Row offset to start from.
            limit: Maximum number of rows to return.

        Returns:
            Arrow IPC stream bytes.

        Raises:
            RuntimeError: If the dataset is not found.
        """
        ...

    def execute_sql(self, sql: str) -> str:
        """Execute a SQL query. The result is stored as a new persistent table.

        Args:
            sql: A valid SQL SELECT statement.

        Returns:
            The name of the result table.

        Raises:
            RuntimeError: If no project is open or SQL execution fails.
        """
        ...

    def query_to_ipc(self, sql: str) -> bytes:
        """Execute a SQL query and return results directly as Arrow IPC bytes.

        Unlike ``execute_sql``, this does not persist the result as a table.

        Args:
            sql: A valid SQL SELECT statement.

        Returns:
            Arrow IPC stream bytes.

        Raises:
            RuntimeError: If no project is open or SQL execution fails.
        """
        ...

    def sort_dataset(self, name: str, columns: list[str], descending: list[bool]) -> str:
        """Sort a dataset by one or more columns. Returns the new dataset name.

        Args:
            name: Source dataset / table name.
            columns: Column names to sort by.
            descending: Parallel list of booleans (True = descending).

        Returns:
            The name of the new sorted dataset.

        Raises:
            RuntimeError: If the dataset is not found.
        """
        ...

    def filter_sql(self, name: str, where_clause: str) -> str:
        """Filter a dataset using a SQL WHERE clause. Returns the new dataset name.

        Args:
            name: Source dataset / table name.
            where_clause: SQL predicate, e.g. ``"age > 30 AND city = 'Boston'"``.

        Returns:
            The name of the new filtered dataset.

        Raises:
            RuntimeError: If the dataset is not found or SQL is invalid.
        """
        ...

    def export_csv(self, name: str, output_path: str) -> None:
        """Export a dataset to a CSV file.

        Args:
            name: Dataset / table name.
            output_path: Destination file path.

        Raises:
            RuntimeError: If the dataset is not found or the path is not writable.
        """
        ...

    def export_parquet(self, name: str, output_path: str) -> None:
        """Export a dataset to a Parquet file.

        Args:
            name: Dataset / table name.
            output_path: Destination file path.

        Raises:
            RuntimeError: If the dataset is not found or the path is not writable.
        """
        ...

    def remove_dataset(self, name: str) -> bool:
        """Remove a dataset (drops DuckDB table or removes transient scan).

        Args:
            name: Dataset / table name.

        Returns:
            True if the dataset was found and removed, False otherwise.
        """
        ...

"""
Smoke test for rustora Python API.

Build and install first:
    cd python_api
    pip install maturin
    maturin develop --release

Then run:
    python test_smoke.py
"""

import io
import os
import tempfile


def main():
    import rustora

    # ── Session creation ──────────────────────────────────────────────────
    session = rustora.Session()
    print("[OK] Session created")

    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "test.duckdb")
        session.new_project(db_path)
        print(f"[OK] Project created: {db_path}")

        # ── CSV creation ──────────────────────────────────────────────────
        csv_path = os.path.join(tmpdir, "test.csv")
        with open(csv_path, "w") as f:
            f.write("name,age,city,score\n")
            f.write("Alice,30,New York,95.5\n")
            f.write("Bob,25,San Francisco,88.0\n")
            f.write("Charlie,35,Chicago,72.3\n")

        # ── import_file (with explicit table_name) ────────────────────────
        table_name = session.import_file(csv_path, "test_data")
        print(f"[OK] Imported as: {table_name}")
        assert table_name == "test_data"

        # ── import_file (auto-generated table_name) ───────────────────────
        auto_name = session.import_file(csv_path)
        print(f"[OK] Auto-named import: {auto_name}")
        assert auto_name != "test_data"

        # ── list_datasets ─────────────────────────────────────────────────
        datasets = session.list_datasets()
        print(f"[OK] Datasets: {datasets}")
        assert "test_data" in datasets

        # ── get_row_count ─────────────────────────────────────────────────
        count = session.get_row_count("test_data")
        print(f"[OK] Row count: {count}")
        assert count == 3

        # ── get_preview -- validate Arrow IPC format ──────────────────────
        ipc_bytes = session.get_preview("test_data", 10)
        print(f"[OK] Preview IPC bytes: {len(ipc_bytes)} bytes")
        assert len(ipc_bytes) > 0
        try:
            import pyarrow.ipc as pa_ipc
            reader = pa_ipc.open_stream(io.BytesIO(ipc_bytes))
            table = reader.read_all()
            assert table.num_rows == 3, f"Expected 3 rows, got {table.num_rows}"
            assert "name" in table.column_names
            print(f"[OK] Arrow IPC valid: {table.num_rows} rows, columns={table.column_names}")
        except ImportError:
            print("[SKIP] pyarrow not installed, skipping Arrow validation")

        # ── get_chunk (pagination) ────────────────────────────────────────
        chunk = session.get_chunk("test_data", 0, 2)
        assert len(chunk) > 0
        print(f"[OK] get_chunk IPC bytes: {len(chunk)}")

        # ── execute_sql ───────────────────────────────────────────────────
        result = session.execute_sql("SELECT * FROM test_data WHERE age > 28")
        print(f"[OK] SQL result table: {result}")
        assert isinstance(result, str) and len(result) > 0
        result_count = session.get_row_count(result)
        assert result_count == 2, f"Expected 2 rows after filter, got {result_count}"

        # ── query_to_ipc (no persistence) ─────────────────────────────────
        ipc_bytes2 = session.query_to_ipc("SELECT name, score FROM test_data ORDER BY score DESC")
        assert len(ipc_bytes2) > 0
        print(f"[OK] query_to_ipc: {len(ipc_bytes2)} bytes")

        # ── sort_dataset ──────────────────────────────────────────────────
        sorted_name = session.sort_dataset("test_data", ["age"], [False])
        print(f"[OK] Sorted dataset: {sorted_name}")
        assert session.get_row_count(sorted_name) == 3

        # ── sort_dataset -- mismatched lengths (should raise ValueError) ───
        try:
            session.sort_dataset("test_data", ["age", "score"], [True])
            assert False, "Expected ValueError for mismatched lengths"
        except ValueError as e:
            print(f"[OK] sort_dataset length mismatch correctly raised ValueError: {e}")

        # ── filter_sql ────────────────────────────────────────────────────
        filtered = session.filter_sql("test_data", "age < 32")
        filtered_count = session.get_row_count(filtered)
        print(f"[OK] filter_sql: {filtered_count} rows")
        assert 0 < filtered_count < 3

        # ── scan_file (transient) ─────────────────────────────────────────
        scan_name = session.scan_file(csv_path)
        print(f"[OK] scan_file: {scan_name}")
        scan_count = session.get_row_count(scan_name)
        assert scan_count == 3

        # ── export_csv ────────────────────────────────────────────────────
        out_csv = os.path.join(tmpdir, "out.csv")
        session.export_csv("test_data", out_csv)
        with open(out_csv) as f:
            content = f.read()
        assert "Alice" in content
        print(f"[OK] Exported CSV: {len(content)} chars")

        # ── export_parquet ────────────────────────────────────────────────
        out_parquet = os.path.join(tmpdir, "out.parquet")
        session.export_parquet("test_data", out_parquet)
        assert os.path.exists(out_parquet)
        assert os.path.getsize(out_parquet) > 0
        print(f"[OK] Exported Parquet: {os.path.getsize(out_parquet)} bytes")

        # ── remove_dataset ────────────────────────────────────────────────
        removed = session.remove_dataset("test_data")
        assert removed is True
        print("[OK] Dataset removed")
        assert "test_data" not in session.list_datasets()

        # ── remove non-existent dataset ───────────────────────────────────
        removed_again = session.remove_dataset("test_data")
        assert removed_again is False
        print("[OK] Removing non-existent dataset returns False")

        # ── error: file not found ─────────────────────────────────────────
        try:
            session.import_file("/nonexistent/path/data.csv")
            assert False, "Expected FileNotFoundError"
        except FileNotFoundError as e:
            print(f"[OK] FileNotFoundError correctly raised: {e}")

        # ── error: unsupported format ─────────────────────────────────────
        bad_path = os.path.join(tmpdir, "data.xlsx")
        with open(bad_path, "w") as f:
            f.write("fake")
        try:
            session.import_file(bad_path)
            assert False, "Expected ValueError for unsupported format"
        except ValueError as e:
            print(f"[OK] ValueError for unsupported format: {e}")

        # ── error: table not found ────────────────────────────────────────
        try:
            session.get_row_count("nonexistent_table")
            assert False, "Expected ValueError for table not found"
        except ValueError as e:
            print(f"[OK] ValueError for missing table: {e}")

        # ── open_project (round-trip persistence) ─────────────────────────
        db2_path = os.path.join(tmpdir, "persist.duckdb")
        s2 = rustora.Session()
        s2.new_project(db2_path)
        s2.import_file(csv_path, "people")
        del s2

        s3 = rustora.Session()
        tables = s3.open_project(db2_path)
        print(f"[OK] open_project tables: {tables}")
        assert "people" in tables
        assert s3.get_row_count("people") == 3
        del s3

    print("\n=== All smoke tests passed! ===")


if __name__ == "__main__":
    main()

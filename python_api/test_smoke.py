"""
Smoke test for rustora Python API.

Build and install first:
    cd python_api
    pip install maturin
    maturin develop --release

Then run:
    python test_smoke.py
"""

import tempfile
import os


def main():
    import rustora

    session = rustora.Session()
    print("[OK] Session created")

    # Create a temp project
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "test.duckdb")
        session.new_project(db_path)
        print(f"[OK] Project created: {db_path}")

        # Create a test CSV
        csv_path = os.path.join(tmpdir, "test.csv")
        with open(csv_path, "w") as f:
            f.write("name,age,city,score\n")
            f.write("Alice,30,New York,95.5\n")
            f.write("Bob,25,San Francisco,88.0\n")
            f.write("Charlie,35,Chicago,72.3\n")

        # Import it
        table_name = session.import_file(csv_path, "test_data")
        print(f"[OK] Imported as: {table_name}")

        # List datasets
        datasets = session.list_datasets()
        print(f"[OK] Datasets: {datasets}")
        assert "test_data" in datasets

        # Row count
        count = session.get_row_count("test_data")
        print(f"[OK] Row count: {count}")
        assert count == 3

        # Get preview as Arrow IPC bytes
        ipc_bytes = session.get_preview("test_data", 10)
        print(f"[OK] Preview IPC bytes: {len(ipc_bytes)} bytes")
        assert len(ipc_bytes) > 0

        # Execute SQL
        result = session.execute_sql("SELECT * FROM test_data WHERE age > 28")
        print(f"[OK] SQL result table: {result}")

        # Export to CSV
        out_csv = os.path.join(tmpdir, "out.csv")
        session.export_csv("test_data", out_csv)
        with open(out_csv) as f:
            content = f.read()
        assert "Alice" in content
        print(f"[OK] Exported CSV: {len(content)} chars")

        # Clean up
        session.remove_dataset("test_data")
        print("[OK] Dataset removed")

        del session

    print("\n=== All smoke tests passed! ===")


if __name__ == "__main__":
    main()

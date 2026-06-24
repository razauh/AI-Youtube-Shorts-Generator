import json
import sqlite3
import subprocess
import sys
from pathlib import Path

def test_migration_script_maps_active_records(tmp_path: Path) -> None:
    # 1. Create a mock D1 SQLite database
    db_path = tmp_path / "mock_d1.db"
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    cursor.execute("""
        CREATE TABLE licenses (
            license_key_hash TEXT PRIMARY KEY,
            purchaser_email TEXT,
            entitlement_status TEXT NOT NULL,
            provider TEXT,
            provider_sale_id TEXT,
            updated_at_ms INTEGER NOT NULL
        )
    """)

    # Populate licenses
    # Key: "KEY-1", Hash: sha256("pepper:KEY-1") -> 863dd7f99d68ee1748234d9a633f56b6d9b70bf59e5f683cd667e7120c6cde33
    # Key: "KEY-2", Hash: sha256("pepper:KEY-2") -> bb7f35ffa331a9e33bcef102af48c23904a37120f308223eab5c77e0adfdb18d
    # Key: "KEY-3" (disabled), Hash: sha256("pepper:KEY-3") -> e5cbb2f61fbf56d4d2f099e1f9c59c35d5aff12e8f12bc6fa31e668b83d5d492
    cursor.executemany("""
        INSERT INTO licenses (license_key_hash, purchaser_email, entitlement_status, provider, provider_sale_id, updated_at_ms)
        VALUES (?, ?, ?, ?, ?, ?)
    """, [
        ("863dd7f99d68ee1748234d9a633f56b6d9b70bf59e5f683cd667e7120c6cde33", "user1@example.com", "active", "gumroad", "sale_1", 1000),
        ("bb7f35ffa331a9e33bcef102af48c23904a37120f308223eab5c77e0adfdb18d", "user2@example.com", "active", "gumroad", "sale_2", 2000),
        ("e5cbb2f61fbf56d4d2f099e1f9c59c35d5aff12e8f12bc6fa31e668b83d5d492", "user3@example.com", "disabled", "gumroad", "sale_3", 3000)
    ])
    conn.commit()
    conn.close()

    # 2. Create raw keys JSON input
    raw_keys_path = tmp_path / "raw_keys.json"
    raw_keys_data = [
        {"license_key": "KEY-1", "email": "user1@example.com", "sale_id": "sale_1"},
        {"license_key": "KEY-2", "email": "user2@example.com", "sale_id": "sale_2"},
        {"license_key": "KEY-3", "email": "user3@example.com", "sale_id": "sale_3"},
        {"license_key": "KEY-4", "email": "user4@example.com", "sale_id": "sale_4"},  # not in D1
    ]
    raw_keys_path.write_text(json.dumps(raw_keys_data), encoding="utf-8")

    # 3. Call the migration script
    script_path = Path("scripts/migrate_d1_to_devolens.py")
    
    result = subprocess.run(
        [
            sys.executable,
            str(script_path),
            "--db", str(db_path),
            "--pepper", "pepper",
            "--raw-keys", str(raw_keys_path),
            "--product-id", "12345",
            "--token", "devolens_web_api_token"
        ],
        capture_output=True,
        text=True
    )

    # In the RED phase, this will exit non-zero (or fail with FileNotFoundError)
    assert result.returncode == 0, f"Migration script failed: {result.stderr}"

    output_data = json.loads(result.stdout)
    assert len(output_data) == 2, f"Expected exactly 2 active keys migrated, got: {output_data}"

    # Verify KEY-1 details
    key1_record = next(x for x in output_data if x["Key"] == "KEY-1")
    assert key1_record["ProductId"] == 12345
    assert key1_record["Email"] == "user1@example.com"
    assert key1_record["token"] == "devolens_web_api_token"

    # Verify KEY-2 details
    key2_record = next(x for x in output_data if x["Key"] == "KEY-2")
    assert key2_record["ProductId"] == 12345
    assert key2_record["Email"] == "user2@example.com"
    assert key2_record["token"] == "devolens_web_api_token"

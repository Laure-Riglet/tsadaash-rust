/* fn main() {
    println!("Hello, world!");
} */

use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // Creates ./data/app.db (and folders if you already created them)
    std::fs::create_dir_all("data").expect("failed to create data directory");

    let conn = Connection::open("data/app.db")?;

    // Create a tiny table to verify everything works
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _healthcheck (id INTEGER PRIMARY KEY, created_at TEXT NOT NULL)",
        [],
    )?;

    conn.execute(
        "INSERT INTO _healthcheck (created_at) VALUES (datetime('now'))",
        [],
    )?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM _healthcheck", [], |row| row.get(0))?;
    println!("SQLite OK ✅ rows in _healthcheck = {}", count);

    Ok(())
}

use rusqlite::{Connection, Result};

pub fn connect() -> Result<Connection> {
    let conn: Connection = Connection::open("data/app.db")?;

    // Create a tiny table to verify everything works
    conn.execute_batch(
        r#"

            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY, 
                username TEXT NOT NULL, 
                email TEXT NOT NULL, 
                password TEXT NOT NULL, 
                tz_continent TEXT NOT NULL, 
                tz_city TEXT NOT NULL, 
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TRIGGER IF NOT EXISTS update_users_updated_at
            AFTER UPDATE ON users
            FOR EACH ROW
            BEGIN
                UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
            END;

            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                is_recurring BOOLEAN NOT NULL,
                recurrence_interval TEXT,
                recurrence_unit TEXT,
                date TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(user_id) REFERENCES users(id)
            );

            CREATE TRIGGER IF NOT EXISTS update_tasks_updated_at
            AFTER UPDATE ON tasks
            FOR EACH ROW
            BEGIN
                UPDATE tasks SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
            END;
            "#,
    )?;

    return Ok(conn);
}

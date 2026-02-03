use crate::domain::User;
use rusqlite::Connection;

pub fn select_by_email_or_username(
    conn: &Connection,
    identifier: &str,
) -> rusqlite::Result<Option<User>> {
    let mut stmt = conn.prepare(
        "SELECT id, username, email, password, tz_continent, tz_city, created_at, updated_at 
         FROM users 
         WHERE email = ?1 OR username = ?1",
    )?;
    let mut rows = stmt.query([identifier])?;

    if let Some(row) = rows.next()? {
        let user = User::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
        );
        Ok(Some(user))
    } else {
        Ok(None)
    }
}

pub fn insert(
    conn: &Connection,
    username: &str,
    email: &str,
    password: &str,
    tz_continent: &str,
    tz_city: &str,
) -> rusqlite::Result<User> {
    conn.execute(
        "INSERT INTO users (username, email, password, tz_continent, tz_city) VALUES (?1, ?2, ?3, ?4, ?5)",
        (username, email, password, tz_continent, tz_city),
    )?;

    let id = conn.last_insert_rowid() as i32;
    
    let mut stmt = conn.prepare(
        "SELECT id, username, email, password, tz_continent, tz_city, created_at, updated_at 
         FROM users 
         WHERE id = ?1",
    )?;
    let user = stmt.query_row([id], |row| {
        Ok(User::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
        ))
    })?;

    Ok(user)
}

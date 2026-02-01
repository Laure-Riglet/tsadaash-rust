mod cli;
mod domain;
mod db;

use cli::menu::init_menu;
use db::connection::connect;
use rusqlite::{ Connection };

fn main() -> rusqlite::Result<()> {

    // Establish & set database connection
    let conn: Connection = connect()?;

    // Start CLI menu loop
    init_menu(&conn)?;

    Ok(())
}

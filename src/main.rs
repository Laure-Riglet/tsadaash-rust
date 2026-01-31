use rusqlite::{Connection, OptionalExtension, Result};
use std::io;
use tsadaash::domain::{Continents, User};

fn connect() -> Result<Connection> {
    let conn: Connection = Connection::open("data/app.db")?;

    // Create a tiny table to verify everything works
    conn.execute(
        "CREATE TABLE IF NOT EXISTS people (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL, tz_continent TEXT NOT NULL, tz_city TEXT NOT NULL)",
        [],
    )?;

    return Ok(conn);
}

fn signup(conn: &Connection) -> Result<User> {
    // --- tiny helpers (MVP style: keep inside signup) ---

    fn read_line_trimmed(prompt: &str) -> String {
        println!("{}", prompt);

        let mut s = String::new();
        io::stdin().read_line(&mut s).expect("Failed to read input");

        s.trim().to_string()
    }

    fn yes(prompt: &str) -> bool {
        let answer = read_line_trimmed(prompt).to_lowercase();
        matches!(answer.as_str(), "y" | "yes")
    }

    fn ask_confirmed_text(field_pretty: &str, question: &str) -> String {
        loop {
            let input = read_line_trimmed(question);

            println!("You entered:");
            println!("{}: {}", field_pretty, input);

            if yes("Is this correct? [y/N]") {
                return input;
            }

            println!("Ok, let's try again.\n");
        }
    }

    fn ask_continent_confirmed() -> String {
        loop {
            println!("Choose your continent:");
            for (i, c) in Continents::iter().enumerate() {
                println!("{}: {}", i + 1, c);
            }

            let raw = read_line_trimmed("Enter the number of your choice:");
            let choice: usize = match raw.parse() {
                Ok(n) => n,
                Err(_) => {
                    println!("Please enter a number.\n");
                    continue;
                }
            };

            let continent = match Continents::from_choice(choice) {
                Some(c) => c,
                None => {
                    println!("Invalid choice, try again.\n");
                    continue;
                }
            };

            println!("You entered:");
            println!("Continent: {}", continent);

            if yes("Is this correct? [y/N]") {
                return continent.to_string();
            }

            println!("Ok, let's try again.\n");
        }
    }

    // --- main signup flow: keep asking until user confirms everything ---

    loop {
        let name = ask_confirmed_text("Name", "What's your name?");
        let email = ask_confirmed_text("Email", "What's your email?");
        let tz_continent = ask_continent_confirmed();
        let tz_city = ask_confirmed_text(
            "Time zone city",
            "What's your time zone city (e.g., Paris, New_York, Tokyo)?",
        );

        println!("\nSummary:");
        println!("Name: {}", name);
        println!("Email: {}", email);
        println!("Time zone: {}/{}", tz_continent, tz_city);

        if yes("Confirm signup? [y/N]") {
            // Insert
            conn.execute(
                "INSERT INTO people (name, email, tz_continent, tz_city) VALUES (?1, ?2, ?3, ?4)",
                (&name, &email, &tz_continent, &tz_city),
            )?;

            // Build User (works if your User has pub fields; otherwise use User::new(...))
            let id = conn.last_insert_rowid() as i32;

            let user = User::new(
                id,
                name,
                email,
                tz_continent,
                tz_city,
            );

            println!("\nSignup complete! Welcome, {}!", user.name());
            return Ok(user);
        }

        println!("\nOk — restarting signup.\n");
    }
}

fn signin(conn: &Connection) -> rusqlite::Result<Option<User>> {
    println!("What's your name?");
    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .expect("Failed to read input");

    let name = name.trim().to_string();

    let user = conn
        .query_row(
            "SELECT id, name, email, tz_continent, tz_city FROM people WHERE name = ?1",
            [&name],
            |row| {
                Ok(User::new(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()?; // <-- turns "no rows" into Ok(None)

    Ok(user)
}

fn ask_yes_no(prompt: &str) -> bool {
    println!("{}", prompt);

    let mut response = String::new();
    io::stdin()
        .read_line(&mut response)
        .expect("Failed to read input");

    matches!(response.trim().to_lowercase().as_str(), "y" | "yes")
}

fn main() -> rusqlite::Result<()> {
    let conn: Connection = connect()?;

    let current_user: User = if ask_yes_no("Are you a registered user? [y/N]") {
        match signin(&conn)? {
            Some(user) => user,
            None => {
                println!("User not found. Please sign up first.");
                signup(&conn)?
            }
        }
    } else {
        signup(&conn)?
    };

    println!("Welcome, {}!", current_user.name());
    println!("Email: {}", current_user.email());
    println!("TZ: {}/{}", current_user.tz_continent(), current_user.tz_city());

    Ok(())
}

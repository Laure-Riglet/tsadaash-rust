use inquire::Select;
use rusqlite::{ Connection, Result };
use crate::cli::{
        auth::{ signin, signup },
        helpers::{ clear_screen, timezone_user }
    };
use crate::domain::User;
use crate::cli::task::menu;

pub fn init_menu(conn: &Connection) -> Result<()> {

    clear_screen();
    println!("");
    println!("=== Tsadaash ===\n");

    let mut current_user: Option<User> = None;

    loop {
        let options = if let Some(_user) = &current_user {
            vec!["Tasks", "Timezone", "Exit"]
        } else {
            vec!["Signup", "Signin", "Exit"]
        };

        let choice = Select::new("Please choose an option:", options)
            .prompt()
            .unwrap_or_else(|_| "Exit");


        match choice {
            "Signup" => match signup(&conn) {
                Ok(user) => {
                    println!("Welcome, {}!", user.username());
                    current_user = Some(user);
                }
                Err(e) => println!("Signup failed: {}", e),
            },
            "Signin" => match signin(&conn) {
                Ok(Some(user)) => {
                    println!("Welcome back, {}!", user.username());
                    current_user = Some(user);
                }
                Ok(None) => println!("Signin aborted."),
                Err(e) => println!("Signin failed: {}", e),
            },
            "Tasks" => {
                if let Some(user) = &current_user {
                    menu(&conn, user.id() as u32).unwrap_or(());
                } else {
                    println!("No user signed in.");
                }
            },
            "Timezone" => {
                if let Some(user) = &current_user {
                    timezone_user(user.clone()).unwrap_or(());
                } else {
                    println!("No user signed in.");
                }
            },
            "Exit" => {
                println!("Goodbye!");
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

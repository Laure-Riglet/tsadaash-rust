use std::io::stdin;
use crate::cli::helpers::clear_screen;
use crate::db::repository::task::{insert, select_by_user_id};
use chrono::{NaiveDate, Weekday};
use inquire::{Confirm, DateSelect, Select, Text};
use rusqlite::{Connection, Result};

pub fn menu(conn: &Connection, user_id: u32) -> Result<(), rusqlite::Error> {
    let options = vec![
        "Create Task",
        "View Tasks",
        "Update Task",
        "Delete Task",
        "Back to Main Menu",
    ];

    loop {
        clear_screen();
        println!("=== Task Management ===");
        let choice = Select::new("Please choose an option:", options.clone())
            .prompt()
            .unwrap_or_else(|_| "Back to Main Menu");

        match choice {
            "Create Task" => {
                create_task(&conn, user_id)?;
            }
            "View Tasks" => {
                view_tasks(&conn, user_id)?;
            }
            "Update Task" => {
                println!("Updating a task...");
                // Implement task updating logic here
            }
            "Delete Task" => {
                println!("Deleting a task...");
                // Implement task deletion logic here
            }
            "Back to Main Menu" => break,
            _ => {
                println!("Invalid option. Press Enter to try again.");
                break;
            }
        }
    }

    Ok(())
}

fn create_task(conn: &Connection, user_id: u32) -> Result<()> {
    // Form
    let title = Text::new("Enter task title:")
        .with_placeholder("Type your answer here")
        .prompt()
        .unwrap_or_default();

    let is_recurring = Confirm::new("Is this task recurring?")
        .with_default(false)
        .prompt()
        .unwrap_or(false);

    let (recurrence_interval, recurrence_unit, date) = match is_recurring {
        false => {
            let naive_date = DateSelect::new("Select start date of task completion:")
                .with_starting_date(NaiveDate::from_ymd_opt(2026, 2, 3).unwrap())
                .with_min_date(NaiveDate::from_ymd_opt(2026, 2, 3).unwrap())
                .with_max_date(NaiveDate::from_ymd_opt(2026, 5, 31).unwrap())
                .with_week_start(Weekday::Mon)
                .prompt()
                .expect("Failed to get date");
            let date = Some(naive_date.format("%Y-%m-%d").to_string());
            (None, None, date)
        }
        true => {
            // Recurring task details
            let interval = Text::new("Enter recurrence interval (e.g., '2'):")
                .with_placeholder("Type your answer here")
                .prompt()
                .unwrap_or_default();

            let unit = Select::new(
                "Select recurrence unit:",
                vec!["days", "weeks", "months", "years"],
            )
            .prompt()
            .unwrap_or_else(|_| "days");
            
            (Some(interval), Some(unit.to_string()), None)
        }
    };

    // Insert into DB
    insert(
        conn,
        user_id,
        &title,
        is_recurring,
        recurrence_interval.as_deref(),
        recurrence_unit.as_deref(),
        date.as_deref(),
    )?;

    println!("Task '{}' created successfully!", title);
    println!("Press Enter to continue...");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    Ok(())
}

fn view_tasks(conn: &Connection, user_id: u32) -> Result<()> {
    let tasks = select_by_user_id(conn, user_id as i32)?;

    clear_screen();
    println!("=== Your Tasks ===");
    let mut index: u8 = 1;
    for task in tasks {
        println!(
            "{}. [{}] {} (ID: {})",
            index,
            if task.is_recurring() { "R" } else { "U" },
            task.title(),
            task.id()
        );
        index += 1;
    }

    println!("\nPress Enter to continue...");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    Ok(())
}

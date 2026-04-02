use crate::config::{printer_files_dir_path, pulse_database_path};
use anyhow::Result;
use chrono::Utc;
use cli_shared::tasks::KonanFile;
use rusqlite::{Connection, OpenFlags};
use serde_rusqlite::{columns_from_statement, from_row_with_columns, to_params_named};

pub mod schema;

const MIGRATIONS: &[&str] = &[include_str!("migrations/create_pulse.sql")];

fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    for migration in MIGRATIONS {
        conn.execute_batch(migration)?;
    }
    Ok(())
}

fn database() -> Result<Connection> {
    let db_path = pulse_database_path()?;
    if db_path.exists() {
        let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        Ok(conn)
    } else {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        run_migrations(&conn)?;
        Ok(conn)
    }
}

pub fn insert_pulse(pulse: &schema::NewPulse) -> Result<()> {
    let conn = database()?;
    conn.execute(
        "INSERT INTO pulse (name, command, start_date, r_rule, last_run) VALUES (:name, :command, :start_date, :r_rule, :last_run)",
        to_params_named(pulse)?.to_slice().as_slice(),
    )?;
    Ok(())
}

pub fn get_all_pulses() -> Result<Vec<schema::Pulse>> {
    let conn = database()?;
    let mut stmt = conn.prepare("SELECT * FROM pulse")?;
    let columns = columns_from_statement(&stmt);
    let pulses = stmt
        .query_and_then([], |row| {
            from_row_with_columns::<schema::Pulse>(row, &columns)
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(pulses)
}

pub fn delete_pulse(pulse_id: i64) -> Result<()> {
    let conn = database()?;

    let command: Option<String> = conn
        .query_row(
            "SELECT command FROM pulse WHERE id = ?1",
            rusqlite::params![pulse_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(Ok(KonanFile { name: filename, .. })) = command.map(|c| serde_json::from_str(&c)) {
        let path = printer_files_dir_path()?.join(&filename);
        let _ = std::fs::remove_file(path);
    }

    conn.execute(
        "DELETE FROM pulse WHERE id = ?1",
        rusqlite::params![pulse_id],
    )?;
    Ok(())
}

pub fn update_last_run(pulse_id: i64) -> Result<()> {
    let conn = database()?;
    let now = Utc::now().timestamp();
    conn.execute(
        "UPDATE pulse SET last_run = ?1 WHERE id = ?2",
        rusqlite::params![now, pulse_id],
    )?;
    Ok(())
}

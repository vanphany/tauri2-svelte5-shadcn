use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, Pool, Sqlite};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::ErrorKind,
};
use tauri::{command, AppHandle, Emitter, Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
pub enum Status {
    Incomplete,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Todo {
    id: u16,
    title: String,
    description: String,
    status: Status,
}

type Db = Pool<Sqlite>;
pub struct AppState {
    pub db: Db,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseStatus {
    status: String,
}

pub async fn setup_db(handle: &AppHandle) -> Result<(), Box<dyn Error>> {
    let path = handle.path().app_data_dir()?;
    fs::create_dir_all(&path)?;
    let mut db_path = path.clone();
    db_path.push("data.db");
    println!("database path: {:?}", db_path);
    match OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&db_path)
    {
        Ok(_) => println!("database file created"),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            println!("database file already exists")
        }
        Err(e) => return Err(format!("Failed to create database file: {}", e).into()),
    }
    let db_path_str = db_path.to_str().ok_or("Failed to convert path to string")?;
    let db: Pool<Sqlite> = SqlitePoolOptions::new().connect(db_path_str).await?;
    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
          id INTEGER PRIMARY KEY,
          title TEXT,
          description TEXT,
          status VARCHAR(30)
        )
        "#,
    )
    .execute(&db)
    .await;
    match result {
        Ok(_) => {
            handle.manage(AppState { db });
            handle.emit(
                "dbstatus",
                DatabaseStatus {
                    status: "ready".to_string(),
                },
            )?;
        }
        Err(e) => handle.emit(
            "dbstatus",
            DatabaseStatus {
                status: e.to_string(),
            },
        )?,
    }
    Ok(())
}

#[command]
pub async fn add_todo(
    state: State<'_, AppState>,
    title: &str,
    description: &str,
) -> Result<Todo, String> {
    let db = &state.db;
    let todo: (i64,) = sqlx::query_as(
        "INSERT INTO todos (title, description, status) VALUES (?1, ?2, ?3) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(Status::Incomplete)
    .fetch_one(db)
    .await
    .map_err(|e| format!("Error saving todo: {}", e))?;
    Ok(Todo {
        id: todo.0 as u16,
        title: title.to_owned(),
        description: description.to_owned(),
        status: Status::Incomplete,
    })
}

#[command]
pub async fn get_todos(state: State<'_, AppState>) -> Result<Vec<Todo>, String> {
    let db = &state.db;
    let todos: Vec<Todo> = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
        .fetch(db)
        .try_collect()
        .await
        .map_err(|e| format!("Failed to get todos {}", e))?;
    Ok(todos)
}

#[command]
pub async fn update_todo(
    state: State<'_, AppState>,
    todo: Todo,
) -> Result<Todo, (String, Option<Todo>)> {
    let db = &state.db;

    let prev_state = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = ?1")
        .bind(todo.id)
        .fetch_one(db)
        .await
        .map_err(|e| {
            (
                format!(
                    "Failed to fetch previous todo state (possible data loss): {}",
                    e
                ),
                None,
            )
        })?;

    let title = if todo.title.trim().is_empty() {
        prev_state.title.clone()
    } else {
        todo.title.clone()
    };

    let description = if todo.description.trim().is_empty() {
        prev_state.description.clone()
    } else {
        todo.description.clone()
    };

    sqlx::query("UPDATE todos SET title = ?1, description = ?2, status = ?3 WHERE id = ?4")
        .bind(&title)
        .bind(&description)
        .bind(&todo.status)
        .bind(todo.id)
        .execute(db)
        .await
        .map_err(|e| (format!("Failed to update todo: {}", e), Some(prev_state)))?;

    Ok(Todo {
        id: todo.id,
        title,
        description,
        status: todo.status,
    })
}

#[command]
pub async fn delete_todo(
    state: State<'_, AppState>,
    id: u16,
) -> Result<(), (String, Option<Todo>)> {
    let db = &state.db;

    let prev_state = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = ?1")
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(|e| {
            (
                format!(
                    "Failed to fetch todo for deletion (possible data loss): {}",
                    e
                ),
                None,
            )
        })?;

    sqlx::query("DELETE FROM todos WHERE id = ?1")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| (format!("Failed to delete todo: {}", e), Some(prev_state)))?;

    Ok(())
}

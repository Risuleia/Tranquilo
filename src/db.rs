use std::str::FromStr;
use slint::SharedString;
use sqlite::Connection;
use chrono::Utc;

use crate::{slint_generatedAppWindow, TaskStatus};

impl ToString for TaskStatus {
    fn to_string(&self) -> String {
        match self {
            TaskStatus::Pending => "Pending".to_string(),
            TaskStatus::Completed => "Completed".to_string(),
            TaskStatus::Cancelled => "Cancelled".to_string()
        }
    }
}
impl FromStr for TaskStatus {
    type Err = ();

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(TaskStatus::Pending),
            "Completed" => Ok(TaskStatus::Completed),
            "Cancelled" => Ok(TaskStatus::Cancelled),
            _ => Err(())
        }
    }
}

pub struct Task {
    pub id: String,
    pub text: String,
    pub status: TaskStatus,
    pub timestamp: String
}

pub struct Database {
    connection: Connection
}

impl Database {
    pub fn new(path: &str) -> Self {
        let connection = Connection::open(path).unwrap();
        Database { connection }
    }

    pub fn init_db(&self) {
        let _ = self.connection.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY NOT NULL,
                text TEXT NOT NULL,
                status TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )"
        );
    }

    pub fn insert_task(&self, task: &Task) -> Result<(), sqlite::Error> {
        let query = format!("INSERT INTO tasks (id, text, status, timestamp) VALUES ('{}', '{}', '{}', '{}')",
            task.id,
            task.text,
            task.status.to_string(),
            task.timestamp
        );

        self.connection.execute(&query)?;
        Ok(())
    }


    pub fn fetch_tasks(&self) -> Result<Vec<slint_generatedAppWindow::Task>, sqlite::Error> {
        let today = Utc::now().date_naive();

        let stmt = format!("SELECT * FROM tasks WHERE timestamp LIKE '%{}%' ORDER BY id DESC", today.to_string());
        let query = self.connection.prepare(stmt.clone())?;

        let mut tasks = Vec::new();

        for row in query.into_iter() {
            let task_result = row.and_then(|row| {
                Ok(slint_generatedAppWindow::Task {
                    id: SharedString::from(row.read::<&str, usize>(0)),
                    text: SharedString::from(row.read::<&str, usize>(1)),
                    status: TaskStatus::from_str(row.read::<&str, usize>(2)).unwrap()
                })
            });

            match task_result {
                Ok(task) => tasks.push(task),
                Err(err) => return Err(err)
            }
        };

        Ok(tasks)
    }

    pub fn update_task_state(&self, id: String, new_status: TaskStatus) -> Result<(), sqlite::Error> {
        let stmt = format!("UPDATE tasks SET status = '{}' WHERE id = '{}'", new_status.to_string(), id);
        self.connection.execute(stmt)?;
        Ok(())
    }

    pub fn remove_task(&self, id: String) -> Result<(), sqlite::Error> {
        let stmt = format!("DELETE FROM tasks WHERE id = '{}'", id);
        self.connection.execute(stmt)?;
        Ok(())
    }
}
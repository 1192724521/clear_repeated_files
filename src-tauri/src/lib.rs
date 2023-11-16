use async_recursion::async_recursion;
use serde::Serialize;
use sqlx::{FromRow, Pool, Sqlite};
use std::{
    fs,
    sync::{Mutex, OnceLock},
    time::UNIX_EPOCH,
};
pub mod commands;

#[derive(Debug, FromRow, Serialize)]
struct FileInfo {
    id: Option<i64>,
    path: String,
    len: i64,
    created_time: String,
    modified_time: String,
    sha1: Option<String>,
}

static USED_MEM: Mutex<usize> = Mutex::new(0);

static PROGRESS_PERCENT: Mutex<f64> = Mutex::new(0.0);

static COUNT: Mutex<usize> = Mutex::new(0);

pub static POOL: OnceLock<Pool<Sqlite>> = OnceLock::new();

async fn select_all() -> Vec<FileInfo> {
    sqlx::query_as("SELECT * from fileInfo")
        .fetch_all(POOL.get().unwrap())
        .await
        .unwrap()
}

async fn clear_not_exists_file_info() {
    let file_infos = select_all().await;
    for i in file_infos {
        let fs = fs::File::open(i.path);
        match fs {
            Ok(_) => {}
            Err(_) => {
                sqlx::query(
                    r#"
DELETE FROM fileInfo
WHERE
    id = ?
                "#,
                )
                .bind(i.id)
                .execute(POOL.get().unwrap())
                .await
                .unwrap();
            }
        }
    }
}

#[async_recursion]
async fn walk_dir(path: &str) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        *COUNT.lock().unwrap() += 1;
        if !entry.path().is_dir() {
            let fileinfo = FileInfo {
                id: None,
                path: entry.path().to_str().unwrap().to_string(),
                len: entry.metadata().unwrap().len() as _,
                created_time: entry
                    .metadata()
                    .unwrap()
                    .created()
                    .unwrap()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
                modified_time: entry
                    .metadata()
                    .unwrap()
                    .modified()
                    .unwrap()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
                sha1: None,
            };
            let selectvalue = sqlx::query_as::<_, FileInfo>(
                r#"
SELECT
    *
FROM
    fileInfo
WHERE
    path = ?"#,
            )
            .bind(&fileinfo.path)
            .fetch_one(POOL.get().unwrap())
            .await;
            match selectvalue {
                Ok(value) => {
                    if value.modified_time != fileinfo.modified_time {
                        sqlx::query(
                            r#"
UPDATE fileInfo
SET
    len = ?,
    created_time = ?,
    modified_time = ?,
    sha1 = ?
WHERE
    id = ?"#,
                        )
                        .bind(fileinfo.len)
                        .bind(fileinfo.created_time)
                        .bind(fileinfo.modified_time)
                        .bind(fileinfo.sha1)
                        .bind(value.id)
                        .execute(POOL.get().unwrap())
                        .await
                        .unwrap();
                    }
                }
                Err(_) => {
                    sqlx::query(
                        r#"
INSERT INTO
    fileInfo
VALUES
    (NULL, ?, ?, ?, ?, NULL)"#,
                    )
                    .bind(fileinfo.path)
                    .bind(fileinfo.len)
                    .bind(fileinfo.created_time)
                    .bind(fileinfo.modified_time)
                    .execute(POOL.get().unwrap())
                    .await
                    .unwrap();
                }
            }
        } else {
            walk_dir(entry.path().to_str().unwrap()).await;
        }
    }
}

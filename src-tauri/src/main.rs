// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, error::Error, fs, time::Duration};

use clear_repeated_files::{
    commands::{
        __cmd__caculate_sha1, __cmd__delete_files, __cmd__get_datas, __cmd__get_progress_percent,
        __cmd__get_walkfile_count, caculate_sha1, delete_files, get_datas, get_progress_percent,
        get_walkfile_count,
    },
    POOL,
};
use sqlx::sqlite::SqlitePoolOptions;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .setup(|_| init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_datas,
            caculate_sha1,
            delete_files,
            get_progress_percent,
            get_walkfile_count
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init() -> Result<(), Box<dyn Error>> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // 系统临时文件夹
        let mut file_path = env::temp_dir();
        file_path.push("quchong");
        file_path.push("datas.db");
        // 如果文件不存在，则创建
        if !file_path.exists() {
            fs::File::create(&file_path).unwrap();
        }
        let pool = SqlitePoolOptions::new()
            .acquire_timeout(Duration::from_secs(60 * 10))
            .connect(&format!("sqlite://{}", file_path.to_str().unwrap()))
            .await
            .unwrap();

        sqlx::query(
            r#"
CREATE TABLE
    IF NOT EXISTS fileInfos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        path TEXT NOT NULL UNIQUE,
        len INTEGER NOT NULL,
        created_time TEXT NOT NULL,
        modified_time TEXT NOT NULL,
        sha1 TEXT
    );
        "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        POOL.set(pool).unwrap();
        Ok(())
    })
}

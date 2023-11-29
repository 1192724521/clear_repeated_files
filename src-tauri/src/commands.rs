use std::{collections::HashMap, env, fs};

use sha1::{Digest, Sha1};
use sysinfo::SystemExt;
use tokio::task::JoinSet;

use crate::{
    clear_not_exists_file_info,
    sql::{GET_DATAS, GET_FILE_INFOS_BY_PATH, SET_FILEINFOS_SHA1_BY_ID},
    walk_dir, FileInfo, COUNT, POOL, PROGRESS_PERCENT, USED_MEM,
};

#[tauri::command]
pub async fn get_datas(path: String) -> Result<String, ()> {
    clear_not_exists_file_info().await;

    *COUNT.lock().unwrap() = 0;

    walk_dir(&path).await;

    let mut file_infos: Vec<FileInfo> = sqlx::query_as(GET_DATAS)
        .bind(path)
        .fetch_all(POOL.get().unwrap())
        .await
        .unwrap();

    file_infos.sort_by(|a, b| b.path.cmp(&a.path));
    file_infos.sort_by(|a, b| a.sha1.cmp(&b.sha1));

    Ok(serde_json::to_string(&file_infos).unwrap())
}

#[tauri::command]
pub async fn caculate_sha1(path: String) -> Result<String, ()> {
    // 获取系统信息
    let sysinfo = sysinfo::System::new_all();
    // 系统内存大小
    let sys_mem = sysinfo.total_memory();
    // 程序可以使用最大内存限制
    let max_process_mem = (sys_mem as f64 * 0.8) as usize;

    let mut file_path = env::temp_dir();
    file_path.push("quchong");
    file_path.push("datas.db");

    let fileinfos: Vec<FileInfo> = sqlx::query_as(GET_FILE_INFOS_BY_PATH)
        .bind(path)
        .fetch_all(POOL.get().unwrap())
        .await
        .unwrap();

    let mut repetitions_count: HashMap<i64, i64> = HashMap::new();
    for i in &fileinfos {
        let count = repetitions_count.get_mut(&i.len);
        match count {
            Some(a) => *a += 1,
            None => {
                repetitions_count.insert(i.len, 1);
            }
        }
    }

    *COUNT.lock().unwrap() = 0;
    let total_file = fileinfos.len();
    let mut handles = JoinSet::new();
    for i in fileinfos {
        if i.sha1.is_none() {
            if repetitions_count.get(&i.len).unwrap() > &1 {
                // 如果内存大于等于程序允许使用的最大内存，阻塞等待一个任务完成
                while *USED_MEM.lock().unwrap() >= max_process_mem {
                    handles.join_next().await;
                }
                handles.spawn(async move {
                    let data = fs::read(&i.path).unwrap();
                    // 使用了多少内存
                    *USED_MEM.lock().unwrap() += data.len();
                    let mut hash = Sha1::new();
                    hash.update(&data);
                    sqlx::query(SET_FILEINFOS_SHA1_BY_ID)
                        .bind(format!("{:x}", hash.finalize()))
                        .bind(i.id)
                        .execute(POOL.get().unwrap())
                        .await
                        .unwrap();
                    // 释放内存
                    *USED_MEM.lock().unwrap() -= data.len();
                    *COUNT.lock().unwrap() += 1;
                    *PROGRESS_PERCENT.lock().unwrap() =
                        *COUNT.lock().unwrap() as f64 / total_file as f64 * 100.0;
                });
            } else {
                *COUNT.lock().unwrap() += 1;
            }
        } else {
            *COUNT.lock().unwrap() += 1;
        }

        *PROGRESS_PERCENT.lock().unwrap() =
            *COUNT.lock().unwrap() as f64 / total_file as f64 * 100.0;
    }

    while let Some(_) = handles.join_next().await {}

    Ok("计算完成".to_string())
}

#[tauri::command]
pub fn delete_files(vec_paths: Vec<String>) -> String {
    for i in vec_paths {
        fs::remove_file(i).unwrap_or_default();
    }
    "删除成功".to_string()
}

#[tauri::command]
pub fn get_progress_percent() -> f64 {
    *PROGRESS_PERCENT.lock().unwrap()
}

#[tauri::command]
pub fn get_walkfile_count() -> usize {
    *COUNT.lock().unwrap()
}

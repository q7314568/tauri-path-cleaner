use tauri::command;
use tauri::{Emitter, EventTarget};
use std::fs;
use std::path::Path;

// 定義刪除資料夾的命令，這個命令可以被前端調用
#[command]
fn delete_paths_with_progress<R: tauri::Runtime>(app_handle: tauri::AppHandle<R>, paths: Vec<String>) -> Result<(), String> {
    let mut total_files = 0;
    let mut deleted_files = 0;

    // 計算所有資料夾和檔案的總數量
    for path_str in &paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("路徑不存在: {}", path.display()));
        }
        if path.is_dir() {
            total_files += count_files_in_directory(path.to_str().unwrap().to_string()).unwrap_or(0);
        } else if path.is_file() {
            total_files += 1;
        }
    }

    // 開始刪除所有檔案和資料夾
    for path_str in paths {
        let path = Path::new(&path_str);
        if path.is_file() {
            fs::remove_file(&path).map_err(|e| format!("刪除檔案失敗: {}", e))?;
            deleted_files += 1;
        } else if path.is_dir() {
            let entries = fs::read_dir(path).map_err(|e| format!("無法讀取資料夾: {}", e))?;
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        fs::remove_file(&entry_path).map_err(|e| format!("刪除檔案失敗: {}", e))?;
                    } else if entry_path.is_dir() {
                        fs::remove_dir_all(&entry_path).map_err(|e| format!("刪除資料夾失敗: {}", e))?;
                    }
                    deleted_files += 1;

                    // 計算並發送進度更新
                    let progress = (deleted_files as f64 / total_files as f64) * 100.0;
                    app_handle.emit_to("main", "delete-progress", progress).map_err(|e| format!("發送進度事件失敗: {}", e))?;
                }
            }
            // 最後刪除資料夾本身
            fs::remove_dir_all(&path).map_err(|e| format!("刪除資料夾失敗: {}", e))?;
            deleted_files += 1;
        } else {
            return Err("無法識別的路徑類型".into());
        }

        // 每刪除一個項目都發送進度更新
        let progress = (deleted_files as f64 / total_files as f64) * 100.0;
        app_handle.emit_to("main", "delete-progress", progress).map_err(|e| format!("發送進度事件失敗: {}", e))?;
    }

    Ok(())
}

#[command]
fn count_files_in_directory(directory_path: String) -> Result<usize, String> {
    let path = Path::new(&directory_path);
    if !path.exists() {
        return Err(format!("路徑不存在: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("不是一個有效的資料夾: {}", path.display()));
    }
    let mut file_count = 0;
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        file_count += 1;
                    } else if entry_path.is_dir() {
                        match count_files_in_directory(entry_path.to_str().unwrap().to_string()) {
                            Ok(count) => file_count += count,
                            Err(_) => continue,
                        }
                    }
                }
            }
            Ok(file_count)
        }
        Err(e) => Err(format!("無法讀取資料夾: {}: {}", path.display(), e)),
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() ->Result<(), String> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![delete_paths_with_progress, count_files_in_directory])
        .run(tauri::generate_context!())
        .map_err(|e| format!("Error while running tauri application: {}", e))
}

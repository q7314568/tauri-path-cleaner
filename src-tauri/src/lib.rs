use tauri::command;
use tauri::{Emitter, EventTarget};
use std::fs;
use std::path::Path;

fn count_entries(path: &Path) -> Result<usize, String> {
    if path.is_file() {
        return Ok(1);
    }

    if path.is_dir() {
        let mut count = 1; // 計入資料夾本身
        for entry in fs::read_dir(path).map_err(|e| format!("無法讀取資料夾: {}", e))? {
            let entry = entry.map_err(|e| format!("無法讀取資料夾項目: {}", e))?;
            count += count_entries(&entry.path())?;
        }
        Ok(count)
    } else {
        Err("無法識別的路徑類型".into())
    }
}

fn delete_entry_with_progress<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    path: &Path,
    total_files: usize,
    deleted_files: &mut usize,
) -> Result<(), String> {
    if path.is_file() {
        fs::remove_file(path).map_err(|e| format!("刪除檔案失敗: {}", e))?;
        *deleted_files += 1;
        let progress = (*deleted_files as f64 / total_files as f64) * 100.0;
        app_handle
            .emit_to("main", "delete-progress", progress)
            .map_err(|e| format!("發送進度事件失敗: {}", e))?;
    } else if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| format!("無法讀取資料夾: {}", e))? {
            let entry = entry.map_err(|e| format!("無法讀取資料夾項目: {}", e))?;
            delete_entry_with_progress(app_handle, &entry.path(), total_files, deleted_files)?;
        }
        fs::remove_dir(path).map_err(|e| format!("刪除資料夾失敗: {}", e))?;
        *deleted_files += 1;
        let progress = (*deleted_files as f64 / total_files as f64) * 100.0;
        app_handle
            .emit_to("main", "delete-progress", progress)
            .map_err(|e| format!("發送進度事件失敗: {}", e))?;
    }
    Ok(())
}

// 定義刪除資料夾的命令，這個命令可以被前端調用
#[command]
fn delete_paths_with_progress<R: tauri::Runtime>(app_handle: tauri::AppHandle<R>, paths: Vec<String>) -> Result<(), String> {
    let mut total_files = 0;
    let mut deleted_files = 0;

    // 計算所有資料夾和檔案的總數量（包含資料夾本身）
    for path_str in &paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("路徑不存在: {}", path.display()));
        }
        total_files += count_entries(path)?;
    }

    // 開始刪除所有檔案和資料夾
    for path_str in paths {
        let path = Path::new(&path_str);
        delete_entry_with_progress(&app_handle, path, total_files, &mut deleted_files)?;
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
    count_entries(path)
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

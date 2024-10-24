use tauri::command;
use std::fs;
use std::path::Path;

// 定義刪除資料夾的命令，這個命令可以被前端調用
#[command]
fn delete_path(path: String) -> Result<(), String> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err(format!("路徑不存在: {}", path.display()));
    }

    let metadata = fs::metadata(&path).map_err(|e| format!("無法取得檔案或資料夾資訊: {}", e))?;
    if metadata.is_dir() {
        match fs::remove_dir_all(&path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("刪除資料夾失敗: {}", e)),
        }
    } else if metadata.is_file() {
        match fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("刪除檔案失敗: {}", e)),
        }
    } else {
        Err("無法識別的路徑類型".into())
    }
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
        .invoke_handler(tauri::generate_handler![delete_path, count_files_in_directory])
        .run(tauri::generate_context!())
        .map_err(|e| format!("Error while running tauri application: {}", e))
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// src-tauri/src/main.rs
use std::process::Command;
use std::fs::OpenOptions;
use std::io::Write;
use tauri_path_cleaner::run;
use log::error;
use env_logger;

fn request_admin_permission() -> Result<(), String> {
    if cfg!(debug_assertions) {
        // 在開發模式中跳過管理員權限檢查
        println!("開發模式中，跳過權限檢查。");
        return Ok(());
    }
    
    if cfg!(target_os = "windows") {
        let output = Command::new("cmd")
            .arg("/C")
            .arg("whoami /priv")
            .output()
            .map_err(|e| format!("無法執行 whoami /priv: {}", e))?;
        let output_str = String::from_utf8_lossy(&output.stdout);
        if !output_str.contains("SeBackupPrivilege") {
            return Err("需要管理員權限來刪除系統資料夾。請以管理員身份運行應用程序。".into());
        }
    } else if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        // 在 macOS 或 Linux 上，檢查是否以 root 權限啟動
        if !is_root() {
            return Err("需要 root 權限來刪除系統資料夾。請以 root 身份運行應用程序。".into());
        }
    }
    Ok(())
}

fn is_root() -> bool {
    match std::env::var("USER") {
        Ok(user) => user == "root",
        Err(_) => false,
    }
}

fn log_to_file(message: &str) {
    let log_file_path = "application.log";
    if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(log_file_path) {
        if let Err(e) = writeln!(file, "{}", message) {
            eprintln!("無法寫入日誌文件: {}", e);
        }
    } else {
        eprintln!("無法打開或創建日誌文件");
    }
}

fn main() {
    env_logger::init();
    if let Err(e) = request_admin_permission() {
        let error_message = format!("需要管理員權限來刪除資料夾: {}", e);
        eprintln!("{}", error_message);
        log_to_file(&error_message);
        return;
    }
    if let Err(e) = run() {
        let error_message = format!("Application error: {}", e);
        error!("{}", error_message);
        log_to_file(&error_message);
    }
}

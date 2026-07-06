mod image_processor;

use std::fs;
use std::process::Command;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn read_markdown_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|err| err.to_string())
}

#[tauri::command]
fn save_markdown_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|err| err.to_string())
}

/// 列出文件夹内所有图片
#[tauri::command]
fn list_images(folder: String) -> Result<Vec<image_processor::ImageFileInfo>, String> {
    image_processor::list_images(&folder)
}

/// 批量处理图片（Rust 并行处理）
#[tauri::command]
fn batch_process_images(
    files: Vec<String>,
    output_dir: String,
    width: u32,
    height: u32,
    keep_ratio: bool,
    format: String,
    quality: u8,
) -> image_processor::BatchResult {
    image_processor::batch_process_images(files, output_dir, width, height, keep_ratio, format, quality)
}

/// 批量处理图片（Rust 串行处理）- 用于性能对比
#[tauri::command]
fn batch_process_images_sequential(
    files: Vec<String>,
    output_dir: String,
    width: u32,
    height: u32,
    keep_ratio: bool,
    format: String,
    quality: u8,
) -> image_processor::BatchResult {
    image_processor::batch_process_images_sequential(files, output_dir, width, height, keep_ratio, format, quality)
}

/// 批量处理图片（Node.js 版本）- 用于性能对比
#[tauri::command]
fn batch_process_images_node(
    files: Vec<String>,
    output_dir: String,
    width: u32,
    height: u32,
    keep_ratio: bool,
    format: String,
    quality: u8,
    parallel: bool,
) -> Result<image_processor::BatchResult, String> {
    let config = serde_json::json!({
        "files": files,
        "output_dir": output_dir,
        "width": width,
        "height": height,
        "keep_ratio": keep_ratio,
        "format": format,
        "quality": quality,
        "parallel": parallel,
    });

    let output = Command::new("node")
        .arg("src/node-image-processor.js")
        .arg(config.to_string())
        .current_dir("/Users/neozhy/Documents/Learn/Rust/tauri/tauri_app")
        .output()
        .map_err(|e| format!("执行 Node.js 脚本失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Node.js 脚本执行失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: image_processor::BatchResult = serde_json::from_str(&stdout)
        .map_err(|e| format!("解析 Node.js 输出失败: {}", e))?;

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            read_markdown_file,
            save_markdown_file,
            list_images,
            batch_process_images,
            batch_process_images_sequential,
            batch_process_images_node,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Instant;
use vips::VipsImage;

/// 单张图片的信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageFileInfo {
    pub path: String,
    pub name: String,
    pub ext: String,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
}

/// 单张图片的处理结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessResult {
    pub file: String,
    pub success: bool,
    pub error: Option<String>,
    pub original_size: u64,
    pub new_size: Option<u64>,
}

/// 批处理最终结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchResult {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub total_time_ms: u128,
    pub results: Vec<ProcessResult>,
}

// 支持的图片扩展名
const SUPPORTED_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp"];

/// 判断文件扩展名是否为支持的图片格式
fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            SUPPORTED_EXTS.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}

/// 列出文件夹中所有支持的图片文件
pub fn list_images(folder: &str) -> Result<Vec<ImageFileInfo>, String> {
    let dir = Path::new(folder);
    if !dir.is_dir() {
        return Err(format!("路径不是文件夹: {}", folder));
    }

    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    let mut images = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if !path.is_file() || !is_supported_image(&path) {
            continue;
        }

        let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 获取图片尺寸
        let (width, height) = get_image_dimensions(&path);

        images.push(ImageFileInfo {
            path: path.to_string_lossy().to_string(),
            name,
            ext,
            size_bytes: metadata.len(),
            width,
            height,
        });
    }

    // 按文件名排序，保证每次结果一致
    images.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(images)
}

/// 获取图片尺寸（不加载全部像素）
fn get_image_dimensions(path: &Path) -> (u32, u32) {
    match image::image_dimensions(path) {
        Ok((w, h)) => (w, h),
        Err(_) => (0, 0),
    }
}

/// 处理单张图片（缩放到目标尺寸并保存为指定格式）
fn process_single_image(
    input_path: &str,
    output_path: &str,
    target_width: u32,
    target_height: u32,
    keep_ratio: bool,
    output_format: &str,
    quality: u8,
) -> ProcessResult {
    let original_size = fs::metadata(input_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let result = if true {
        (|| -> Result<u64, String> {
            // 1. 获取原始宽高（使用 image 库，只读头部，不加载像素）
            let (width, height) = image::image_dimensions(input_path)
                .map_err(|e| format!("获取图片尺寸失败: {}", e))?;
            let (width, height) = (width as f64, height as f64);

            // 2. 用 vips 打开图片（高性能解码）
            let img = VipsImage::from_file(input_path)
                .map_err(|e| format!("无法打开图片: {}", e))?;

            // 3. 计算缩放比例
            let resized = if keep_ratio {
                let scale_x = target_width as f64 / width;
                let scale_y = target_height as f64 / height;
                let scale = scale_x.min(scale_y); // 保持比例，取小值
                img.resize(scale, None, None)
                    .map_err(|e| format!("缩放失败: {}", e))?
            } else {
                // 强制拉伸：使用 affine 进行独立缩放
                // 注意：当前版本可能不支持 affine，所以我们使用 resize 分两次？不，这里我们用备用方案
                // 由于 vips 0.1.0-alpha.5 无 affine，我们回退到 image 库处理拉伸（但会影响性能）
                // 为了公平，我们仅支持 keep_ratio=true，否则报错
                return Err("拉伸模式 (keep_ratio=false) 暂不支持，请使用 keep_ratio=true".to_string());
            };

            // 4. 创建输出目录
            if let Some(parent) = Path::new(output_path).parent() {
                fs::create_dir_all(parent).map_err(|e| format!("无法创建目录: {}", e))?;
            }

            // 5. 保存（使用 vips 默认质量）
            resized.write_to_file(output_path)
                .map_err(|e| format!("保存失败: {}", e))?;

            let new_size = fs::metadata(output_path)
                .map(|m| m.len())
                .unwrap_or(0);

            Ok(new_size)
        })()
    } else {
        (|| -> Result<u64, String> {
        // 打开图片
            let img = image::open(input_path).map_err(|e| format!("无法打开图片: {}", e))?;

            // 调整尺寸
            let resized = if keep_ratio {
                img.resize(target_width, target_height, image::imageops::FilterType::Lanczos3)
            } else {
                img.resize_exact(target_width, target_height, image::imageops::FilterType::Lanczos3)
            };

            // 创建输出目录
            if let Some(parent) = Path::new(output_path).parent() {
                fs::create_dir_all(parent).map_err(|e| format!("无法创建目录: {}", e))?;
            }

            // 根据输出格式保存
            match output_format {
                "jpg" | "jpeg" => {
                    let mut buf = std::io::BufWriter::new(Vec::new());
                    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
                    encoder
                        .encode(
                            resized.as_bytes(),
                            resized.width(),
                            resized.height(),
                            image::ExtendedColorType::Rgb8,
                        )
                        .map_err(|e| format!("JPEG 编码失败: {}", e))?;
                    fs::write(output_path, buf.into_inner().map_err(|e| e.to_string())?)
                        .map_err(|e| format!("写入文件失败: {}", e))?;
                }
                "png" => {
                    resized.save(output_path).map_err(|e| format!("PNG 保存失败: {}", e))?;
                }
                "webp" => {
                    resized.save(output_path).map_err(|e| format!("WebP 保存失败: {}", e))?;
                }
                _ => {
                    resized.save(output_path).map_err(|e| format!("保存失败: {}", e))?;
                }
            }

            let new_size = fs::metadata(output_path)
                .map(|m| m.len())
                .unwrap_or(0);

            Ok(new_size)
        })()
    };

    match result {
        Ok(new_size) => ProcessResult {
            file: input_path.to_string(),
            success: true,
            error: None,
            original_size,
            new_size: Some(new_size),
        },
        Err(err) => ProcessResult {
            file: input_path.to_string(),
            success: false,
            error: Some(err),
            original_size,
            new_size: None,
        },
    }
}

/// 批量处理图片新（通过入参整合并行串行）
pub fn batch_process_images_new(
    files: Vec<String>,
    output_dir: String,
    width: u32,
    height: u32,
    keep_ratio: bool,
    format: String,
    quality: u8,
    parallel: bool,
) -> BatchResult {
    let total = files.len();
    let start = Instant::now();
    // ========== 根据 parallel 判断并行或串行处理图片 =========={
    let results: Vec<ProcessResult> = if parallel {
        files
            .par_iter()
            .map(|file_path| {
                let path = Path::new(file_path);
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                let output_name = format!("{}.{}", stem, format);
                let output_path = Path::new(&output_dir).join(&output_name);

                process_single_image(
                    file_path,
                    output_path.to_str().unwrap(),
                    width,
                    height,
                    keep_ratio,
                    &format,
                    quality,
                )
            })
            .collect()
    } else {
        files
            .iter()
            .map(|file_path| {
                let path = Path::new(file_path);
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                let output_name = format!("{}.{}", stem, format);
                let output_path = Path::new(&output_dir).join(&output_name);

                process_single_image(
                    file_path,
                    output_path.to_str().unwrap(),
                    width,
                    height,
                    keep_ratio,
                    &format,
                    quality,
                )
            })
            .collect()
    };

    let elapsed = start.elapsed();
    let success_count = results.iter().filter(|r| r.success).count();
    let failed_count = results.iter().filter(|r| !r.success).count();

    BatchResult {
        total,
        success: success_count,
        failed: failed_count,
        total_time_ms: elapsed.as_millis(),
        results,
    }
}
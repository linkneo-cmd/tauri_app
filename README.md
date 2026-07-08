# Tauri + Vue + TypeScript

Tauri框架自学记录

## MarkDown编辑器
  核心依赖 marked npm 包

## 图片批处理压缩
  对比 Rust 并行、串行， Node 并行、串行
  Rust 核心依赖 rayon， Node 核心依赖 sharp
  注：Node 使用了基于 C++ 的高性能库 Sharp（libvips），Rust 使用了安全的纯原生库。 实际对比中 Node 效果反而更佳，待更新更合理的对比方式
  260708，更新rust使用的底层库为vips，对比更具合理性，图片数量越多、体积越大，rust优势越明显
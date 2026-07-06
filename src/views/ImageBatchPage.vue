<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

// ── 类型定义 ──
interface ImageFileInfo {
  path: string;
  name: string;
  ext: string;
  size_bytes: number;
  width: number;
  height: number;
}

interface ProcessResult {
  file: string;
  success: boolean;
  error: string | null;
  original_size: number;
  new_size: number | null;
}

interface BatchResult {
  total: number;
  success: number;
  failed: number;
  total_time_ms: number;
  results: ProcessResult[];
}

interface ModeResult {
  mode: string;
  label: string;
  result: BatchResult | null;
  status: "pending" | "running" | "completed" | "error";
}

const folderPath = ref("");
const images = ref<ImageFileInfo[]>([]);
const loading = ref(false);
const processing = ref(false);

// 处理选项
const targetWidth = ref(720);
const targetHeight = ref(480);
const keepRatio = ref(true);
const format = ref("jpg");
const quality = ref(80);

// 选中状态
const selectedFiles = ref<Set<string>>(new Set());
const selectAll = ref(true);

const processingMode = ref<"rust-parallel" | "rust-sequential" | "node-parallel" | "node-sequential" | "compare-all">("rust-parallel");

const modeResults = ref<ModeResult[]>([
  { mode: "rust-parallel", label: "Rust 并行", result: null, status: "pending" },
  { mode: "rust-sequential", label: "Rust 串行", result: null, status: "pending" },
  { mode: "node-parallel", label: "Node.js 并行", result: null, status: "pending" },
  { mode: "node-sequential", label: "Node.js 串行", result: null, status: "pending" },
]);

async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "选择图片文件夹",
  });
  if (!selected) return;
  folderPath.value = selected as string;
  await loadImages();
}

async function loadImages() {
  if (!folderPath.value) return;
  loading.value = true;
  try {
    const list = await invoke<ImageFileInfo[]>("list_images", {
      folder: folderPath.value,
    });
    images.value = list;
    selectedFiles.value = new Set(list.map((img) => img.path));
  } catch (error) {
    alert(`加载图片失败: ${error}`);
  } finally {
    loading.value = false;
  }
}

function toggleSelectAll() {
  selectAll.value = !selectAll.value;
  if (selectAll.value) {
    selectedFiles.value = new Set(images.value.map((img) => img.path));
  } else {
    selectedFiles.value = new Set();
  }
}

function toggleFile(path: string) {
  const newSet = new Set(selectedFiles.value);
  if (newSet.has(path)) {
    newSet.delete(path);
  } else {
    newSet.add(path);
  }
  selectedFiles.value = newSet;
  selectAll.value = newSet.size === images.value.length;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

async function runRustParallel(files: string[], outputDir: string): Promise<BatchResult> {
  return invoke<BatchResult>("batch_process_images", {
    files,
    outputDir,
    width: targetWidth.value,
    height: targetHeight.value,
    keepRatio: keepRatio.value,
    format: format.value,
    quality: quality.value,
  });
}

async function runRustSequential(files: string[], outputDir: string): Promise<BatchResult> {
  return invoke<BatchResult>("batch_process_images_sequential", {
    files,
    outputDir,
    width: targetWidth.value,
    height: targetHeight.value,
    keepRatio: keepRatio.value,
    format: format.value,
    quality: quality.value,
  });
}

async function runNode(files: string[], outputDir: string, parallel: boolean): Promise<BatchResult> {
  return invoke<BatchResult>("batch_process_images_node", {
    files,
    outputDir,
    width: targetWidth.value,
    height: targetHeight.value,
    keepRatio: keepRatio.value,
    format: format.value,
    quality: quality.value,
    parallel,
  });
}

async function startProcessing() {
  const files = Array.from(selectedFiles.value);
  if (files.length === 0) {
    alert("请先选择要处理的图片");
    return;
  }

  processing.value = true;
  modeResults.value = modeResults.value.map((m) => ({ ...m, result: null, status: "pending" }));

  const modesToRun = processingMode.value === "compare-all"
    ? ["rust-parallel", "rust-sequential", "node-parallel", "node-sequential"]
    : [processingMode.value];

  try {
    for (const mode of modesToRun) {
      const modeResult = modeResults.value.find((m) => m.mode === mode);
      if (!modeResult) continue;

      modeResult.status = "running";

      const outputDir = `${folderPath.value}/processed_${mode}`;

      let result: BatchResult;
      switch (mode) {
        case "rust-parallel":
          result = await runRustParallel(files, outputDir);
          break;
        case "rust-sequential":
          result = await runRustSequential(files, outputDir);
          break;
        case "node-parallel":
          result = await runNode(files, outputDir, true);
          break;
        case "node-sequential":
          result = await runNode(files, outputDir, false);
          break;
        default:
          throw new Error(`Unknown mode: ${mode}`);
      }

      modeResult.result = result;
      modeResult.status = "completed";
    }
  } catch (error) {
    console.error(error);
    alert(`处理失败: ${error}`);
  } finally {
    processing.value = false;
  }
}

function getBestMode(): ModeResult | null {
  const completed = modeResults.value.filter((m) => m.status === "completed" && m.result);
  if (completed.length === 0) return null;

  return completed.reduce((best, current) => {
    if (!best.result || !current.result) return best;
    return current.result.total_time_ms < best.result.total_time_ms ? current : best;
  });
}

function getSpeedup(baseMs: number, compareMs: number): string {
  if (baseMs === 0 || compareMs === 0) return "-";
  return (compareMs / baseMs).toFixed(1) + "x";
}
</script>

<template>
  <div class="batch-page">
    <!-- 顶部工具栏 -->
    <header class="batch-toolbar">
      <button @click="selectFolder" class="btn btn-primary">
        📁 选择文件夹
      </button>
      <span class="folder-path">{{ folderPath || "未选择文件夹" }}</span>
      <span v-if="loading" class="loading">⏳ 加载中...</span>
      <span v-else-if="images.length" class="count">
        共 {{ images.length }} 张图片
      </span>
    </header>

    <!-- 主区域 -->
    <div class="batch-body">
      <!-- 左侧：图片列表 -->
      <section class="image-list-panel">
        <div class="panel-header">
          <label class="checkbox-label">
            <input
              type="checkbox"
              :checked="selectAll && images.length > 0"
              :indeterminate="
                selectedFiles.size > 0 && selectedFiles.size < images.length
              "
              @change="toggleSelectAll"
            />
            全选
          </label>
          <span class="selected-count">
            已选 {{ selectedFiles.size }} / {{ images.length }}
          </span>
        </div>

        <div v-if="images.length === 0 && !loading" class="empty-hint">
          <p>👆 点击上方按钮选择一个图片文件夹</p>
        </div>

        <div class="image-grid">
          <div
            v-for="img in images"
            :key="img.path"
            class="image-card"
            :class="{ selected: selectedFiles.has(img.path) }"
            @click="toggleFile(img.path)"
          >
            <div class="card-check">
              <input
                type="checkbox"
                :checked="selectedFiles.has(img.path)"
                @click.stop="toggleFile(img.path)"
              />
            </div>
            <div class="card-icon">🖼️</div>
            <div class="card-name">{{ img.name }}</div>
            <div class="card-meta">
              {{ img.width }}×{{ img.height }} · {{ formatSize(img.size_bytes) }}
            </div>
          </div>
        </div>
      </section>

      <!-- 右侧：处理面板 -->
      <section class="process-panel">
        <h3 class="panel-title">⚙️ 处理选项</h3>

        <div class="option-group">
          <label class="option-label">处理模式</label>
          <select v-model="processingMode" class="option-select">
            <option value="rust-parallel">🚀 Rust 并行 (Rayon)</option>
            <option value="rust-sequential">🐢 Rust 串行</option>
            <option value="node-parallel">⚡ Node.js 并行</option>
            <option value="node-sequential">🐌 Node.js 串行</option>
            <option value="compare-all">🔄 对比全部模式</option>
          </select>
        </div>

        <div class="option-group">
          <label class="option-label">目标宽度 (px)</label>
          <input v-model.number="targetWidth" type="number" class="option-input" min="1" />
        </div>

        <div class="option-group">
          <label class="option-label">目标高度 (px)</label>
          <input v-model.number="targetHeight" type="number" class="option-input" min="1" />
        </div>

        <div class="option-group">
          <label class="option-label checkbox-label-inline">
            <input v-model="keepRatio" type="checkbox" />
            保持宽高比
          </label>
        </div>

        <div class="option-group">
          <label class="option-label">输出格式</label>
          <select v-model="format" class="option-select">
            <option value="jpg">JPEG</option>
            <option value="png">PNG</option>
            <option value="webp">WebP</option>
          </select>
        </div>

        <div class="option-group">
          <label class="option-label">
            质量: {{ quality }}%
          </label>
          <input v-model.number="quality" type="range" min="1" max="100" class="option-range" />
        </div>

        <button
          @click="startProcessing"
          class="btn btn-process"
          :disabled="processing || selectedFiles.size === 0"
        >
          {{ processing ? "⏳ 处理中..." : "🚀 开始批处理" }}
        </button>

        <div v-if="modeResults.some(m => m.status === 'completed' || m.status === 'running')" class="result-panel">
          <h4 class="result-title">📊 处理结果对比</h4>
          
          <div v-if="processing" class="running-indicator">
            正在处理，请稍候...
          </div>

          <div v-else class="comparison-grid">
            <div
              v-for="item in modeResults"
              :key="item.mode"
              class="comparison-card"
              :class="{
                'card-running': item.status === 'running',
                'card-completed': item.status === 'completed',
                'card-winner': getBestMode()?.mode === item.mode,
              }"
            >
              <div class="card-header">
                <span class="card-label">{{ item.label }}</span>
                <span v-if="getBestMode()?.mode === item.mode" class="winner-badge">🏆 最快</span>
              </div>

              <div v-if="item.status === 'running'" class="card-status running">
                ⏳ 处理中...
              </div>

              <div v-else-if="item.status === 'completed' && item.result" class="card-content">
                <div class="stat-row">
                  <span class="stat-label">耗时</span>
                  <span class="stat-value time">{{ item.result.total_time_ms }} ms</span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">成功</span>
                  <span class="stat-value success">{{ item.result.success }}</span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">失败</span>
                  <span class="stat-value" :class="{ failed: item.result.failed > 0 }">
                    {{ item.result.failed }}
                  </span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">平均</span>
                  <span class="stat-value">{{ item.result.total > 0 ? (item.result.total_time_ms / item.result.total).toFixed(1) : 0 }} ms</span>
                </div>
                <div v-if="getBestMode() && getBestMode() !== item" class="stat-row">
                  <span class="stat-label">慢于最快</span>
                  <span class="stat-value slow">{{ getSpeedup(getBestMode()!.result!.total_time_ms, item.result.total_time_ms) }}</span>
                </div>
              </div>

              <div v-else class="card-status pending">
                等待运行
              </div>
            </div>
          </div>

          <div v-if="getBestMode()" class="result-summary">
            <p>🏆 <strong>{{ getBestMode()!.label }}</strong> 表现最佳，耗时 <strong>{{ getBestMode()!.result!.total_time_ms }} ms</strong></p>
          </div>

          <div class="result-note">
            <p>💡 <strong>性能对比说明：</strong></p>
            <ul>
              <li>Rust 并行：使用 Rayon 自动分配多核心并行处理</li>
              <li>Rust 串行：使用标准迭代器，单线程顺序处理</li>
              <li>Node.js 并行：使用 Promise.all 并发处理</li>
              <li>Node.js 串行：使用 for-await 顺序处理</li>
            </ul>
            <p>图片越多，并行处理的优势越明显！</p>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.batch-page {
  display: flex;
  flex-direction: column;
  height: calc(100vh - 40px);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  background: #f5f6f8;
}

/* 顶部工具栏 */
.batch-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  background: #fff;
  border-bottom: 1px solid #e0e0e0;
  height: 48px;
}
.folder-path {
  flex: 1;
  font-size: 13px;
  color: #666;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.loading { color: #f7971e; font-size: 13px; }
.count { color: #56ccf2; font-size: 13px; font-weight: bold; }

/* 主区域双栏 */
.batch-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* 左侧：图片列表 */
.image-list-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  border-right: 1px solid #e0e0e0;
  background: #fff;
}
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid #eee;
  font-size: 13px;
}
.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  user-select: none;
}
.selected-count { color: #666; }
.empty-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: #999;
  font-size: 14px;
}
.image-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 8px;
  padding: 12px;
  overflow-y: auto;
  flex: 1;
  align-content: start;
}
.image-card {
  position: relative;
  padding: 10px;
  border: 2px solid #eee;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  text-align: center;
}
.image-card:hover {
  border-color: #56ccf2;
  background: #f0f9ff;
}
.image-card.selected {
  border-color: #2f80ed;
  background: #e8f4fd;
}
.card-check {
  position: absolute;
  top: 6px;
  left: 6px;
}
.card-icon { font-size: 28px; margin: 4px 0; }
.card-name {
  font-size: 11px;
  color: #333;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin: 4px 0;
}
.card-meta {
  font-size: 10px;
  color: #888;
}

/* 右侧：处理面板 */
.process-panel {
  width: 400px;
  padding: 20px;
  overflow-y: auto;
  background: #fafafa;
}
.panel-title {
  font-size: 16px;
  margin-bottom: 20px;
  color: #333;
}
.option-group {
  margin-bottom: 16px;
}
.option-label {
  display: block;
  font-size: 13px;
  color: #555;
  margin-bottom: 4px;
}
.checkbox-label-inline {
  display: flex !important;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}
.option-input {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  font-size: 14px;
  outline: none;
}
.option-input:focus {
  border-color: #56ccf2;
  box-shadow: 0 0 0 2px rgba(86, 204, 242, 0.15);
}
.option-select {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  background: #fff;
}
.option-range {
  width: 100%;
  margin-top: 4px;
}
.btn-process {
  width: 100%;
  padding: 12px;
  font-size: 15px;
  border: none;
  border-radius: 8px;
  background: linear-gradient(135deg, #56ccf2, #2f80ed);
  color: white;
  cursor: pointer;
  font-weight: bold;
  transition: all 0.2s;
  margin-top: 8px;
}
.btn-process:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(47, 128, 237, 0.3);
}
.btn-process:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.btn {
  padding: 6px 14px;
  font-size: 13px;
  border: 1px solid #ced4da;
  background-color: #fff;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}
.btn:hover { background-color: #f1f3f5; }
.btn-primary {
  background-color: #2f80ed;
  color: white;
  border-color: #2f80ed;
}
.btn-primary:hover { background-color: #266dd3; }

/* 结果面板 */
.result-panel {
  margin-top: 24px;
  padding: 16px;
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 10px;
}
.result-title {
  font-size: 14px;
  margin-bottom: 12px;
  color: #333;
}
.running-indicator {
  text-align: center;
  padding: 20px;
  color: #f7971e;
  font-size: 14px;
}
.comparison-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.comparison-card {
  padding: 12px;
  background: #f8f9fa;
  border-radius: 8px;
  border: 2px solid transparent;
  transition: all 0.2s;
}
.comparison-card.card-winner {
  border-color: #27ae60;
  background: #e8f5e9;
}
.comparison-card.card-running {
  opacity: 0.7;
}
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.card-label {
  font-size: 12px;
  font-weight: bold;
  color: #333;
}
.winner-badge {
  font-size: 10px;
  background: #27ae60;
  color: white;
  padding: 2px 6px;
  border-radius: 4px;
}
.card-status {
  font-size: 12px;
  text-align: center;
  padding: 8px;
  border-radius: 4px;
}
.card-status.running {
  color: #f7971e;
  background: #fff8e1;
}
.card-status.pending {
  color: #999;
  background: #f0f0f0;
}
.stat-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
  font-size: 12px;
}
.stat-label {
  color: #888;
}
.stat-value {
  font-weight: bold;
  color: #333;
}
.stat-value.time { color: #f7971e; }
.stat-value.success { color: #27ae60; }
.stat-value.failed { color: #e74c3c; }
.stat-value.slow { color: #e74c3c; }
.result-summary {
  margin-top: 16px;
  padding: 12px;
  background: #e8f5e9;
  border-radius: 6px;
  font-size: 13px;
  color: #2e7d32;
  text-align: center;
}
.result-note {
  margin-top: 12px;
  padding: 12px;
  background: #fff8e1;
  border-radius: 6px;
  font-size: 12px;
  color: #856404;
  line-height: 1.5;
}
.result-note ul {
  margin: 4px 0;
  padding-left: 18px;
}
.result-note li {
  margin-bottom: 4px;
}
</style>
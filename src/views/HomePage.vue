<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { marked } from "marked";

const filePath = ref("");
const markdownContent = ref(`# 我的第一篇 Tauri 笔记

在这里输入 Markdown，右侧会**实时预览**效果！

### 支持的特性：
- 列表 1
- 列表 2

\`\`\`rust
fn main() {
    println!("Hello, Tauri!");
}
\`\`\``);

const previewHtml = computed(() => {
  return marked.parse(markdownContent.value) as string;
});

async function handleSave() {
  try {
    if (!filePath.value) {
      const selected = await save({
        filters: [{ name: "Markdown", extensions: ["md"] }],
      });
      if (!selected) return;
      filePath.value = selected;
    }
    await invoke("save_markdown_file", {
      path: filePath.value,
      content: markdownContent.value,
    });
  } catch (error) {
    alert(`保存失败: ${error}`);
  }
}

async function handleOpen() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (selected) {
      filePath.value = selected as string;
      const res = await invoke<string>("read_markdown_file", {
        path: selected,
      });
      markdownContent.value = res;
    }
  } catch (error) {
    alert(`打开失败: ${error}`);
  }
}
</script>

<template>
  <div class="app-window">
    <header class="toolbar">
      <div class="file-info">📂 {{ filePath || "未保存的新文件" }}</div>
      <div class="actions">
        <button @click="handleOpen" class="btn">📖 打开</button>
        <button @click="handleSave" class="btn btn-primary">💾 保存</button>
      </div>
    </header>

    <main class="workspace">
      <section class="pane editor-pane">
        <textarea
          v-model="markdownContent"
          placeholder="开始你的写作..."
          class="markdown-input"
        ></textarea>
      </section>
      <section class="pane preview-pane">
        <div class="markdown-body" v-html="previewHtml"></div>
      </section>
    </main>
  </div>
</template>

<style scoped>
.app-window {
  display: flex;
  flex-direction: column;
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  background-color: #f8f9fa;
  color: #212529;
}
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 20px;
  background-color: #ffffff;
  border-bottom: 1px solid #e9ecef;
  height: 50px;
  box-sizing: border-box;
}
.file-info {
  font-size: 14px;
  color: #495057;
  max-width: 60%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.btn {
  padding: 6px 14px;
  font-size: 13px;
  border: 1px solid #ced4da;
  background-color: #fff;
  border-radius: 4px;
  cursor: pointer;
  margin-left: 8px;
  transition: all 0.2s;
}
.btn:hover { background-color: #f1f3f5; }
.btn-primary {
  background-color: #2f80ed;
  color: white;
  border-color: #2f80ed;
}
.btn-primary:hover { background-color: #266dd3; }
.workspace {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.pane {
  flex: 1;
  height: 100%;
  overflow-y: auto;
  box-sizing: border-box;
}
.editor-pane {
  border-right: 1px solid #e9ecef;
  background-color: #ffffff;
}
.markdown-input {
  width: 100%;
  height: 100%;
  border: none;
  resize: none;
  padding: 20px;
  box-sizing: border-box;
  font-family: "Fira Code", Consolas, Monaco, monospace;
  font-size: 14px;
  line-height: 1.6;
  color: #333;
  outline: none;
  background-color: #fcfcfc;
}
.preview-pane {
  padding: 20px 30px;
  background-color: #ffffff;
}
.markdown-body {
  font-size: 15px;
  line-height: 1.6;
  color: #24292e;
}
.markdown-body :deep(h1),
.markdown-body :deep(h2) {
  border-bottom: 1px solid #eaecef;
  padding-bottom: 0.3em;
  margin-top: 24px;
  margin-bottom: 16px;
}
.markdown-body :deep(h1) { font-size: 2em; }
.markdown-body :deep(h2) { font-size: 1.5em; }
.markdown-body :deep(h3) { font-size: 1.25em; margin-top: 24px; }
.markdown-body :deep(p) { margin-top: 0; margin-bottom: 16px; }
.markdown-body :deep(ul),
.markdown-body :deep(ol) { padding-left: 2em; margin-bottom: 16px; }
.markdown-body :deep(code) {
  padding: 0.2em 0.4em;
  background-color: rgba(27, 31, 35, 0.05);
  border-radius: 3px;
  font-family: monospace;
  font-size: 85%;
}
.markdown-body :deep(pre) {
  padding: 16px;
  background-color: #f6f8fa;
  border-radius: 6px;
  overflow: auto;
}
.markdown-body :deep(pre code) {
  background-color: transparent;
  padding: 0;
  font-size: 100%;
}
</style>

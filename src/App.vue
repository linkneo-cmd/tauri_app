<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from '@tauri-apps/plugin-dialog'; // 引入对话框 API
import { marked } from 'marked'; // 引入 markdown 解析器

const filePath = ref(''); // 路径初始化为空
const markdownContent = ref('# 本地图片批处理工具（体验 Rust 多线程高并发威力）\n# 我的第一篇 Tauri 笔记\n\n在这里输入 Markdown，右侧会**实时预览**效果！\n\n### 支持的特性：\n- 列表 1\n- 列表 2\n\n```rust\nfn main() {\n    println!("Hello, Tauri!");\n}\n```');

const previewHtml = computed(() => {
  return marked.parse(markdownContent.value);
});

async function handleSave() {
  try {
    if(!filePath.value){
      const selected = await save({
        filters: [{ name: 'Markdown', extensions: ['md'] }]
      });
      if (!selected) return; // 用户取消了保存
      filePath.value = selected;
    }
    await invoke('save_markdown_file', { 
      path: filePath.value, 
      content: markdownContent.value 
    });
    alert('保存成功！');
  } catch (error) {
    alert(`保存失败: ${error}`);
  }
}

// 1. 打开文件
async function handleOpen() {
  try {
    // 弹出原生选择文件窗口，并限制只能选 md 文件
    const selected = await open({
      multiple: false, // 只能选单个文件
      filters: [{ name: 'Markdown', extensions: ['md'] }]
    });

    if (selected) {
      filePath.value = selected; // 保存用户选择的路径
      // 调用 Rust 读取文件内容
      const res = await invoke<string>('read_markdown_file', { path: selected });
      markdownContent.value = res;
    }
  } catch (error) {
    alert(`打开失败: ${error}`);
  }
}

</script>

<template>
  <!-- <main class="container">
    <div class="status-bar">
      当前文件: {{ filePath || '未保存的新文件' }}
    </div>
    <textarea v-model="markdownContent"></textarea>
    <div class="actions">
      <button @click="handleSave">保存到本地</button>
      <button @click="handleOpen">从本地读取</button>
    </div>
  </main> -->
  <div class="app-window">
    <header class="toolbar">
      <div class="file-info">📂 {{ filePath || '未保存的新文件' }}</div>
      <div class="actions">
        <button @click="handleOpen" class="btn">打开</button>
        <button @click="handleSave" class="btn btn-primary">保存</button>
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
/* 全局基础样式重置 */
.app-window {
  display: flex;
  flex-direction: column;
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  background-color: #f8f9fa;
  color: #212529;
}

/* 顶部工具栏样式 */
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

/* 工作区双栏布局 */
.workspace {
  display: flex;
  flex: 1;
  overflow: hidden; /* 防止主窗口出现滚动条 */
}

/* 通用分栏样式 */
.pane {
  flex: 1;
  height: 100%;
  overflow-y: auto; /* 让两栏独立滚动 */
  box-sizing: border-box;
}

/* 左侧编辑器 */
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

/* 右侧预览区及精美 Markdown 样式（仿 GitHub） */
.preview-pane {
  padding: 20px 30px;
  background-color: #ffffff;
}
.markdown-body {
  font-size: 15px;
  line-height: 1.6;
  color: #24292e;
}
.markdown-body :deep(h1), .markdown-body :deep(h2) {
  border-bottom: 1px solid #eaecef;
  padding-bottom: 0.3em;
  margin-top: 24px;
  margin-bottom: 16px;
}
.markdown-body :deep(h1) { font-size: 2em; }
.markdown-body :deep(h2) { font-size: 1.5em; }
.markdown-body :deep(h3) { font-size: 1.25em; margin-top: 24px; }
.markdown-body :deep(p) { margin-top: 0; margin-bottom: 16px; }
.markdown-body :deep(ul), .markdown-body :deep(ol) { padding-left: 2em; margin-bottom: 16px; }
.markdown-body :deep(code) {
  padding: 0.2em 0.4em;
  background-color: rgba(27,31,35,0.05);
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
<style>
</style>
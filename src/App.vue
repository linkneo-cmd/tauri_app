<script setup lang="ts">
</script>

<template>
  <div id="app-shell">
    <nav class="nav-bar">
      <span class="nav-title">🦀 Tauri 学习项目</span>
      <div class="nav-links">
        <router-link to="/" class="nav-link" active-class="nav-link--active">
          📝 Markdown 编辑器
        </router-link>
        <router-link to="/image-batch" class="nav-link" active-class="nav-link--active">
          🖼️ 图片批处理
        </router-link>
      </div>
    </nav>
    <router-view />
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
html, body, #app, #app-shell {
  height: 100%;
  overflow: hidden;
}
.nav-bar {
  display: flex;
  align-items: center;
  height: 40px;
  padding: 0 16px;
  background: #1a1a2e;
  color: #e0e0e0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  font-size: 13px;
  -webkit-app-region: drag;
  user-select: none;
}
.nav-title {
  font-weight: bold;
  margin-right: 24px;
  color: #56ccf2;
}
.nav-links {
  display: flex;
  gap: 4px;
  -webkit-app-region: no-drag;
}
.nav-link {
  text-decoration: none;
  color: #aaa;
  padding: 4px 12px;
  border-radius: 4px;
  transition: all 0.2s;
}
.nav-link:hover {
  color: #fff;
  background: rgba(255,255,255,0.08);
}
.nav-link--active {
  color: #56ccf2;
  background: rgba(86,204,242,0.12);
}
</style>

<template>
  <main class="unsupported-page">
    <section class="unsupported-card" role="status">
      <div class="unsupported-icon" aria-hidden="true">
        <Icon name="cloud-off" :size="32" />
      </div>
      <h1>远程服务暂不可用</h1>
      <p>
        当前仓库中的同步服务器实现不支持新版数据库的复合记录标识与多对多关联，
        因此多设备同步和错题社区已停用。
      </p>
      <p class="local-note">
        本地错题管理、复习、导入导出及数据清理不受影响。
      </p>
    </section>
  </main>
</template>

<script setup lang="ts">
import Icon from '../components/Icon.vue'
</script>

<style scoped>
.unsupported-page {
  min-height: calc(100vh - 96px);
  display: grid;
  place-items: center;
  padding: 24px;
}

.unsupported-card {
  width: min(560px, 100%);
  padding: 32px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 18px;
  background: var(--card-bg, #fff);
  color: var(--text-primary, #1f2937);
  text-align: center;
  box-shadow: 0 12px 32px rgb(15 23 42 / 8%);
}

.unsupported-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 18px;
  display: grid;
  place-items: center;
  border-radius: 50%;
  background: rgb(245 158 11 / 14%);
  color: #d97706;
}

h1 {
  margin: 0 0 14px;
  font-size: 1.4rem;
}

p {
  margin: 0;
  line-height: 1.7;
  color: var(--text-secondary, #64748b);
}

.local-note {
  margin-top: 12px;
  font-size: 0.9rem;
}
</style>

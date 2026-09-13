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
      <p class="local-note">本地错题管理、复习、导入导出及数据清理不受影响。</p>
    </section>
  </main>
</template>

<script setup lang="ts">
import Icon from '../components/Icon.vue'
</script>

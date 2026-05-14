<template>
  <div class="sync-button-wrapper">
    <!-- 主按钮 -->
    <button
      class="sync-btn"
      :class="{ 'sync-btn--syncing': isSyncing }"
      @click="handleStartSync"
      :disabled="isSyncing"
    >
      <span v-if="!isSyncing" class="btn-text">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
          <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
          <path d="M16 16h5v5" />
        </svg>
        {{ buttonText }}
      </span>
      <span v-else class="sync-indicator">
        <svg class="sync-spinner" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" opacity="0.3" />
          <circle cx="12" cy="12" r="10" cx="12" cy="12" r="8" stroke="currentColor" stroke-width="3" fill="none" stroke-dasharray="50" stroke-dashoffset="-12" />
        </svg>
        同步中...
      </span>
    </button>

    <!-- 同步进度覆盖层 -->
    <SyncOverlay
      v-model="showProgress"
      :total="handshakeResult?.pull_list.length + handshakeResult?.push_list.length || 0"
      :current="syncProgress.current"
      :pulled="syncProgress.details.pulled"
      :pushed="syncProgress.details.pushed"
      :conflicts-resolved="syncProgress.details.conflicts_resolved"
      :failed="syncProgress.details.failed"
      :phase="syncProgress.phase"
      :show-cancel-button="true"
    />

    <!-- 冲突解决覆盖层 -->
    <ConflictResolver
      v-model="showConflictResolver"
      :conflicts="conflictsToResolve"
      @resolve="handleConflictResolution"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { startSync, handshake, type SyncHandshakeResult, type ConflictInfo, type ResolvedConflict } from '../apis/sync';
import SyncOverlay from './SyncOverlay.vue';
import ConflictResolver from './ConflictResolver.vue';

const isSyncing = ref(false);
const syncProgress = ref({
  phase: '' as string,
  current: 0,
  total: 0,
  details: {
    pulled: 0,
    pushed: 0,
    conflicts_resolved: 0,
    failed: 0,
  },
});
const handshakeResult = ref<SyncHandshakeResult | null>(null);
const showProgress = ref(false);
const showConflictResolver = ref(false);
const conflictsToResolve = ref<ConflictInfo[]>([]);

const buttonText = computed(() => {
  if (isSyncing.value) return '同步中...';
  return '同步数据';
});

const handleStartSync = async () => {
  try {
    isSyncing.value = true;
    showProgress.value = true;

    // 获取本地 pending 记录和服务端数据
    const localPending = await invoke('get_local_pending_records');
    const authKey = localStorage.getItem('auth_key') || '';
    const remoteRecords = await import('../apis/sync').then(m => m.getAllSyncData(authKey));

    // 执行握手
    const result = handshake(localPending, remoteRecords);
    handshakeResult.value = result;

    // 如果有冲突，先弹出冲突解决器
    if (result.conflicts.length > 0) {
      showConflictResolver.value = true;
      conflictsToResolve.value = result.conflicts;
      return; // 等待用户解决冲突后再继续
    }

    // 直接开始同步流程
    await startSync({
      onProgress: (progress) => {
        syncProgress.value = progress;
      },
      onConflict: async (conflicts) => {
        // 如果程序检测到冲突（比如解决后有新的），也弹出解决器
        showConflictResolver.value = true;
        conflictsToResolve.value = conflicts;
        return []; // 空数组表示稍后会处理
      },
    });

    // 同步完成
    showToast('同步完成', 'success');

  } catch (error) {
    console.error('Sync failed:', error);
    showToast(`同步失败：${error}`, 'error');
  } finally {
    isSyncing.value = false;
    showProgress.value = false;
  }
};

const handleConflictResolution = async (resolutions: ResolvedConflict[]) => {
  showConflictResolver.value = false;

  // 应用冲突解决结果到本地数据库
  for (const resolution of resolutions) {
    await invoke('apply_conflict_resolution', {
      resolution,
    });
  }

  // 重新计算推送/拉取列表
  const localPending = await invoke('get_local_pending_records');
  const authKey = localStorage.getItem('auth_key') || '';
  const remoteRecords = await import('../apis/sync').then(m => m.getAllSyncData(authKey));

  const newResult = handshake(localPending, remoteRecords);
  handshakeResult.value = newResult;

  // 继续同步流程
  if (newResult.push_list.length > 0 || newResult.pull_list.length > 0) {
    showProgress.value = true;

    await startSync({
      onProgress: (progress) => {
        syncProgress.value = progress;
      },
    });

    showToast('同步完成', 'success');
  } else {
    showToast('所有冲突已解决，无需同步', 'info');
  }
};

const showToast = (message: string, type: 'success' | 'error' | 'info') => {
  // TODO: 使用项目现有的 Toast 通知系统
  console.log(`[${type.toUpperCase()}] ${message}`);
};
</script>

<style scoped>
.sync-button-wrapper {
  display: inline-block;
}

.sync-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1.25rem;
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: #ffffff;
  border: none;
  border-radius: 12px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(16, 185, 129, 0.3);
}

.sync-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.4);
}

.sync-btn:active:not(:disabled) {
  transform: translateY(0);
}

.sync-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.btn-text {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.btn-text svg {
  width: 18px;
  height: 18px;
}

.sync-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.sync-spinner {
  width: 18px;
  height: 18px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>

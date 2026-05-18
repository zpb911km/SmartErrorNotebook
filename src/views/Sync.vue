<template>
  <div class="sync-page">
    <!-- 配置区 -->
    <section class="sync-section">
      <h2 class="section-title">同步服务器</h2>
      <div class="info-card">
        <div class="info-row">
          <span class="info-label">服务器地址</span>
          <div class="info-value">
            <span class="status-dot" :class="serverStatusClass"></span>
            <code class="mono">{{ serverUrl }}</code>
            <button class="btn-text" @click="showUrlInput = true">修改</button>
          </div>
        </div>
        <div v-if="showUrlInput" class="auth-input-row">
          <input
            v-model="urlInput"
            type="text"
            placeholder="http://localhost:5000"
            class="input"
          />
          <button class="btn btn-primary" @click="saveServerUrl">保存</button>
          <button class="btn btn-secondary" @click="showUrlInput = false">
            取消
          </button>
        </div>
        <div class="info-row">
          <span class="info-label">授权码</span>
          <div class="info-value">
            <span class="status-dot" :class="authStatusClass"></span>
            <template v-if="authKey">
              <code class="auth-key">{{ maskedAuthKey }}</code>
              <button class="btn-text" @click="showAuthInput = true">
                更换
              </button>
            </template>
            <template v-else>
              <span class="text-warning">未配置</span>
              <button class="btn-text" @click="showAuthInput = true">
                去设置
              </button>
            </template>
          </div>
        </div>
        <div v-if="showAuthInput" class="auth-input-row">
          <input
            v-model="authInput"
            type="text"
            placeholder="输入授权码..."
            class="input"
          />
          <button class="btn btn-primary" @click="saveAuthKey">保存</button>
          <button class="btn btn-secondary" @click="showAuthInput = false">
            取消
          </button>
        </div>
        <div class="info-row info-row--action">
          <button
            class="btn btn-secondary btn--sm"
            @click="checkConnection"
            :disabled="checking"
          >
            {{ checking ? '检测中...' : '检测连接' }}
          </button>
          <span v-if="checkMessage" class="check-msg" :class="checkMsgClass">{{
            checkMessage
          }}</span>
        </div>
      </div>
    </section>

    <!-- 同步状态区 -->
    <section class="sync-section">
      <h2 class="section-title">数据同步</h2>
      <div class="info-card">
        <div class="info-row">
          <span class="info-label">待上传</span>
          <span class="info-value">
            <span class="badge" :class="statsUploadClass">
              {{ syncStats.toUpload }}
            </span>
          </span>
        </div>
        <div class="info-row">
          <span class="info-label">待下载</span>
          <span class="info-value">
            <span class="badge" :class="statsDownloadClass">
              {{
                syncStats.toDownload !== null ? syncStats.toDownload : '未知'
              }}
            </span>
          </span>
        </div>
        <div class="info-row">
          <span class="info-label">待处理（冲突）</span>
          <span class="info-value">
            <span class="badge" :class="statsConflictClass">
              {{
                syncStats.toConflicts !== null ? syncStats.toConflicts : '未知'
              }}
            </span>
          </span>
        </div>
      </div>

      <button class="sync-btn" :disabled="syncing" @click="handleSync">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          width="20"
          height="20"
        >
          <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
          <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
          <path d="M16 16h5v5" />
        </svg>
        {{ syncing ? '同步中...' : '开始同步' }}
      </button>
    </section>

    <!-- 使用说明 -->
    <section class="sync-section">
      <h2 class="section-title">使用说明</h2>
      <div class="help-text">
        <p>1. 在服务端管理页面生成授权码</p>
        <p>2. 在此页面输入授权码并保存</p>
        <p>3. 点击"开始同步"按钮同步数据</p>
        <p>4. 多台设备使用相同授权码即可互相同步</p>
      </div>
    </section>

    <!-- 同步进度覆盖层 -->
    <SyncOverlay
      v-model="showProgress"
      :total="syncTotal"
      :current="syncProgress.current"
      :pulled="syncProgress.details.pulled"
      :pushed="syncProgress.details.pushed"
      :conflicts-resolved="syncProgress.details.conflicts_resolved"
      :failed="syncProgress.details.failed"
      :phase="syncProgress.phase"
      :show-cancel-button="false"
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
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  handshake,
  getAllSyncData,
  uploadRecord,
  applyPullRecords,
  getAllLocalRecords,
  downloadRecord
} from '../apis/sync'
import type {
  SyncHandshakeResult,
  ConflictInfo,
  ResolvedConflict,
  ServerRecord
} from '../apis/sync'
import SyncOverlay from '../components/SyncOverlay.vue'
import ConflictResolver from '../components/ConflictResolver.vue'
import { showError, showSuccess } from '../utils/notification'

const AUTH_STORAGE_KEY = 'auth_key'
const URL_STORAGE_KEY = 'sync_server_url'

// ---------- 服务器地址 ----------
const serverUrl = ref(
  localStorage.getItem(URL_STORAGE_KEY) || 'http://localhost:5000'
)
const showUrlInput = ref(false)
const urlInput = ref('')

const saveServerUrl = () => {
  const val = urlInput.value.trim()
  if (val) {
    localStorage.setItem(URL_STORAGE_KEY, val)
    serverUrl.value = val
  }
  showUrlInput.value = false
  urlInput.value = ''
  checkConnection()
}

// ---------- 授权码 ----------
const authKey = ref(localStorage.getItem(AUTH_STORAGE_KEY) || '')
const showAuthInput = ref(false)
const authInput = ref('')

const maskedAuthKey = computed(() => {
  if (!authKey.value) return ''
  const k = authKey.value
  return k.length > 8 ? `${k.slice(0, 4)}****${k.slice(-4)}` : k
})

const saveAuthKey = () => {
  const val = authInput.value.trim()
  if (val) {
    localStorage.setItem(AUTH_STORAGE_KEY, val)
    authKey.value = val
  }
  showAuthInput.value = false
  authInput.value = ''
  checkConnection()
}

// ---------- 连接检测 ----------
const serverOnline = ref<boolean | null>(null)
const authValid = ref<boolean | null>(null)
const checking = ref(false)
const checkMessage = ref('')
const checkOk = ref(true)

const serverStatusClass = computed(() => {
  if (serverOnline.value === null) return ''
  return serverOnline.value ? 'dot-online' : 'dot-offline'
})

const authStatusClass = computed(() => {
  if (authValid.value === null) return ''
  return authValid.value ? 'dot-online' : 'dot-offline'
})

const checkMsgClass = computed(() => (checkOk.value ? 'msg-ok' : 'msg-err'))

const checkConnection = async () => {
  checking.value = true
  checkMessage.value = ''

  const { checkServerHealth, validateAuthKey } = await import('../apis/sync')

  // 检测服务器
  const online = await checkServerHealth()
  serverOnline.value = online

  if (!online) {
    checkOk.value = false
    checkMessage.value = '无法连接到服务器'
    checking.value = false
    return
  }

  // 检测授权码
  if (authKey.value) {
    const valid = await validateAuthKey(authKey.value)
    authValid.value = valid
    if (valid) {
      checkOk.value = true
      checkMessage.value = '服务器在线，授权码有效'
    } else {
      checkOk.value = false
      checkMessage.value = '服务器在线，但授权码无效'
    }
  } else {
    authValid.value = null
    checkOk.value = true
    checkMessage.value = '服务器在线（未配置授权码）'
  }

  checking.value = false
}

// ---------- 同步状态 ----------
const syncing = ref(false)
const showProgress = ref(false)
const showConflictResolver = ref(false)
const conflictsToResolve = ref<ConflictInfo[]>([])
const handshakeResult = ref<SyncHandshakeResult | null>(null)
const syncProgress = ref({
  phase: 'handshake' as string,
  current: 0,
  total: 0,
  details: { pulled: 0, pushed: 0, conflicts_resolved: 0, failed: 0 }
})

const syncTotal = computed(() => {
  const r = handshakeResult.value
  return r ? r.push_list.length + r.pull_list.length : 0
})

// ---------- 握手统计（待上传/待下载/待处理） ----------
const syncStats = ref<{
  toUpload: number
  toDownload: number | null
  toConflicts: number | null
}>({ toUpload: 0, toDownload: null, toConflicts: null })

const statsUploadClass = computed(() =>
  syncStats.value.toUpload > 0 ? 'badge-warning' : 'badge-success'
)
const statsDownloadClass = computed(() => {
  if (syncStats.value.toDownload === null) return 'badge-neutral'
  return syncStats.value.toDownload > 0 ? 'badge-warning' : 'badge-success'
})
const statsConflictClass = computed(() => {
  if (syncStats.value.toConflicts === null) return 'badge-neutral'
  return syncStats.value.toConflicts > 0 ? 'badge-danger' : 'badge-success'
})

/** 尝试握手并刷新三栏统计 */
const refreshSyncStats = async () => {
  try {
    const localPending = await invoke<ServerRecord[]>('get_all_pending_records')
    syncStats.value = {
      toUpload: localPending.length,
      toDownload: null,
      toConflicts: null
    }

    const key = localStorage.getItem(AUTH_STORAGE_KEY)
    if (!key) return // 无密钥，其余两项显示未知

    const { checkServerHealth, getAllSyncData } = await import('../apis/sync')
    const online = await checkServerHealth()
    if (!online) return // 服务不可达，其余两项显示未知

    // 服务在线且有密钥 → 使用全量记录握手
    const [allLocal, remoteRecords] = await Promise.all([
      getAllLocalRecords(),
      getAllSyncData(key)
    ])
    console.log('all local records:', allLocal)
    console.log('remote records:', remoteRecords)
    const result = handshake(allLocal, remoteRecords)
    console.log('handshake result:', result)
    syncStats.value = {
      toUpload: result.push_list.length,
      toDownload: result.pull_list.length,
      toConflicts: result.conflicts.length
    }
  } catch (e) {
    console.warn('Failed to refresh sync stats:', e)
  }
}

onMounted(() => {
  checkConnection()
  refreshSyncStats()
})

// ---------- 同步流程 ----------
const handleSync = async () => {
  if (!authKey.value) {
    showError('请先配置授权码', 'error')
    return
  }

  syncing.value = true
  showProgress.value = true
  syncProgress.value = {
    phase: 'handshake',
    current: 0,
    total: 0,
    details: { pulled: 0, pushed: 0, conflicts_resolved: 0, failed: 0 }
  }

  try {
    const [allLocal, remoteRecords] = await Promise.all([
      getAllLocalRecords(),
      getAllSyncData(authKey.value)
    ])

    const result = handshake(allLocal, remoteRecords)
    console.log('handshake result:', result)
    handshakeResult.value = result

    if (result.conflicts.length > 0) {
      syncProgress.value.phase = 'resolving_conflicts'
      conflictsToResolve.value = result.conflicts
      showConflictResolver.value = true
      return
    }

    await doSync(result, authKey.value)

    showSuccess('同步完成', 'success')
    refreshSyncStats()
  } catch (e) {
    console.error('Sync failed:', e)
    showError(`同步失败：${e}`, 'error')
  } finally {
    if (!showConflictResolver.value) {
      syncing.value = false
      showProgress.value = false
    }
  }
}

const doSync = async (result: SyncHandshakeResult, key: string) => {
  const total = result.push_list.length + result.pull_list.length
  syncProgress.value.total = total
  let current = 0

  // 逐个上传，及时更新进度
  for (const recordId of result.push_list) {
    syncProgress.value.phase = 'pushing'
    try {
      const record = await invoke<ServerRecord>('get_record_for_upload', {
        recordId
      })
      record.status = 'synced'
      record.version = record.version + 1
      const uploadRes = await uploadRecord(record.id, record, key)
      if (uploadRes.success) {
        if (uploadRes.new_version != null) {
          await invoke('set_record_sync_status_version', {
            recordId: record.id,
            version: uploadRes.new_version,
            status: 'synced'
          })
        }
        syncProgress.value.details.pushed++
      } else {
        syncProgress.value.details.failed++
      }
    } catch (e) {
      console.error(`Failed to upload record ${recordId}:`, e)
      syncProgress.value.details.failed++
    }
    current++
    syncProgress.value.current = current
  }

  // 逐个下载，及时更新进度
  let downloaded = []
  for (const rc_id of result.pull_list) {
    syncProgress.value.phase = 'pulling'
    try {
      const fullRecord = await downloadRecord(rc_id, key)
      downloaded.push(fullRecord)
      syncProgress.value.details.pulled++
    } catch (e) {
      console.error(`Failed to download record ${rc_id}:`, e)
      syncProgress.value.details.failed++
    }
    current++
    syncProgress.value.current = current
  }
  console.log('downloaded:', downloaded)
  await applyPullRecords(downloaded)

  syncProgress.value.phase = 'updating'
}

// ---------- 冲突解决 ----------
const handleConflictResolution = async (resolutions: ResolvedConflict[]) => {
  showConflictResolver.value = false
  syncing.value = true
  showProgress.value = true

  try {
    const conflicts = handshakeResult.value?.conflicts || []
    if (!conflicts.length || !handshakeResult.value) {
      showError('没有待解决的冲突', 'error')
      return
    }

    const resolvedPushList = []
    const resolvedPullList = []

    for (const res of resolutions) {
      const conflict = conflicts.find((c) => c.id === res.id)
      if (!conflict) continue

      // const newVersion = Math.max(conflict.local_version, conflict.server_version) + 1

      if (res.resolution === 'keep_local') {
        // 保留本地：加入push 列表,覆盖服务器
        resolvedPushList.push(res.id)
      } else {
        // 保留服务端：加入pull 列表,覆盖本地
        resolvedPullList.push(res.id)
      }
    }

    // 合并原有的 push/pull 列表
    const merged: SyncHandshakeResult = {
      push_list: [...handshakeResult.value.push_list, ...resolvedPushList],
      pull_list: [...handshakeResult.value.pull_list, ...resolvedPullList],
      conflicts: []
    }

    await doSync(merged, authKey.value)
    showSuccess('同步完成', 'success')
    refreshSyncStats()
  } catch (e) {
    console.error('Conflict resolution failed:', e)
    showError(`冲突解决失败：${e}`, 'error')
  } finally {
    syncing.value = false
    showProgress.value = false
  }
}
</script>

<style scoped>
.sync-page {
  max-width: 600px;
  margin: 0 auto;
  padding: 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.sync-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary, #6b7280);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0;
}

/* 信息卡片 */
.info-card {
  background: var(--card-bg, #ffffff);
  border-radius: 12px;
  border: 1px solid var(--border-color, #e5e7eb);
  overflow: hidden;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
}

.info-row + .info-row {
  border-top: 1px solid var(--border-color, #e5e7eb);
}

.info-label {
  font-size: 14px;
  color: var(--text-secondary, #6b7280);
}

.info-value {
  font-size: 14px;
  color: var(--text-primary, #1f2937);
  display: flex;
  align-items: center;
  gap: 8px;
}

.auth-key {
  font-size: 12px;
  padding: 2px 8px;
  background: var(--input-bg, #f3f4f6);
  border-radius: 4px;
}

/* 状态圆点 */
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #d1d5db;
  flex-shrink: 0;
}

.status-dot.dot-online {
  background: #10b981;
  box-shadow: 0 0 4px rgba(16, 185, 129, 0.5);
}

.status-dot.dot-offline {
  background: #ef4444;
  box-shadow: 0 0 4px rgba(239, 68, 68, 0.5);
}

/* 检测连接行 */
.info-row--action {
  justify-content: flex-start;
  gap: 12px;
  background: var(--input-bg, #f9fafb);
}

.btn--sm {
  padding: 6px 14px;
  font-size: 13px;
}

.check-msg {
  font-size: 13px;
}

.check-msg.msg-ok {
  color: #10b981;
}

.check-msg.msg-err {
  color: #ef4444;
}

.text-warning {
  color: #f59e0b;
}

.mono {
  font-family: monospace;
  font-size: 13px;
}

/* 授权码输入 */
.auth-input-row {
  display: flex;
  gap: 8px;
}

.input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 8px;
  font-size: 14px;
  background: var(--input-bg, #f9fafb);
  color: var(--text-primary, #1f2937);
  outline: none;
  transition: border-color 0.2s;
}

.input:focus {
  border-color: #3b82f6;
}

/* 按钮 */
.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: #3b82f6;
  color: #fff;
}

.btn-primary:hover {
  background: #2563eb;
}

.btn-secondary {
  background: var(--input-bg, #f3f4f6);
  color: var(--text-primary, #1f2937);
  border: 1px solid var(--border-color, #e5e7eb);
}

.btn-secondary:hover {
  background: var(--border-color, #e5e7eb);
}

.btn-text {
  background: none;
  border: none;
  color: #3b82f6;
  font-size: 13px;
  cursor: pointer;
  padding: 0;
}

.btn-text:hover {
  text-decoration: underline;
}

/* 徽标 */
.badge {
  padding: 2px 10px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
}

.badge-success {
  background: #d1fae5;
  color: #065f46;
}

.badge-warning {
  background: #fef3c7;
  color: #92400e;
}

.badge-neutral {
  background: #f3f4f6;
  color: #9ca3af;
}

.badge-danger {
  background: #fde8e8;
  color: #dc2626;
}

/* 同步按钮 */
.sync-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 14px;
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: #fff;
  border: none;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(16, 185, 129, 0.3);
}

.sync-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.4);
}

.sync-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

/* 帮助文本 */
.help-text {
  background: var(--card-bg, #ffffff);
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 12px;
  padding: 16px;
  font-size: 14px;
  line-height: 1.8;
  color: var(--text-secondary, #6b7280);
}

.help-text p {
  margin: 0;
}
</style>

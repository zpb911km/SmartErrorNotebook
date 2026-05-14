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
          <input v-model="urlInput" type="text" placeholder="http://localhost:5000" class="input" />
          <button class="btn btn-primary" @click="saveServerUrl">保存</button>
          <button class="btn btn-secondary" @click="showUrlInput = false">取消</button>
        </div>
        <div class="info-row">
          <span class="info-label">授权码</span>
          <div class="info-value">
            <span class="status-dot" :class="authStatusClass"></span>
            <template v-if="authKey">
              <code class="auth-key">{{ maskedAuthKey }}</code>
              <button class="btn-text" @click="showAuthInput = true">更换</button>
            </template>
            <template v-else>
              <span class="text-warning">未配置</span>
              <button class="btn-text" @click="showAuthInput = true">去设置</button>
            </template>
          </div>
        </div>
        <div class="info-row info-row--action">
          <button class="btn btn-secondary btn--sm" @click="checkConnection" :disabled="checking">
            {{ checking ? '检测中...' : '检测连接' }}
          </button>
          <span v-if="checkMessage" class="check-msg" :class="checkMsgClass">{{ checkMessage }}</span>
        </div>
      </div>
      <div v-if="showAuthInput" class="auth-input-row">
        <input v-model="authInput" type="text" placeholder="输入授权码..." class="input" />
        <button class="btn btn-primary" @click="saveAuthKey">保存</button>
        <button class="btn btn-secondary" @click="showAuthInput = false">取消</button>
      </div>
    </section>

    <!-- 同步状态区 -->
    <section class="sync-section">
      <h2 class="section-title">数据同步</h2>
      <div class="info-card">
        <div class="info-row">
          <span class="info-label">待同步记录</span>
          <span class="info-value">
            <span class="badge" :class="pendingCount > 0 ? 'badge-warning' : 'badge-success'">
              {{ pendingCount }}
            </span>
          </span>
        </div>
        <div class="info-row">
          <span class="info-label">上次同步</span>
          <span class="info-value">{{ lastSyncText }}</span>
        </div>
      </div>

      <button
        class="sync-btn"
        :disabled="syncing"
        @click="handleSync"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="20" height="20">
          <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
          <path d="M3 3v5h5"/>
          <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
          <path d="M16 16h5v5"/>
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
import { handshake, getAllSyncData, uploadRecords, applyPullRecords } from '../apis/sync'
import type { SyncHandshakeResult, ConflictInfo, ResolvedConflict, ServerRecord } from '../apis/sync'
import SyncOverlay from '../components/SyncOverlay.vue'
import ConflictResolver from '../components/ConflictResolver.vue'

const AUTH_STORAGE_KEY = 'auth_key'
const URL_STORAGE_KEY = 'sync_server_url'

// ---------- 服务器地址 ----------
const serverUrl = ref(localStorage.getItem(URL_STORAGE_KEY) || 'http://localhost:5000')
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

const checkMsgClass = computed(() => checkOk.value ? 'msg-ok' : 'msg-err')

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
const pendingCount = ref(0)
const lastSyncTime = ref<number | null>(null)
const syncing = ref(false)
const showProgress = ref(false)
const showConflictResolver = ref(false)
const conflictsToResolve = ref<ConflictInfo[]>([])
const handshakeResult = ref<SyncHandshakeResult | null>(null)
const syncProgress = ref({
  phase: 'handshake' as string,
  current: 0,
  total: 0,
  details: { pulled: 0, pushed: 0, conflicts_resolved: 0, failed: 0 },
})

const syncTotal = computed(() => {
  const r = handshakeResult.value
  return r ? r.push_list.length + r.pull_list.length : 0
})

const lastSyncText = computed(() => {
  if (!lastSyncTime.value) return '从未同步'
  const d = new Date(lastSyncTime.value)
  return d.toLocaleString('zh-CN')
})

const showToast = (message: string, type: 'success' | 'error' | 'info') => {
  console.log(`[${type.toUpperCase()}] ${message}`)
}

// ---------- 刷新待同步计数 ----------
const refreshPendingCount = async () => {
  try {
    const records = await invoke<ServerRecord[]>('get_all_pending_records')
    pendingCount.value = records.length
  } catch {
    pendingCount.value = 0
  }
}

onMounted(refreshPendingCount)

// ---------- 同步流程 ----------
const handleSync = async () => {
  if (!authKey.value) {
    showToast('请先配置授权码', 'error')
    return
  }

  syncing.value = true
  showProgress.value = true
  syncProgress.value = { phase: 'handshake', current: 0, total: 0, details: { pulled: 0, pushed: 0, conflicts_resolved: 0, failed: 0 } }

  try {
    const [localPending, remoteRecords] = await Promise.all([
      invoke<ServerRecord[]>('get_all_pending_records'),
      getAllSyncData(authKey.value),
    ])

    const result = handshake(localPending, remoteRecords)
    handshakeResult.value = result

    if (result.conflicts.length > 0) {
      syncProgress.value.phase = 'resolving_conflicts'
      conflictsToResolve.value = result.conflicts
      return
    }

    await doSync(result, authKey.value)
    lastSyncTime.value = Date.now()
    showToast('同步完成', 'success')
    refreshPendingCount()
  } catch (e) {
    console.error('Sync failed:', e)
    showToast(`同步失败：${e}`, 'error')
  } finally {
    if (!showConflictResolver.value) {
      syncing.value = false
      showProgress.value = false
    }
  }
}

const doSync = async (result: SyncHandshakeResult, key: string) => {
  let current = 0
  const total = result.push_list.length + result.pull_list.length
  syncProgress.value.total = total

  if (result.push_list.length > 0) {
    syncProgress.value.phase = 'pushing'
    const records = await Promise.all(
      result.push_list.map(id =>
        invoke<ServerRecord>('get_record_for_upload', { recordId: id })
      )
    )
    const uploadResults = await uploadRecords(records, key)
    for (const succeeded of uploadResults.values()) {
      current++
      if (succeeded) syncProgress.value.details.pushed++
      else syncProgress.value.details.failed++
    }
    syncProgress.value.current = current
  }

  if (result.pull_list.length > 0) {
    syncProgress.value.phase = 'pulling'
    await applyPullRecords(result.pull_list)
    syncProgress.value.details.pulled = result.pull_list.length
    current += result.pull_list.length
    syncProgress.value.current = current
  }

  syncProgress.value.phase = 'updating'
}

// ---------- 冲突解决 ----------
const handleConflictResolution = async (resolutions: ResolvedConflict[]) => {
  showConflictResolver.value = false
  syncing.value = true
  showProgress.value = true

  try {
    for (const resolution of resolutions) {
      await invoke('apply_conflict_resolution', { resolution })
    }

    const [localPending, remoteRecords] = await Promise.all([
      invoke<ServerRecord[]>('get_all_pending_records'),
      getAllSyncData(authKey.value),
    ])

    const newResult = handshake(localPending, remoteRecords)
    handshakeResult.value = newResult

    if (newResult.push_list.length > 0 || newResult.pull_list.length > 0) {
      await doSync(newResult, authKey.value)
      showToast('同步完成', 'success')
    } else {
      showToast('所有冲突已解决，无需同步', 'info')
    }
    lastSyncTime.value = Date.now()
    refreshPendingCount()
  } catch (e) {
    console.error('Conflict resolution failed:', e)
    showToast(`冲突解决失败：${e}`, 'error')
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

<template>
  <div class="community-page">
    <!-- 顶部导航栏 -->
    <div class="community-header">
      <button class="back-btn" @click="goBack">
        <Icon name="arrow-left" :size="16" class="back-icon" />
        <span>返回</span>
      </button>
      <h2>🌐 错题社区</h2>
    </div>

    <!-- 加载中 -->
    <div v-if="loading && items.length === 0" class="state-box">
      <div class="loading-spinner"></div>
      <p>加载中...</p>
    </div>

    <!-- 错误提示 -->
    <div v-else-if="errorMsg" class="state-box">
      <p class="error-text">{{ errorMsg }}</p>
      <button class="btn btn-primary" @click="loadData">重试</button>
    </div>

    <!-- 空状态 -->
    <div v-else-if="items.length === 0" class="empty-illustration">
      <div class="empty-icon is-share"></div>
      <div class="empty-title">暂无分享的错题</div>
      <div class="empty-desc">还没有人分享错题，快来分享第一道吧！</div>
    </div>

    <!-- 分享列表 -->
    <div v-else class="community-list">
      <div
        v-for="(item, index) in items"
        :key="item.id"
        v-scroll-reveal="{ delay: index * 80 }"
        class="share-card"
      >
        <!-- 题目预览 -->
        <div
          class="card-prompt markdown-body"
          v-html="renderMarkdown(item.prompt)"
        ></div>

        <!-- 元信息 + 操作 -->
        <div class="card-footer">
          <div class="card-meta">
            <span class="meta-type">{{ item.type_ }}</span>
            <span class="meta-time">{{ formatTime(item.created_at) }}</span>
          </div>
          <button class="btn btn-primary btn--sm" @click="handleFetch(item)">
            获取
          </button>
        </div>
      </div>

      <!-- 底部加载 -->
      <div v-if="loadingMore" class="load-more">
        <div class="loading-spinner"></div>
        <span>加载更多...</span>
      </div>
      <div v-else-if="!hasMore && items.length > 0" class="load-more">
        — 已经到底了 —
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { marked } from 'marked'
import markedKatex from 'marked-katex-extension'
import { fetchShareList } from '../apis/share'
import type { SharedQuestionItem } from '../apis/share'
import { setSharedData } from '../services/shareStore'

marked.use(
  markedKatex({
    throwOnError: false,
    output: 'html'
  })
)

const renderer = new marked.Renderer()
marked.use({ renderer })

const router = useRouter()

const AUTH_KEY = 'auth_key'
const PAGE_SIZE = 20

const items = ref<SharedQuestionItem[]>([])
const page = ref(0)
const hasMore = ref(true)
const loading = ref(false)
const loadingMore = ref(false)
const errorMsg = ref('')

function getAuthKey(): string {
  return localStorage.getItem(AUTH_KEY) || ''
}

function formatTime(ts: number): string {
  const date = new Date(ts)
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function renderMarkdown(content: string): string {
  if (!content) return ''
  const normalized = content
    .replace(/\\\[/g, '$$$$')
    .replace(/\\\]/g, '$$$$')
    .replace(/\\\(/g, '$$')
    .replace(/\\\)/g, '$$')
  try {
    return marked.parse(normalized, { breaks: true, gfm: true }) as string
  } catch {
    return `${content}`
  }
}

async function loadData() {
  const authKey = getAuthKey()
  if (!authKey) {
    errorMsg.value = '未配置授权码，请在同步页面配置后重试'
    return
  }

  loading.value = true
  errorMsg.value = ''
  page.value = 0
  items.value = []
  hasMore.value = true

  try {
    const data = await fetchShareList({
      auth_key: authKey,
      page: 1,
      page_size: PAGE_SIZE
    })
    items.value = data.items
    page.value = 1
    hasMore.value = data.has_more
  } catch (e: any) {
    errorMsg.value = e.message || '加载失败'
  } finally {
    loading.value = false
  }
}

async function loadMore() {
  if (loadingMore.value || !hasMore.value) return
  loadingMore.value = true
  try {
    const data = await fetchShareList({
      auth_key: getAuthKey(),
      page: page.value + 1,
      page_size: PAGE_SIZE
    })
    items.value.push(...data.items)
    page.value++
    hasMore.value = data.has_more
  } catch (e: any) {
    console.error('加载更多失败:', e)
  } finally {
    loadingMore.value = false
  }
}

function handleFetch(item: SharedQuestionItem) {
  setSharedData({
    prompt: item.prompt,
    type_: item.type_,
    answer: item.answer,
    analysis: item.analysis,
    error_note: item.error_note
  })
  router.push({ name: 'Add' })
}

function goBack() {
  router.back()
}

// ==================== 无限滚动 ====================
let scrollContainer: HTMLElement | null = null

function onScroll() {
  if (!scrollContainer) return
  const { scrollTop, scrollHeight, clientHeight } = scrollContainer
  if (scrollHeight - scrollTop - clientHeight < 200) {
    loadMore()
  }
}

onMounted(() => {
  scrollContainer = document.querySelector(
    '.community-page'
  ) as HTMLElement | null
  scrollContainer?.addEventListener('scroll', onScroll)
  loadData()
})

onUnmounted(() => {
  scrollContainer?.removeEventListener('scroll', onScroll)
})
</script>

<style scoped>
.community-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  background: var(--bg-secondary);
}

.community-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--card-bg);
  border-bottom: 1px solid var(--border-color);
  position: sticky;
  top: 0;
  z-index: 10;
}

.community-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.back-btn {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  border: none;
  background: none;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: var(--font-size-base);
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-md);
}

.back-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

/* 状态容器 */
.state-box {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-secondary);
}

.hint-text {
  font-size: 13px;
  color: var(--text-hint);
  margin-top: 8px;
}

.error-text {
  color: var(--danger-color);
  margin-bottom: 12px;
}

/* 卡片列表 */
.community-list {
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.share-card {
  background: var(--card-bg);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.share-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-lg);
}

/* 题目内容 — 限制高度 + 渐变遮罩 */
.card-prompt {
  padding: 16px 16px 8px;
  max-height: 280px;
  overflow: hidden;
  position: relative;
  font-size: 14px;
  line-height: 1.6;
}

.card-prompt::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 48px;
  background: linear-gradient(transparent, var(--card-bg));
  pointer-events: none;
}

/* 底部 */
.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px 12px;
  border-top: 1px solid var(--border-color);
}

.card-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.meta-type {
  display: inline-block;
  padding: 2px 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

.meta-time {
  font-size: var(--font-size-sm);
  color: var(--text-hint);
}

/* 加载 */
.load-more {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-lg);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 3px solid var(--border-color);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 按钮 */
.btn {
  padding: var(--spacing-sm) var(--spacing-lg);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--font-size-base);
}

.btn-primary {
  background: var(--primary-color);
  color: #fff;
}

.btn-primary:hover {
  opacity: 0.9;
}

.btn--sm {
  padding: 6px 12px;
  font-size: var(--font-size-sm);
}

/* markdown 渲染样式 */
.card-prompt.markdown-body {
  font-size: 14px;
  line-height: 1.7;
  color: var(--text-primary);
}

.card-prompt.markdown-body :deep(h1),
.card-prompt.markdown-body :deep(h2),
.card-prompt.markdown-body :deep(h3),
.card-prompt.markdown-body :deep(h4),
.card-prompt.markdown-body :deep(h5),
.card-prompt.markdown-body :deep(h6) {
  margin: 0.6em 0 0.4em;
  font-weight: 600;
  line-height: 1.3;
}

.card-prompt.markdown-body :deep(h1) {
  font-size: 1.3em;
}
.card-prompt.markdown-body :deep(h2) {
  font-size: 1.15em;
}
.card-prompt.markdown-body :deep(h3) {
  font-size: 1.05em;
}

.card-prompt.markdown-body :deep(p) {
  margin: 0.4em 0;
}

.card-prompt.markdown-body :deep(code) {
  padding: 0.15em 0.35em;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  font-size: 0.9em;
  font-family: 'Courier New', Courier, monospace;
}

.card-prompt.markdown-body :deep(pre) {
  padding: 0.8em;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  overflow-x: auto;
  font-size: 0.9em;
  line-height: 1.5;
}

.card-prompt.markdown-body :deep(pre code) {
  padding: 0;
  background: none;
  border-radius: 0;
}

.card-prompt.markdown-body :deep(ul),
.card-prompt.markdown-body :deep(ol) {
  padding-left: 1.5em;
  margin: 0.4em 0;
}

.card-prompt.markdown-body :deep(blockquote) {
  margin: 0.4em 0;
  padding: 0.2em 0.8em;
  border-left: 3px solid var(--border-color);
  color: var(--text-secondary);
}

.card-prompt.markdown-body :deep(a) {
  color: var(--primary-color);
  text-decoration: none;
}

.card-prompt.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.card-prompt.markdown-body :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.4em 0;
  font-size: 0.9em;
}

.card-prompt.markdown-body :deep(th),
.card-prompt.markdown-body :deep(td) {
  border: 1px solid var(--border-color);
  padding: 0.3em 0.6em;
  text-align: left;
}

.card-prompt.markdown-body :deep(th) {
  background: var(--bg-tertiary);
  font-weight: 600;
}
</style>

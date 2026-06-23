/**
 * 社区分享 API
 */

function getBaseUrl(): string {
  return localStorage.getItem('sync_server_url') || 'http://localhost:5000'
}

/** 分享列表项（公开格式，不含 sender_auth_key） */
export interface SharedQuestionItem {
  id: string
  prompt: string
  type_: string
  answer: string
  analysis: string
  error_note: string
  created_at: number
}

/** 分享列表响应 */
interface ShareListResponse {
  items: SharedQuestionItem[]
  has_more: boolean
}

/**
 * 分享错题到社区
 */
export async function publishShare(params: {
  auth_key: string
  id: string
  prompt: string
  type_: string
  answer?: string
  analysis?: string
  error_note?: string
}): Promise<void> {
  const res = await fetch(`${getBaseUrl()}/api/share/publish`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
  })
  if (!res.ok) {
    const data = await res.json().catch(() => ({}))
    throw new Error(data.error || `publish failed (${res.status})`)
  }
}

/**
 * 获取社区分享列表
 */
export async function fetchShareList(params: {
  auth_key: string
  page?: number
  page_size?: number
}): Promise<ShareListResponse> {
  const query = new URLSearchParams({
    auth_key: params.auth_key,
    page: String(params.page ?? 1),
    page_size: String(params.page_size ?? 20),
  })
  const res = await fetch(`${getBaseUrl()}/api/share/list?${query}`)
  if (!res.ok) {
    const data = await res.json().catch(() => ({}))
    throw new Error(data.error || `list failed (${res.status})`)
  }
  return res.json()
}

/**
 * 撤回分享的错题
 */
export async function revokeShare(params: {
  auth_key: string
  id: string
}): Promise<void> {
  const res = await fetch(`${getBaseUrl()}/api/share/revoke`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
  })
  if (!res.ok) {
    const data = await res.json().catch(() => ({}))
    throw new Error(data.error || `revoke failed (${res.status})`)
  }
}

/**
 * 检查指定题目是否已分享
 */
export async function checkShare(params: {
  auth_key: string
  id: string
}): Promise<boolean> {
  const query = new URLSearchParams({
    auth_key: params.auth_key,
    id: params.id,
  })
  const res = await fetch(`${getBaseUrl()}/api/share/check?${query}`)
  if (!res.ok) {
    const data = await res.json().catch(() => ({}))
    throw new Error(data.error || `check failed (${res.status})`)
  }
  const data: { shared: boolean } = await res.json()
  return data.shared
}

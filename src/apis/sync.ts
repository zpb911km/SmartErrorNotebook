/**
 * 同步 API 和协议实现 - 所有表同步
 */

import { invoke } from '@tauri-apps/api/core';

// ==================== 服务器响应数据类型 ====================

/** 服务器记录通用格式（to_dict 输出） */
export interface ServerRecord {
  id: string;                    // 记录 ID (UUID)
  table_name: string;            // 表名 (error_questions, subjects, srs_data, attachments, error_tags, sources)
  version: number;               // 版本号
  status: 'synced' | 'pending';  // 同步状态
  deleted_at: number | null;     // 软删除时间戳
  updated_at: number;            // 更新时间
  data: object;                  // 所有业务字段
}

/** 获取所有同步数据响应 */
interface GetAllSyncDataResponse {
  all_records: ServerRecord[];
}

/** 上传记录响应 */
interface UploadRecordResponse {
  success: boolean;
  new_version?: number;
  conflict?: boolean;
  server_version?: number;
  error?: string;
}

/** 下载记录响应 */
interface DownloadRecordResponse {
  record: ServerRecord;
}

// ==================== 同步状态类型 ====================

/** 握手结果 */
export interface SyncHandshakeResult {
  pull_list: ServerRecord[];     // 需要从服务端拉取的记录
  push_list: string[];   // 可以安全推送到服务端的记录 ID
  conflicts: ConflictInfo[];     // 需要用户处理的冲突
}

/** 冲突信息 */
export interface ConflictInfo {
  id: string;                      // 记录 ID
  local_version: number;           // 本地版本
  server_version: number;          // 服务端版本
  local_deleted: boolean;          // 本地是否已软删除
  server_deleted: boolean;         // 服务端是否已软删除
}

/** 冲突解决方案 */
export type ConflictResolution = 'keep_local' | 'keep_remote' | 'merge';

/** 解决后的冲突记录 */
export interface ResolvedConflict {
  id: string;
  resolution: ConflictResolution;
  final_data?: object; // merge 时需要
  final_deleted_at?: number | null;
}

// ==================== 进度类型 ====================

/** 同步进度状态 */
export interface SyncProgress {
  phase: 'handshake' | 'pulling' | 'pushing' | 'resolving_conflicts' | 'updating';
  total: number;
  current: number;
  details: {
    pulled: number;
    pushed: number;
    conflicts_resolved: number;
    failed: number;
  };
}

// ==================== API 接口函数 ====================

const SYNC_SERVER_BASE_URL = 'http://localhost:5000';

/**
 * 验证 auth_key
 */
export async function validateAuthKey(auth_key: string): Promise<boolean> {
  const res = await fetch(`${SYNC_SERVER_BASE_URL}/api/auth/validate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ auth_key }),
  });
  const data = await res.json();
  return data.valid === true;
}

/**
 * 获取当前用户的所有同步数据（服务端所有记录）
 */
export async function getAllSyncData(auth_key: string): Promise<ServerRecord[]> {
  const res = await fetch(
    `${SYNC_SERVER_BASE_URL}/api/sync/get_all_sync_data?auth_key=${encodeURIComponent(auth_key)}`
  );
  if (!res.ok) {
    throw new Error(`Failed to fetch sync data: ${res.statusText}`);
  }
  const data: GetAllSyncDataResponse = await res.json();
  return data.all_records;
}

/**
 * 上传单个记录到服务端
 */
export async function uploadRecord(
  record_id: string,
  record: ServerRecord,
  auth_key: string
): Promise<UploadRecordResponse> {
  const res = await fetch(
    `${SYNC_SERVER_BASE_URL}/api/sync/upload/${record_id}`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ...record, auth_key }),
    }
  );
  return res.json();
}

/**
 * 从服务端下载单条记录
 */
export async function downloadRecord(
  record_id: string,
  auth_key: string
): Promise<ServerRecord> {
  const res = await fetch(
    `${SYNC_SERVER_BASE_URL}/api/sync/download/${record_id}?auth_key=${encodeURIComponent(auth_key)}`
  );
  if (!res.ok) {
    throw new Error(`Failed to download record: ${res.statusText}`);
  }
  const data: DownloadRecordResponse = await res.json();
  return data.record;
}

// ==================== 握手算法（客户端核心逻辑） ====================

/**
 * 执行握手算法，计算拉取表、推送表和冲突列表
 *
 * @param local_pending 本地所有状态为 pending 的记录数组
 * @param remote_records 服务端返回的所有记录
 * @returns SyncHandshakeResult
 */
export function handshake(
  local_pending: Array<{
    id: string;
    version: number;
    status: 'pending' | 'synced';
    deleted_at: number | null;
  } & { [key: string]: any }>,
  remote_records: ServerRecord[]
): SyncHandshakeResult {
  const result: SyncHandshakeResult = {
    pull_list: [],
    push_list: [],
    conflicts: [],
  };

  // 构建远程记录 Map（按 table_name + id 组合键）
  const remote_map = new Map<string, ServerRecord>();
  for (const rec of remote_records) {
    const key = `${rec.table_name}:${rec.id}`;
    remote_map.set(key, rec);
  }

  // ========== c -> s: 处理本地 pending 记录 ==========
  for (const local_rec of local_pending) {
    const rec_id = local_rec.id;
    const local_status = local_rec.status;
    const local_version = local_rec.version;

    // 查找对应的远程记录（需要从原始数据中匹配）
    let server_rec: ServerRecord | undefined;
    for (const rec of remote_records) {
      if (rec.id === rec_id && /* 假设其他字段也能匹配 */ true) {
        server_rec = rec;
        break;
      }
    }
    const remote_exists = server_rec !== undefined;

    if (remote_exists) {
      const server_version = server_rec.version;

      // 条件：local.status == 'pending' && (remote not exists || local.version == remote.version)
      if (local_status === 'pending' && local_version === server_version) {
        // 可以推送
        result.push_list.push(rec_id);
      } else {
        // CONFLICT
        result.conflicts.push({
          id: rec_id,
          local_version,
          server_version,
          local_deleted: local_rec.deleted_at !== null,
          server_deleted: server_rec.deleted_at !== null,
        });
      }
    } else {
      // 服务端不存在，可以直接推送（新建记录）
      result.push_list.push(rec_id);
    }
  }

  // ========== s -> c: 处理需要拉取的记录 ==========
  for (const server_rec of remote_records) {
    const rec_id = server_rec.id;
    const remote_deleted = server_rec.deleted_at !== null;

    // 查找本地记录
    const local_rec = local_pending.find((r) => r.id === rec_id);
    const local_exists = local_rec !== undefined;

    if (!local_exists) {
      // 本地不存在：如果服务端未被删除，则需要拉取
      if (!remote_deleted) {
        result.pull_list.push(server_rec);
      }
    } else {
      // 本地存在
      const local_status = local_rec.status;
      const local_version = local_rec.version;

      // 条件：(local.status == 'synced' && local.version < remote.version)
      if (local_status === 'synced' && local_version < server_rec.version) {
        // 需要拉取更新
        result.pull_list.push(server_rec);
      }
    }
  }

  return result;
}

/**
 * 应用拉取的记录到本地数据库
 * TS 根据 table_name 路由到各个实体的具体 upsert 接口
 */
export async function applyPullRecords(records: ServerRecord[]): Promise<void> {
  for (const rec of records) {
    const { table_name, id, version, status, deleted_at, updated_at, data } = rec;

    switch (table_name) {
      case 'error_questions':
        await invoke('upsert_error_question', {
          input: { id, version, status, deleted_at, ...data },
        });
        break;
      case 'subjects':
        await invoke('upsert_subject', {
          input: { id, version, status, deleted_at, ...data },
        });
        break;
      case 'srs_data':
        await invoke('upsert_srs_data', {
          input: { id, version, status, deleted_at, ...data },
        });
        break;
      case 'attachments':
        await invoke('upsert_attachment', {
          input: { id, version, status, deleted_at, ...data },
        });
        break;
      case 'error_tags':
        await invoke('upsert_error_tag', {
          input: { id, version, status, deleted_at, ...data },
        });
        break;
      case 'sources':
        await invoke('upsert_source', {
          input: { id, version, status, deleted_at, ...data },
        });
        break;
      default:
        console.warn(`Unknown table_name: ${table_name}`);
        break;
    }
  }
}

/**
 * 批量上传记录到服务器
 * @param records - 包含 table_name 的完整记录
 * @param auth_key - 用户认证 key
 */
export async function uploadRecords(
  records: ServerRecord[],
  auth_key: string
): Promise<Map<string, boolean>> {
  const results = new Map<string, boolean>();

  // 并发上传，限制同时请求数
  const concurrencyLimit = 5;

  for (let i = 0; i < records.length; i += concurrencyLimit) {
    const chunk = records.slice(i, i + concurrencyLimit);
    await Promise.all(
      chunk.map(async (rec) => {
        try {
          const res = await uploadRecord(rec.id, rec, auth_key);
          results.set(rec.id, res.success);
        } catch (e) {
          console.error(`Failed to upload record ${rec.id}:`, e);
          results.set(rec.id, false);
        }
      })
    );
  }

  return results;
}

/**
 * 解决冲突 - 根据用户选择生成最终数据
 */
export function resolveConflict(
  conflict: ConflictInfo,
  resolution: ConflictResolution
): ResolvedConflict {
  return {
    id: conflict.id,
    resolution,
    final_deleted_at:
      resolution === 'keep_local' ? (conflict.local_deleted ? null : Date.now()) : (conflict.server_deleted ? Date.now() : null),
  };
}

/**
 * 应用冲突解决结果
 */
export async function applyConflictResolution(resolved: ResolvedConflict[]): Promise<void> {
  for (const res of resolved) {
    await invoke('apply_conflict_resolution', {
      resolution: res,
    });
  }
}

/**
 * 获取本地所有 pending 记录（通过 Rust invoke）
 * 返回包含 table_name 的完整记录列表
 */
export async function getLocalPendingRecords(): Promise<ServerRecord[]> {
  return invoke('get_all_pending_records');
}

/**
 * 开始同步流程（主入口）
 */
export async function startSync(options: {
  onProgress?: (progress: SyncProgress) => void;
  onConflict?: (conflicts: ConflictInfo[]) => Promise<ResolvedConflict[]>;
}): Promise<SyncProgress> {
  const { onProgress, onConflict } = options || {};
  const auth_key = localStorage.getItem('auth_key') || '';

  if (!auth_key) {
    throw new Error('No auth_key found. Please login first.');
  }

  // Step 1: 获取服务端数据
  const remote_records = await getAllSyncData(auth_key);

  // Step 2: 获取本地 pending 记录
  const local_pending = await getLocalPendingRecords();

  // Step 3: 执行握手
  const handshake_result = handshake(local_pending, remote_records);

  let progress: SyncProgress = {
    phase: 'handshake',
    total:
      handshake_result.pull_list.length +
      handshake_result.push_list.length,
    current: 0,
    details: { pulled: 0, pushed: 0, conflicts_resolved: 0, failed: 0 },
  };

  if (onProgress) {
    onProgress(progress);
  }

  // Step 4: 处理冲突（如果有）
  if (handshake_result.conflicts.length > 0) {
    progress.phase = 'resolving_conflicts';
    if (onProgress) {
      onProgress(progress);
    }

    if (onConflict) {
      const resolved = await onConflict(handshake_result.conflicts);
      progress.details.conflicts_resolved = resolved.length;

      // 应用冲突解决
      await applyConflictResolution(resolved);
    } else {
      throw new Error('Conflicts detected but no handler provided');
    }
  }

  // Step 5: 推送记录
  progress.phase = 'pushing';
  if (handshake_result.push_list.length > 0) {
    const push_records: ServerRecord[] = [];

    for (const record_id of handshake_result.push_list) {
      // 需要从本地数据库中获取完整的记录数据（包含 table_name）
      const record = await invoke('get_record_for_upload', {
        record_id,
      }) as ServerRecord;
      push_records.push(record);
    }

    // 调用服务器上传（需要 auth_key）
    const upload_results = await uploadRecords(push_records, auth_key);

    for (const succeeded of upload_results.values()) {
      if (succeeded) {
        progress.details.pushed++;
      } else {
        progress.details.failed++;
      }
    }
  }
  progress.current += handshake_result.push_list.length;
  if (onProgress) {
    onProgress(progress);
  }

  // Step 6: 拉取记录
  progress.phase = 'pulling';
  if (handshake_result.pull_list.length > 0) {
    await applyPullRecords(handshake_result.pull_list);
    progress.details.pulled = handshake_result.pull_list.length;
    progress.current += handshake_result.pull_list.length;
    if (onProgress) {
      onProgress(progress);
    }
  }

  // Step 7: 更新状态
  progress.phase = 'updating';

  progress.total = progress.current;
  if (onProgress) {
    onProgress(progress);
  }

  return progress;
}

// ==================== 导出 ====================

export type {
  GetAllSyncDataResponse,
  UploadRecordResponse,
  DownloadRecordResponse,
};

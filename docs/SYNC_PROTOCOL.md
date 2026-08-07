# 多用户多设备同步协议

> 本文档定义 Smart Error Notebook 的多设备同步机制，包括协议设计、握手算法、传输策略和服务器部署。

---

## 📑 目录

1. [概述](#-概述)
2. [核心约定](#-核心约定)
3. [握手算法](#-握手算法)
4. [传输阶段](#-传输阶段)
5. [更新阶段](#-更新阶段)
6. [冲突解决](#-冲突解决)
7. [软删除传播机制](#-软删除传播机制)
8. [重试机制](#-重试机制)
9. [服务器部署](#-服务器部署)
10. [API 端点参考](#-api-端点参考)

---

## 📖 概述

支持多设备间错题数据的同步，采用 **客户端主导 (Client-Driven)** 的同步模式：服务端仅作为无状态的数据中转站，所有握手决策、冲突检测、同步编排均由客户端独立完成。

```mermaid
sequenceDiagram
    participant A as 设备 A (SQLite)
    participant S as 同步服务器 (Flask)
    participant B as 设备 B (SQLite)

    Note over A: 本地修改 → status=pending
    A->>S: GET /api/sync/get_all_sync_data
    S-->>A: 返回所有记录的轻量 header（不含 data）
    Note over A: 本地执行 handshake()<br/>比对 local vs remote<br/>→ push_list / pull_list / conflicts
    A->>S: POST /api/sync/upload/<id> (推送 push_list)
    S->>S: version++
    S-->>A: 返回 new_version
    A->>S: GET /api/sync/download/<id> (拉取 pull_list)
    S-->>A: 返回完整记录
    Note over A: 更新本地数据库<br/>status=synced
    Note over A: 多轮握手直到<br/>push_list + pull_list 均为空

    Note over B: 类似流程，独立于设备 A
    B->>S: GET /api/sync/get_all_sync_data
    S-->>B: headers（含 A 推送的新记录）
    Note over B: 本地 handshake() → 拉取
    B->>S: GET /api/sync/download/<id>
    S-->>B: 完整记录
```

### 设计原则

1. **离线优先**：用户在任何环境下都可正常使用，无需网络连接
2. **版本驱动**：每条记录有独立递增版本号，用于冲突检测
3. **客户端主导**：所有握手决策在客户端本地执行，服务端不做编排，不维护同步状态机
4. **最小传输**：握手阶段只传轻量 header（不含 data 负载），避免传输大量数据
5. **最终一致**：多设备最终会达到一致状态

---

## 📋 核心约定

### 1. Version 增长规则

- **Version 仅在服务端接收客户端推送时递增**
- 客户端修改数据只设置 `status='pending'`，不改变 version
- 服务端 version 永远不低于任意客户端的 version（单调递增）

### 2. 数据字段分类

| 类别 | 字段 | 说明 |
|------|------|------|
| **ID 字段** | `id` | 记录唯一标识（UUID） |
| **同步字段** | `version`, `status`, `deleted_at` | 参与同步协议计算 |
| **数据字段** | 所有其他字段（含时间戳） | 不参与协议，仅随记录传输 |
| **用户标识** | `auth_key` | 服务端区分用户的 key，仅存在服务端 |

### 3. 流程

```
握手 → 传输 → 更新 → 再握手（空结果 = 完成）→ 结束或循环
```

### 4. 用户识别

- 使用单一的 `auth_key`（UUID）进行识别和验证
- auth_key 通过服务器管理页面生成并分发
- 每个 API 请求需携带 auth_key（存储在客户端 localStorage）
- 服务端按 `auth_key` 列进行数据分片

---

## 🤝 握手算法

**目的**：确认双方需要传输哪些数据、向哪个方向传输，以及提前发现冲突。

**执行位置**：**完全在客户端本地执行**。服务端不参与握手计算，仅提供一个全量 header 拉取接口供客户端获取远程状态。

**输入**：
1. 本地数据库全部记录的轻量 header（`status`, `version`, `deleted_at` 等，不含 data）
2. 服务端通过 `GET /api/sync/get_all_sync_data` 返回的远程全部记录 header

**输出**：三个列表 —— **拉取表**（服务端→客户端）、**推送表**（客户端→服务端）、**冲突表**（需用户介入）

### 判断逻辑

```mermaid
flowchart TD
    Start[开始握手] --> Check{local 是否存在<br/>对应 remote?}
    
    Check -->|不存在| NoRemote{local.status?}
    NoRemote -->|pending| Push[推送: c → s<br/>新建记录同步]
    NoRemote -->|synced| Push2[推送: c → s<br/>服务端丢失补传]
    
    Check -->|存在| BothExist
    
    BothExist --> Case1{local.status == pending<br/>AND<br/>local.version == remote.version?}
    Case1 -->|是| Push3[推送: c → s<br/>本地已修改待同步]
    
    Case1 -->|否| Case2{local.status == synced<br/>AND<br/>local.version < remote.version?}
    Case2 -->|是| Pull[拉取: s → c<br/>拉取服务端更新]
    
    Case2 -->|否| Case3{local.status == synced<br/>AND<br/>local.version == remote.version?}
    Case3 -->|是| Ignore[忽略: 两边一致<br/>无需操作]
    
    Case3 -->|否| Conflict[冲突: 需用户干预<br/>version 错位]
    
    subgraph Legend["汇总"]
        L1[Push: 客户端 → 服务端]
        L2[Pull: 服务端 → 客户端]
        L3[Ignore: 跳过]
        L4[Conflict: 弹窗解决]
    end
```

### 伪代码

```python
def handshake(local_records, remote_records):
    push_list = []    # 客户端 → 服务端
    pull_list = []    # 服务端 → 客户端
    conflict_list = []

    for local in local_records:
        remote = remote_records.get(local.id)

        if remote is None:
            # 服务端不存在此记录
            if local.status == 'pending':
                push_list.append(local)
            else:
                # 本地有、服务端无、本地也是 synced —— 说明服务端丢失，推送
                push_list.append(local)
        else:
            # 双方都有
            if local.status == 'pending' and local.version == remote.version:
                push_list.append(local)
            elif local.status == 'synced' and local.version < remote.version:
                pull_list.append(remote)
            elif local.status == 'synced' and local.version == remote.version:
                pass  # IGNORE
            else:
                conflict_list.append((local, remote))

    # 处理服务端有但本地没有的记录
    for remote_id, remote in remote_records.items():
        if remote_id not in {r.id for r in local_records}:
            if remote.deleted_at is None:
                pull_list.append(remote)
            # 已删除的不拉取

    return push_list, pull_list, conflict_list
```

---

## 📡 传输阶段

- **单位**：以记录为单位
- **方式**：并发异步传输
- **策略**：id 相同直接覆盖，不存在合并
- **顺序**：先处理推送表，再处理拉取表

### 推送

客户端将 push_list 中的记录通过 `POST /api/sync/upload/<record_id>` 逐条发送到服务端。服务端无论记录是否存在，均执行：

- **version 单调递增**：`server.version = server.version + 1`
- 用客户端数据覆盖记录内容
- 返回 `new_version`

客户端收到响应后，**用服务端返回的 `new_version` 覆盖本地 version**，并将 `status` 设为 `'synced'`。

> ⚠️ 客户端不自增 version，而是信任服务端返回的值。服务端是 version 的唯一权威来源，通过单调递增保证其他设备在握手时能检测到变化并拉取。

### 拉取

客户端通过 `GET /api/sync/download/<record_id>` 逐条获取 pull_list 中的完整记录。客户端：

1. 将服务端返回的完整记录写入本地数据库（按 `id` 覆盖）
2. 将本地 `version` 更新为服务端的 version
3. 将本地 `status` 设为 `'synced'`

---

## 🔄 更新阶段

根据握手结果更新同步字段：

| 操作 | 客户端更新 | 服务端更新 |
|------|-----------|-----------|
| 推送成功 | `status='synced'`, `version=server.new_version`（接受服务端返回值） | `version++`（始终递增，单调增长） |
| 拉取成功 | `status='synced'`, `version=remote.version` | 无 |
| 冲突解决后（保留本地） | 推送本地版本 → `status='synced'`, `version=server.new_version` | `version++` |
| 冲突解决后（采用远程） | `status='synced'`, `version=remote.version` | 无 |

---

## ⚔️ 冲突解决

### 触发条件

```mermaid
flowchart LR
    A[设备 A 修改记录<br/>status=pending, version=5] -->|未同步| C{设备 A 尝试同步}
    B[设备 B 修改同一条记录<br/>并成功推送<br/>status=synced, version=6] --> C
    C --> D[⚡ 检测到冲突]
```

### 解决方式

1. **自动检测**：握手时发现上述冲突条件
2. **用户介入**：弹出冲突解决界面，展示两个版本
3. **解决选项**：
   - **保留本地**：以本地版本覆盖服务端
   - **采用远程**：以服务端版本覆盖本地
   - **手动合并**：编辑合并后的内容

### 冲突解决界面

对应的前端组件：`ConflictResolver.vue` + `ConflictItem.vue`

---

## 🗑️ 软删除传播机制

软删除通过 version 递增保证至少向下同步一次：

```mermaid
sequenceDiagram
    participant A as 设备 A
    participant S as 同步服务器
    participant B as 设备 B

    Note over A: 删除记录<br/>deleted_at=now<br/>status=pending
    A->>S: 推送删除标记
    S->>S: version++ (5→6)
    S->>B: 拉取（version落后: 6 > 5）
    Note over B: 设置 deleted_at=now<br/>status=pending
    B->>S: 推送删除确认
    S->>S: version++ (6→7)
    Note over A: 清理条件:<br/>deleted_at != null<br/>&& status == 'synced'
    Note over B: 清理条件:<br/>deleted_at != null<br/>&& status == 'synced'
```

### 清理策略

- 当记录的 `deleted_at IS NOT NULL` 且 `status == 'synced'` 时，可在本地安全真删除
- **服务端保留已删除记录**（用于传播到其他设备），待所有设备确认后再清理

#### 本地真删除：`purgeSyncedDeletions()`

客户端 RPC 调用 `purge_synced_deletions`（Rust command），删除本地数据库中所有 `deleted_at IS NOT NULL` 且 `status = 'synced'` 的记录。返回每张表删除的条数。

#### 服务端真删除

管理员可通过管理后台或直接操作数据库清理已确认删除的记录。

#### 孤儿记录检查：`checkAndDeleteOrphans()`

用于修复因同步异常产生的孤立记录（如引用了不存在的父记录）。返回已软删除的孤儿记录列表和总检查数。

---

## 🔁 重试机制

- **失败记录**：不进行版本更新，保持 `status='pending'`
- **再握手**：下次握手时会重新检出这些 pending 记录
- **成功记录**：已更新的记录不会在下轮握手时被检出

---

## 🚀 服务器部署

### 前置要求

- Python ≥ 3.10
- pip

### 快速部署

```bash
cd server
pip install -r requirements.txt
python app.py
```

服务器默认运行在 `http://localhost:60032`。

### 环境变量配置

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `DB_TYPE` | 数据库类型 | `sqlite` |
| `DB_PATH` | SQLite 文件路径 | `./sync_data.db` |
| `DATABASE_URL` | 外部数据库 URL（pg/mysql） | — |
| `SECRET_KEY` | Flask 密钥 | 开发模式默认值 |

### 生产部署建议

**使用 PostgreSQL：**

```bash
DB_TYPE=postgresql \
DATABASE_URL=postgresql://user:password@localhost:5432/sync_db \
SECRET_KEY=$(openssl rand -hex 32) \
python app.py
```

**使用 Gunicorn：**

```bash
pip install gunicorn
gunicorn -w 4 -b 0.0.0.0:60032 app:app
```

**使用 Docker（参考）：**

```dockerfile
FROM python:3.11-slim
WORKDIR /app
COPY server/ .
RUN pip install -r requirements.txt
EXPOSE 60032
CMD ["gunicorn", "-w", "4", "-b", "0.0.0.0:60032", "app:app"]
```

### 安全建议

1. **修改 SECRET_KEY**：生产环境务必使用强随机密钥
2. **启用 HTTPS**：使用反向代理（Nginx/Caddy）配置 SSL
3. **限制访问**：使用防火墙限制 admin 管理页面的访问 IP
4. **定期备份**：定期备份数据库文件

---

## 📱 移动端同步注意事项

### 网络切换

Android 端在 Wi-Fi 与移动数据间切换时，同步连接会自动断开并重建。
建议在 Wi-Fi 环境下执行同步以避免流量消耗。

### 后台同步

当前版本同步在应用前台运行时触发。Android 端的后台定时同步（WorkManager）
计划在后续版本中加入。目前可通过以下方式手动同步：
1. 打开应用 → 进入 **同步** 页面 → 点击同步按钮
2. 每次打开应用时会自动检测 pending 记录并尝试同步

### 冲突处理

Android 端与桌面端同时编辑同一条记录时，冲突解决界面会弹出。
在手机屏幕上，冲突项使用 `ConflictResolver.vue` 的响应式布局，
两个版本上下排列展示，方便触屏操作。

### 服务器可达性

- Android 设备如果与服务器在同一局域网内，使用内网 IP（如 `http://192.168.1.100:60032`）
- 外网访问需要服务器具备公网 IP 或使用内网穿透工具（如 frp、ngrok）
- 同步超时默认为 15 秒，弱网环境下可能需要调整

---

## 📡 API 端点参考

> 所有端点均使用 **HTTP 明文传输**，生产环境建议通过反向代理配置 HTTPS。

| 方法 | 路径 | 说明 | 请求体 / 参数 |
|------|------|------|--------------|
| GET | `/health` | 健康检查 | — |
| POST | `/api/auth/validate` | 验证 auth_key 有效性 | `{ "auth_key": "xxx" }` |
| POST | `/api/auth/generate` | 生成新 auth_key（管理员，需 token） | `{ "remark": "可选备注" }` |
| **GET** | **`/api/sync/get_all_sync_data?auth_key=xxx`** | **拉取全量记录 header（握手输入）** | query param |
| **POST** | **`/api/sync/upload/<record_id>`** | **推送单条记录到服务端** | `{ auth_key, table_name, version, status, deleted_at, data }` |
| **GET** | **`/api/sync/download/<record_id>?auth_key=xxx`** | **从服务端拉取单条完整记录** | query param |
| GET | `/admin` | 管理后台（浏览器访问） | — |

### 握手阶段的数据流（客户端主导）

```
┌─────────────────────────────────────────────────────────────┐
│  客户端                             服务端                    │
│                                                             │
│  1. GET /api/sync/get_all_sync_data ──────────→              │
│      ?auth_key=xxx                                           │
│                                     ←─── 返回记录 header[]   │
│                                                             │
│  2. 执行 handshake(local_headers, remote_headers)           │
│     → push_list: string[]  (需要推送的记录 ID)               │
│     → pull_list: string[]  (需要拉取的记录 ID)               │
│     → conflicts: ConflictInfo[]  (需要用户处理的冲突)        │
│                                                             │
│  ┌─ 推送循环 ──────────────────────────────────────────┐    │
│  │ for each id in push_list:                            │    │
│  │   POST /api/sync/upload/<id> ──────────→             │    │
│  │     body: { auth_key, table_name, version, data }    │    │
│  │                       ←─── { success, new_version }  │    │
│  │   更新本地: status='synced', version=new_version     │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌─ 拉取循环 ──────────────────────────────────────────┐    │
│  │ for each id in pull_list:                            │    │
│  │   GET /api/sync/download/<id> ──────────→            │    │
│  │      ?auth_key=xxx                                   │    │
│  │                       ←─── { record: ServerRecord }  │    │
│  │   更新本地: status='synced', version=remote.version  │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                             │
│  3. 回到步骤 1 重新握手，直到 push_list + pull_list 均为空  │
└─────────────────────────────────────────────────────────────┘
```

### 关键接口详情

#### `GET /api/sync/get_all_sync_data`

拉取当前用户**所有记录的轻量 header**（不含 data 负载），作为客户端握手算法的远程侧输入。

**响应格式：**

```json
{
  "all_records": [
    {
      "id": "uuid-string",
      "table_name": "error_questions",
      "version": 5,
      "status": "synced",
      "deleted_at": null,
      "updated_at": 1700000000000
    }
  ]
}
```

> ⚠️ 注意：返回的是 `to_header()` 格式，**不含 `data` 字段**。前端 TypeScript 类型 `GetAllSyncDataResponse` 中的 `ServerRecord[]` 在运行时不含 data——拉取完整记录需要通过 `downloadRecord`。

#### `POST /api/sync/upload/<record_id>`

推送单条记录到服务端。服务端根据记录是否存在决定新建（version+1）或覆盖（保持当前 version）。

**请求体：**

```json
{
  "auth_key": "uuid-string",
  "table_name": "error_questions",
  "version": 5,
  "status": "synced",
  "deleted_at": null,
  "data": { "prompt": "...", "answer": "...", ... }
}
```

**响应：**

```json
{
  "success": true,
  "new_version": 6
}
```

#### `GET /api/sync/download/<record_id>`

拉取单条完整记录（含 data）。

**响应：**

```json
{
  "record": {
    "id": "uuid-string",
    "table_name": "error_questions",
    "version": 6,
    "status": "synced",
    "deleted_at": null,
    "updated_at": 1700000100000,
    "data": { "prompt": "...", "answer": "...", ... }
  }
}
```

---

> 同步协议相关问题请提交 [GitHub Issue](https://github.com/zpb911km/SmartErrorNotebook/issues)

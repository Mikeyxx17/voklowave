# voklowave

一个轻量级的实时团队聊天应用，类似 Slack / Discord。支持多频道、实时消息推送、用户认证、邮箱验证、密码重置、访客模式、会话管理、管理员后台、Markdown 渲染、消息搜索、表情回应、@提及通知和 IP 限流。

## 技术栈

**后端 (Rust)**
- [Axum 0.8](https://github.com/tokio-rs/axum) — Web 框架，内置 WebSocket 支持
- [SQLx 0.8](https://github.com/launchbadge/sqlx) — PostgreSQL 驱动，编译期 SQL 校验 + 自动迁移
- [jsonwebtoken](https://github.com/Keats/jsonwebtoken) — JWT 认证（HMAC 签名）
- [bcrypt](https://github.com/Keats/rust-bcrypt) — 密码哈希
- [lettre](https://github.com/lettre/lettre) — SMTP 邮件发送（验证码与密码重置邮件）
- [tokio](https://tokio.rs/) — 异步运行时，broadcast channel 实现消息广播
- [DashMap](https://github.com/xacrimon/dashmap) — 并发安全的频道广播注册表 + IP 限流存储
- [uuid](https://github.com/uuid-rs/uuid) — 访客标识 / JWT jti 生成
- [rand](https://github.com/rust-random/rand) — 验证码随机数生成
- [tower-http](https://github.com/tower-rs/tower-http) — CORS 中间件

**前端 (JavaScript)**
- [Vue 3](https://vuejs.org/) — Composition API + `<script setup>`
- [Vue Router 4](https://router.vuejs.org/) — Hash 路由
- [Vite 8](https://vitejs.dev/) — 构建工具
- [Tailwind CSS v4](https://tailwindcss.com/) — 原子化 CSS
- [daisyUI v5](https://daisyui.com/) — 组件库，内置 14 款主题
- [marked](https://marked.js.org/) — Markdown 解析
- [DOMPurify](https://github.com/cure53/DOMPurify) — HTML 防 XSS 清洗
- [KaTeX](https://katex.org/) — 数学公式渲染

## 功能特性

**认证与用户管理**
- 邮箱注册 + 6 位验证码激活，15 分钟有效，每日最多重发 3 次（60 秒冷却）
- 邮箱域名白名单（`.env` 可配置，留空不限制）
- 密码重置（通过邮箱验证码）
- 访客模式（一键体验，24 小时自动清理，仅限 `general` 频道）
- JWT 双 token 版本控制（修改密码后旧 token 全部失效）
- 会话管理（查看在线会话、远程踢出）
- 登录 IP 变更检测（新 IP 登录时提示）

**实时消息**
- WebSocket + tokio broadcast channel，消息即时送达
- 连接时回放最近 50 条历史消息 + 现有表情回应
- 墓碑同步：重连客户端自动移除已删除消息
- 硬删除消息（删除 + 墓碑表写入，事务保证）
- 30 秒心跳保活，断开后指数退避自动重连（1s → 2s → 4s → ... → 30s）
- 服务端覆盖消息中的 username 字段，防止身份伪造

**频道**
- 动态创建频道（重名返回 409）
- 访客仅能访问 `general` 频道

**Markdown & 搜索**
- Markdown 渲染：粗体、斜体、代码块、引用、链接、GFM
- KaTeX 数学公式：`$...$`（内联）和 `$$...$$`（块级）
- XSS 防护：DOMPurify 清洗
- 消息搜索：PostgreSQL pg_trgm 全文索引，支持全局 / 指定频道搜索，分页

**表情回应**
- 6 种预设表情：👍 ❤️ 😂 😮 😢 🙏
- 点击切换添加/移除，实时广播同步
- 回应计数 + "我"的状态高亮

**@提及**
- 输入 `@` 触发用户搜索自动补全（ILIKE 模糊匹配最多 10 条）
- 消息中 @ 提及高亮显示
- 被 @ 时桌面通知（浏览器 Notification API）

**用户资料**
- 编辑昵称（display_name）、头像 URL、个性签名（bio）
- 无头像时显示彩色哈希首字母头像

**UI / UX**
- 14 款 daisyUI 主题，localStorage 持久化
- 会话持久化（token、频道、资料存 sessionStorage，刷新自动恢复）
- 智能滚动：新消息自动滚底，查看历史时不打断
- 响应式布局：侧边栏抽屉式展开（移动端适配）
- 日期分隔符、消息时间戳悬浮显示

**管理员后台**
- 仪表盘：总用户/消息/频道/今日消息/访客数统计
- 用户管理：搜索、分页、删除用户、升降管理员
- 频道管理：消息数统计、删除频道
- 消息审计：全局搜索、强制删除
- 操作日志：管理员操作记录（含 IP 地址追溯）
- 实时刷新：通过 `/ws/admin` WebSocket 推送事件，所有管理页面无需手动刷新

**安全**
- 分层权限：AuthUser → AdminUser → SuperAdminUser
  - Admin：仪表盘、用户/频道列表、消息审计、操作日志、强制删除消息
  - SuperAdmin（额外）：删除用户、删除频道、升降管理员
- Admin 升级即时生效：中间件含 DB 回退查询，无需重登
- Admin 降级强制重登：token_version 递增，旧 JWT 失效
- 用户被删强制登出：全局事件通道推送到 WebSocket 客户端
- IP 限流：登录 10/分钟、注册 5/小时、重发验证码 5/小时、忘记密码 5/小时

## 项目结构

```
voklowave/
├── backend/                          # Rust 后端
│   ├── Cargo.toml
│   ├── .env                          # 数据库连接、JWT 密钥、SMTP 配置
│   ├── migrations/                   # SQLx 数据库迁移脚本（15 个）
│   └── src/
│       ├── main.rs                   # 入口：环境加载、数据库初始化、路由注册
│       ├── state.rs                  # AppState + ControlEvent 枚举
│       ├── middleware/
│       │   ├── mod.rs
│       │   ├── auth.rs               # JWT Claims、AuthUser 提取器
│       │   └── admin.rs              # AdminUser / SuperAdminUser 提取器
│       ├── services/
│       │   ├── mod.rs
│       │   ├── rate_limit.rs         # IP 滑动窗口限流器
│       │   └── cleanup.rs            # 后台清理：过期访客 + 墓碑
│       ├── handlers/
│       │   ├── mod.rs
│       │   ├── auth.rs               # 注册、登录、访客登录、邮箱验证、密码重置
│       │   ├── channels.rs           # 频道列表、创建频道
│       │   ├── messages.rs           # 消息删除、模糊搜索
│       │   ├── profile.rs            # 用户资料编辑、用户搜索（@ 提及补全）
│       │   ├── reactions.rs          # 表情回应切换
│       │   ├── sessions.rs           # 会话列表、踢出会话
│       │   ├── ws.rs                 # WebSocket：消息广播、心跳、墓碑同步、四路 select
│       │   └── admin.rs              # 管理后台：仪表盘、用户/频道/消息/日志管理
│       └── models/
│           ├── mod.rs
│           ├── user.rs               # 用户、认证请求/响应 DTO
│           ├── channel.rs            # 频道数据结构
│           └── message.rs            # 聊天消息数据结构
│
├── frontend/                         # Vue 3 前端
│   ├── package.json
│   ├── vite.config.js               # 开发服务器 :5173，代理 /api 和 /ws 到 :3000
│   └── src/
│       ├── main.js                   # createApp 入口
│       ├── App.vue                   # 根组件
│       ├── style.css                 # Tailwind + daisyUI + KaTeX + Markdown 样式
│       ├── router.js                 # Vue Router（hash 模式，含 admin 路由守卫）
│       ├── config.js                 # 前端配置常量
│       ├── composables/
│       │   ├── useAppState.js        # 全局状态：认证、主题、资料、通知权限
│       │   ├── useChannels.js        # 频道列表、创建频道
│       │   ├── useWebSocket.js       # WebSocket 连接、消息收发、删除/反应/通知处理
│       │   ├── useMarkdown.js        # Markdown + KaTeX 渲染
│       │   ├── useAdmin.js           # 管理后台 API 调用
│       │   └── useAdminEvents.js     # 管理后台 WebSocket 实时事件
│       └── components/
│           ├── LoginView.vue         # 登录/注册/快速体验/邮箱验证/密码重置
│           ├── ChatLayout.vue        # 聊天主布局
│           ├── ChatHeader.vue        # 频道标题 + 消息搜索（可拖拽面板）
│           ├── MessageList.vue       # 消息列表（智能滚动、搜索跳转）
│           ├── MessageBubble.vue     # 单条消息气泡（Markdown、@ 高亮、表情回应、删除）
│           ├── MessageInput.vue      # 消息输入框（@ 自动补全）
│           ├── Sidebar.vue           # 侧边栏：频道列表、主题切换、用户信息
│           ├── CreateChannelModal.vue # 创建频道弹窗
│           ├── ProfileEditModal.vue  # 个人资料编辑弹窗
│           ├── SessionListModal.vue  # 会话管理弹窗
│           └── admin/
│               ├── AdminLayout.vue   # 管理后台布局
│               ├── DashboardView.vue # 仪表盘
│               ├── UsersView.vue     # 用户管理
│               ├── ChannelsView.vue  # 频道管理
│               ├── MessagesAuditView.vue # 消息审计
│               ├── AuditLogsView.vue # 操作日志
│               └── SettingsView.vue  # 设置
│
└── README.md
```

## 数据库迁移

共 15 个迁移脚本，启动时由 SQLx 自动按序执行。

| 时间戳 | 说明 |
|--------|------|
| `20260419051354` | 创建 `messages` 表（id, channel, username, content, created_at） |
| `20260421065815` | 创建 `channels` 表（id, name, created_at） |
| `20260527142314` | 创建 `users` 表（id, username, email, password_hash, display_name, avatar_url, bio） |
| `20260604025405` | 添加 `is_guest` 字段 |
| `20260606073236` | 添加邮箱验证字段（is_verified, token, expires_at） |
| `20260606210000` | 添加重发计数字段（resend_count, last_resend_at） |
| `20260608000000` | 提取 `verification_codes` 独立表 |
| `20260613000000` | 创建 `deleted_messages` 墓碑表 |
| `20260613000001` | 消息搜索索引（pg_trgm + GIN） |
| `20260614000000` | 创建 `message_reactions` 表情回应表 |
| `20260615170000` | 添加 `token_version` 字段（密码重置时递增，旧 token 失效） |
| `20260615180000` | 创建 `sessions` 会话管理表 |
| `20260617000000` | `deleted_messages` 添加 UNIQUE 约束 |
| `20260618000000` | 添加 `is_admin` 字段 + `admin_audit_logs` 表 |
| `20260618000001` | 添加 `is_superadmin` 字段（替代硬编码 SuperAdmin） |

## API 端点

### 公开端点

| 方法 | 路径 | 说明 | 限流 |
|------|------|------|------|
| POST | `/api/register` | 注册账号，发送验证邮件 | 5/小时（每 IP） |
| POST | `/api/login` | 邮箱密码登录，返回 JWT | 10/分钟（每 IP） |
| POST | `/api/guest_login` | 创建 24h 临时访客账户，返回 JWT | 无 |
| POST | `/api/verify_email` | 提交 6 位验证码激活账号 | 无 |
| POST | `/api/resend_verification` | 重新发送验证码 | 5/小时（每 IP） |
| POST | `/api/forgot_password` | 发送密码重置验证码 | 5/小时（每 IP） |
| POST | `/api/reset_password` | 提交验证码 + 新密码完成重置 | 无 |

### 认证端点（JWT Bearer）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/me` | 获取当前用户信息（含 is_admin、is_superadmin） |
| PATCH | `/api/me` | 更新个人资料（display_name / avatar_url / bio，按需更新） |
| GET | `/api/users` | 搜索用户（@ 提及补全，`?q=` ILIKE 模糊匹配，最多 10 条） |
| GET | `/api/channels` | 频道列表（访客仅见 `general`） |
| POST | `/api/channels` | 创建频道（访客禁止，重名返回 409） |
| DELETE | `/api/messages/{id}` | 删除自己的消息 + 墓碑写入 + 广播 |
| GET | `/api/messages/search` | 模糊搜索历史消息（`?q=&channel=&page=&limit=`） |
| POST | `/api/messages/{id}/react` | 切换表情回应（toggle） |
| GET | `/api/sessions` | 当前用户活跃会话列表 |
| DELETE | `/api/sessions/{id}` | 踢出指定会话（不可踢当前会话） |

### WebSocket

| 路径 | 说明 | 认证 |
|------|------|------|
| `/ws/{channel}` | 聊天实时消息 | `?token=` URL 参数 |
| `/ws/admin` | 管理后台实时事件推送 | `?token=` URL 参数（需 Admin） |

### 管理后台端点（JWT Bearer + Admin）

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/api/admin/dashboard` | 统计数据（用户/消息/频道/今日消息/访客数） | Admin |
| GET | `/api/admin/users` | 用户列表（搜索 + 分页） | Admin |
| DELETE | `/api/admin/users/{id}` | 删除用户及关联数据 | SuperAdmin |
| PATCH | `/api/admin/users/{id}/toggle-admin` | 切换管理员身份 | SuperAdmin |
| GET | `/api/admin/channels` | 频道列表（含消息数） | Admin |
| DELETE | `/api/admin/channels/{id}` | 删除频道及所有消息 | SuperAdmin |
| GET | `/api/admin/messages` | 全局消息搜索 | Admin |
| DELETE | `/api/admin/messages/{id}` | 强制删除任意消息 | Admin |
| GET | `/api/admin/audit-logs` | 操作日志（分页） | Admin |

## WebSocket 协议

### 聊天 WebSocket — `/ws/{channel}?token=<jwt>`

**连接流程**

1. 服务端解析 URL 参数中的 JWT，校验有效性、token_version、会话状态
2. 访客仅允许连接 `general` 频道；非访客校验频道存在性
3. 服务端下发墓碑列表（最近 1 小时删除的消息 ID），客户端移除对应消息
4. 服务端回放该频道最近 50 条历史消息
5. 服务端回放该频道现有表情回应（最多 300 条）

**消息格式**

客户端发送聊天消息：
```json
{"channel": "general", "username": "（可任意填）", "content": "Hello World"}
```
> username 会被服务端替换为 JWT 中的真实身份（防伪造）；服务端还会自动附加 display_name 和 avatar_url。

客户端心跳（每 30 秒）：
```json
{"type": "ping"}
```
服务端回复：
```json
{"type": "pong"}
```

**服务端推送的控制事件**

```json
// 消息被删除
{"type": "message_deleted", "message_id": 42}

// 表情回应切换
{"type": "reaction_toggled", "message_id": 42, "emoji": "👍", "username": "mikey", "action": "added"}

// 账号被管理员删除（强制登出）
{"type": "user_deleted", "user_id": 7}
```

### 管理后台 WebSocket — `/ws/admin?token=<jwt>`

**连接条件**：JWT 中 `is_admin` 为 true 或数据库 `is_admin` 为 true。

**推送的事件类型**

| 事件 | 携带数据 | 触发时机 |
|------|---------|---------|
| `message_created` | `message_id`, `channel`, `username` | 用户发送新消息 |
| `message_deleted` | `message_id` | 管理员或用户删除消息 |
| `channel_created` | `name` | 创建新频道 |
| `channel_deleted` | `name` | 管理员删除频道 |
| `user_created` | `username` | 注册 / 访客登录 |
| `user_deleted` | `user_id` | 管理员删除用户 |
| `user_admin_toggled` | `user_id` | 升降管理员身份 |

前端管理页面通过监听这些事件自动刷新数据，无需手动刷新。

## 快速开始

### 前置条件

- [Rust](https://www.rust-lang.org/) (stable，建议通过 [rustup](https://rustup.rs/) 安装)
- [Node.js](https://nodejs.org/) 22+ 和 npm
- [PostgreSQL](https://www.postgresql.org/) 16+ 运行中的实例，并安装 `pg_trgm` 扩展
- 一个 SMTP 邮箱账号（用于发送验证码，推荐 QQ 邮箱或 Gmail）

### 1. 创建 PostgreSQL 数据库

```sql
CREATE DATABASE voklowave;
-- 可选：创建专用用户
CREATE USER voklowave_user WITH PASSWORD '你的密码';
GRANT ALL PRIVILEGES ON DATABASE voklowave TO voklowave_user;
```

> 确保数据库启用了 `pg_trgm` 扩展。SQLx 迁移会自动处理字符集和索引，无需手动操作。

### 2. 配置并启动后端

```bash
cd backend

# 创建 .env 配置文件（参考下方说明填写各项）
cp .env.example .env    # 如果没有 example 文件则手动创建

# 编译并启动（首次编译需下载依赖，约 2-5 分钟）
cargo run
```

**.env 配置说明**

```env
# ── 必填 ──
DATABASE_URL=postgres://用户名:密码@主机:端口/数据库名
JWT_SECRET=至少 32 字符的随机密钥（用于 JWT 签名）

# ── 邮件发送（必填，否则注册验证码无法发送） ──
SMTP_SERVER=smtp.qq.com             # SMTP 服务器地址
SMTP_USERNAME=你的邮箱@qq.com         # 发件邮箱
SMTP_PASSWORD=你的SMTP授权码           # QQ邮箱需开启SMTP后获取授权码

# ── 可选 ──
ALLOWED_DOMAINS=qq.com,gmail.com     # 允许注册的邮箱域名（逗号分隔，留空不限制）
CLEANUP_INTERVAL_SECS=1800           # 访客清理任务间隔（秒），默认 1800（30 分钟）
GUEST_MAX_AGE_HOURS=24               # 访客账户过期时间（小时），默认 24
```

> **JWT_SECRET**：可以用 `openssl rand -base64 64` 生成一个安全的随机密钥。

> **SMTP 密码**：QQ 邮箱需要在「设置 → 账户 → POP3/SMTP 服务」中开启并获取授权码，不能使用登录密码。Gmail 需使用 App Password。

### 3. 后端启动流程详解

当你执行 `cargo run` 后，后端按以下顺序初始化：

**阶段 I：环境加载 & 日志初始化**

```
tracing_subscriber::fmt::init()
dotenv()
```
- 初始化结构化日志（tracing），后续所有 `info!` / `warn!` / `error!` 都会输出到控制台
- 从 `.env` 文件加载环境变量到 `std::env`

**阶段 II：配置校验**

- **强制校验 `JWT_SECRET`**：如果未设置，进程立即 panic 退出，防止使用硬编码 fallback 密钥导致安全漏洞
- 读取 `DATABASE_URL`：未设置则 panic

**阶段 III：数据库连接 & 迁移**

```
PgPoolOptions::new().max_connections(5).connect(&DATABASE_URL)
sqlx::migrate!("./migrations").run(&pool)
```
- 创建连接池（最大 5 个连接）
- 连接 PostgreSQL 验证连通性
- **自动执行目录 `migrations/` 下的所有 SQL 迁移脚本**（共 15 个）。SQLx 内部维护 `_sqlx_migrations` 表追踪已执行的迁移，已执行过的不会重复执行。首次启动时建表 + 建索引，后续启动只检查不执行

**阶段 IV：应用状态初始化**

```
AppState {
    db: PgPool,                                          // 数据库连接池
    channels: Arc<DashMap<String, Sender>>,               // 频道 → 消息广播通道
    control_channels: Arc<DashMap<String, Sender>>,       // 频道 → 控制事件广播通道
    admin_events: broadcast::Sender,                       // 全局管理后台事件通道（容量 256）
    global_events: broadcast::Sender,                      // 全局用户事件通道（容量 256）
    login_limiter, register_limiter, ...                   // 4 个 IP 限流器
}
```

- 创建 4 个内存 `DashMap`（频道注册表 + 3 个控制通道）
- 创建 2 个全局 `broadcast::channel`（容量 256，满了时旧消息被丢弃）
- 初始化 4 个 IP 限流器（登录/注册/重发/忘记密码），底层是带清理的 `DashMap<IpAddr, Vec<Instant>>`

**阶段 V：加载已有频道**

```sql
SELECT name FROM channels
```

- 从数据库读取所有频道名称
- 为每个频道创建 `broadcast::channel`（消息通道 + 控制事件通道，容量各 100）
- 写入 `AppState.channels` 和 `AppState.control_channels`
- 日志输出：`已成功加载 N 个频道`

**阶段 VI：启动后台任务**

```rust
tokio::spawn(spawn_cleanup_task(pool, control_channels, interval, max_age))
```

- 异步启动**访客清理 + 墓碑清理**定时任务（不阻塞主线程）
- 每 `CLEANUP_INTERVAL_SECS` 秒执行一次
- 清理逻辑：查找过期访客 → 事务删除消息 + 用户 + 验证码 + 会话 → 广播 `MessageDeleted` 事件 → 清理 1 小时前的墓碑记录

**阶段 VII：注册路由 & 启动 HTTP 服务器**

```rust
Router::new()
    .route("/ws/{channel}", get(ws_handler))         // 聊天 WebSocket
    .route("/ws/admin", get(admin_ws_handler))       // 管理后台 WebSocket
    .route("/api/login", post(login))                // 登录
    .route("/api/register", post(register))          // 注册
    // ... 共 25 个路由
    .layer(cors)                                     // 允许跨域
    .with_state(state)                               // 注入共享状态

axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
```

- 注册 25 个 HTTP 路由 + 2 个 WebSocket 路由
- 添加 CORS 中间件（允许所有来源，开发模式）
- `into_make_service_with_connect_info` 关键作用：使每个请求能获取客户端 SocketAddr，用于 IP 限流和审计日志中的 IP 追溯
- 绑定 `0.0.0.0:3000` 开始监听
- 日志输出：`后端引擎已就绪：http://0.0.0.0:3000`

**初始化流程图**

```
.env 加载 → JWT_SECRET 校验 → PgPool 连接 → 迁移执行(15 SQL)
    → AppState 构建 (4 DashMap + 2 Broadcast Channel + 4 RateLimiter)
    → 从 DB 加载频道 → 创建频道广播通道 → 注入 DashMap
    → 启动后台清理任务 (tokio::spawn)
    → 注册路由 → CORS 中间件 → 绑定 :3000 → 开始监听
```

> **注意**：后端进程必须保持运行。关闭终端或 Ctrl+C 会停止服务。生产环境建议配合 systemd 或 Docker 守护进程。

### 4. 配置并启动前端

```bash
cd frontend

# 安装依赖
npm install

# 启动开发服务器
npm run dev
```

前端开发服务器运行在 `http://localhost:5173`。

**代理配置**（`vite.config.js`）：前端请求 `/api/*` 和 `/ws/*` 会自动代理到后端 `http://127.0.0.1:3000`，WebSocket 代理也已启用。无需在前端配置后端地址。

### 5. 首次使用

1. 浏览器打开 `http://localhost:5173`
2. 注册一个账号（需要能收到验证码的邮箱），提交验证码激活
3. 第一个注册的用户默认**不是**管理员。需要手动在数据库中设置 SuperAdmin：

```sql
UPDATE users SET is_admin = true, is_superadmin = true WHERE email = '你的邮箱';
```

4. 设置完成后该用户即可访问管理后台 `/#/admin`

### 6. 生产部署注意事项

- 修改 `JWT_SECRET` 为生产环境唯一密钥
- 前端 `vite.config.js` 中的代理仅用于开发。生产环境需由 nginx / Caddy 反向代理，或由前端构建产物直接指向后端地址
- 前端生产构建：`cd frontend && npm run build`，产物在 `dist/` 目录
- 设置 `ALLOWED_DOMAINS` 限制注册邮箱
- 建议开启 PostgreSQL SSL 连接（`DATABASE_URL` 添加 `?sslmode=require`）
- 定期备份数据库（`pg_dump`）

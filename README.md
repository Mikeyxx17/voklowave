# voklowave

一个轻量级的实时团队聊天应用，类似 Slack / Discord。支持多频道、实时消息推送、用户认证、邮箱验证、密码重置、访客模式、Markdown 渲染、消息搜索、表情回应、@提及通知和 IP 限流。

## 技术栈

**后端 (Rust)**
- [Axum 0.8](https://github.com/tokio-rs/axum) — Web 框架，内置 WebSocket 支持
- [SQLx 0.8](https://github.com/launchbadge/sqlx) — PostgreSQL 驱动，编译期 SQL 校验 + 自动迁移
- [jsonwebtoken](https://github.com/Keats/jsonwebtoken) — JWT 认证（HMAC 签名）
- [bcrypt](https://github.com/Keats/rust-bcrypt) — 密码哈希
- [lettre](https://github.com/lettre/lettre) — SMTP 邮件发送（验证码与密码重置邮件）
- [tokio](https://tokio.rs/) — 异步运行时，broadcast channel 实现消息广播
- [DashMap](https://github.com/xacrimon/dashmap) — 并发安全的频道广播注册表 + IP 限流存储
- [uuid](https://github.com/uuid-rs/uuid) — 访客标识生成
- [rand](https://github.com/rust-random/rand) — 验证码随机数生成
- [tower-http](https://github.com/tower-rs/tower-http) — CORS 中间件

**前端 (JavaScript)**
- [Vue 3](https://vuejs.org/) — Composition API + `<script setup>`
- [Vite 8](https://vitejs.dev/) — 构建工具
- [Tailwind CSS v4](https://tailwindcss.com/) — 原子化 CSS
- [daisyUI v5](https://daisyui.com/) — 组件库，内置多款主题
- [marked](https://marked.js.org/) — Markdown 解析
- [DOMPurify](https://github.com/cure53/DOMPurify) — HTML 防 XSS 清洗
- [KaTeX](https://katex.org/) — 数学公式渲染

## 功能特性

- **实时消息** — WebSocket + tokio broadcast channel，消息即时送达
- **JWT 认证** — 注册 / 登录，token 有效期 7 天，WebSocket 通过 URL 参数传递 token
- **邮箱验证** — 注册后发送 6 位验证码到邮箱，15 分钟有效，每日最多重发 3 次、60 秒冷却
- **密码重置** — 忘记密码时通过邮箱验证码重置，与注册验证码隔离存储
- **域名白名单** — 可配置允许注册的邮箱域名，防止滥用
- **多频道** — 创建和切换文字频道，注册用户可创建新频道（重名检测）
- **消息历史** — 每次连接自动回放最近 50 条消息
- **自动重连** — WebSocket 断开后指数退避自动重连，30 秒心跳保活
- **访客模式** — 一键匿名体验，仅限 `general` 频道，24 小时自动清理
- **访客清理** — 后台定时任务删除过期访客账号及消息，事务保证原子性，实时广播同步
- **会话持久化** — token 和当前频道保存在 sessionStorage，主题保存在 localStorage，刷新页面自动恢复
- **14 款主题** — dark、light、cyberpunk、cupcake、synthwave、nord、sunset、winter、coffee、lemonade、luxury、business、autumn、dim
- **智能滚动** — 新消息自动滚到底部，向上查看历史时不打断
- **移动端适配** — 响应式布局，侧边栏抽屉式展开
- **用户资料** — 编辑昵称、头像、个性签名
- **消息删除** — 硬删除 + 墓碑表方案，实时广播同步
- **Markdown 渲染** — 支持粗体、斜体、代码块、引用、链接、数学公式（KaTeX）
- **消息搜索** — PostgreSQL pg_trgm 全文索引，支持全局/指定频道搜索，关键词高亮
- **表情回应** — 👍 ❤️ 😂 😮 😢 🙏，实时切换 + 广播
- **@提及** — 输入 `@` 自动补全用户名，消息中高亮显示，被 @ 时桌面通知
- **IP 限流** — 登录/注册/重发验证码接口 IP 级别速率限制

## 项目结构

```
voklowave/
├── backend/                      # Rust 后端
│   ├── Cargo.toml
│   ├── .env                     # 数据库连接、JWT 密钥、SMTP 配置、清理任务配置
│   ├── migrations/              # SQLx 数据库迁移脚本（10 个）
│   └── src/
│       ├── main.rs              # 入口：环境加载、数据库初始化、路由注册、启动监听 :3000
│       ├── state.rs             # AppState：PgPool + DashMap + ControlEvent + RateLimiter
│       ├── middleware/
│       │   ├── mod.rs
│       │   └── auth.rs          # JWT Claims、AuthUser 提取器（访客额外 DB 存活检查）
│       ├── services/
│       │   ├── mod.rs
│       │   ├── cleanup.rs       # 后台清理任务：访客清理 + 墓碑清理 + 删除广播
│       │   └── rate_limit.rs    # IP 限流器：滑动窗口算法
│       ├── handlers/
│       │   ├── mod.rs
│       │   ├── auth.rs          # 注册、登录、访客登录、邮箱验证、密码重置（限流）
│       │   ├── channels.rs      # 频道列表、创建频道（重名检测 + 访客权限控制）
│       │   ├── messages.rs      # 消息硬删除（墓碑表） + 模糊搜索（pg_trgm）
│       │   ├── profile.rs       # 用户资料编辑 + 用户搜索（@ 提及补全）
│       │   ├── reactions.rs     # 表情回应切换
│       │   └── ws.rs            # WebSocket：JWT 校验、消息广播、心跳、墓碑同步、三路 select
│       └── models/
│           ├── mod.rs
│           ├── user.rs          # 用户、认证请求/响应 DTO
│           ├── channel.rs       # 频道数据结构
│           └── message.rs       # 聊天消息数据结构
│
├── frontend/                     # Vue 3 前端
│   ├── package.json
│   ├── vite.config.js           # 开发服务器 :5173，代理 /api 和 /ws 到 :3000
│   └── src/
│       ├── main.js              # createApp 入口
│       ├── App.vue              # 根组件：登录 / 聊天布局切换
│       ├── style.css            # Tailwind + daisyUI + KaTeX + Markdown 样式
│       ├── composables/
│       │   ├── useAppState.js   # 全局状态：认证、主题、会话持久化、用户资料、通知权限
│       │   ├── useChannels.js   # 频道列表、创建频道
│       │   ├── useMarkdown.js   # Markdown + KaTeX 渲染
│       │   └── useWebSocket.js  # WebSocket 连接、消息收发、自动重连、心跳、删除/反应/通知处理
│       └── components/
│           ├── ChatLayout.vue    # 聊天主布局
│           ├── ChatHeader.vue    # 频道标题 + 消息搜索（可拖拽面板）
│           ├── MessageList.vue   # 消息列表（智能滚动、搜索跳转）
│           ├── MessageBubble.vue # 单条消息气泡（Markdown、@ 高亮、表情回应、删除）
│           ├── MessageInput.vue  # 消息输入框（@ 自动补全）
│           ├── Sidebar.vue       # 侧边栏：频道列表、主题切换、用户信息、资料编辑入口
│           ├── LoginView.vue     # 登录/注册/快速体验/邮箱验证/密码重置
│           ├── CreateChannelModal.vue # 创建频道弹窗
│           └── ProfileEditModal.vue   # 个人资料编辑弹窗
│
└── README.md
```

## API 端点

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/ws/{channel}` | WebSocket 实时消息 | URL 参数 `?token=` |
| GET | `/api/channels` | 获取频道列表（访客仅见 general） | JWT |
| POST | `/api/channels` | 创建频道（访客禁止，重名返回 409） | JWT |
| POST | `/api/register` | 注册账号，发送验证邮件（每 IP 每小时 5 次） | 否 |
| POST | `/api/login` | 登录，返回 JWT（每 IP 每分钟 10 次） | 否 |
| POST | `/api/guest_login` | 创建临时访客账户，返回 JWT（1 天有效） | 否 |
| POST | `/api/verify_email` | 提交 6 位验证码激活账号 | 否 |
| POST | `/api/resend_verification` | 重新发送验证码（每 IP 每小时 5 次） | 否 |
| POST | `/api/forgot_password` | 发送密码重置验证码到注册邮箱 | 否 |
| POST | `/api/reset_password` | 提交验证码 + 新密码完成密码重置 | 否 |
| GET | `/api/users` | 搜索用户（@ 提及补全，ILIKE 模糊匹配） | JWT |
| GET | `/api/me` | 获取当前用户信息 | JWT |
| PATCH | `/api/me` | 更新当前用户资料（昵称/头像/签名） | JWT |
| DELETE | `/api/messages/{id}` | 硬删除自己的消息 + 墓碑表 + 广播 | JWT |
| GET | `/api/messages/search` | 模糊搜索历史消息（q/通道/分页） | JWT |
| POST | `/api/messages/{id}/react` | 切换表情回应 | JWT |

### WebSocket 协议

连接 `/ws/{channel}?token=<jwt>` 后：

1. 服务端校验 URL 参数中的 JWT，访客仅允许连接 `general` 频道
2. 服务端下发墓碑列表（最近 1 小时删除的消息 ID），客户端移除对应消息
3. 服务端回放该频道最近 50 条历史消息
4. 客户端发送消息：`{"channel": "name", "username": "user", "content": "text"}`（username 会被服务端替换为 JWT 中的身份，防止伪造）
5. 服务端存入 PostgreSQL 后广播给频道内所有在线客户端（带上昵称和头像）
6. 客户端每 30 秒发送心跳：`{"type": "ping"}`，服务端回复 `{"type": "pong"}`
7. 服务端广播控制事件：
   - `{"type": "message_deleted", "message_id": 42}` — 消息被删除
   - `{"type": "reaction_toggled", "message_id": 42, "emoji": "👍", "username": "mikey", "action": "added|removed"}` — 表情回应切换

## 数据库迁移

| 时间戳 | 说明 |
|--------|------|
| `20260419051354` | 创建 `messages` 表 |
| `20260421065815` | 创建 `channels` 表 |
| `20260527142314` | 创建 `users` 表（含 display_name/avatar_url/bio） |
| `20260604025405` | 添加 `is_guest` 字段 |
| `20260606073236` | 添加邮箱验证字段 |
| `20260606210000` | 添加重发计数字段 |
| `20260608000000` | 提取 `verification_codes` 独立表 |
| `20260613000000` | 创建 `deleted_messages` 墓碑表 |
| `20260613000001` | 消息搜索索引（pg_trgm + GIN） |
| `20260614000000` | 创建 `message_reactions` 表情回应表 |

## 快速开始

### 前置条件

- [Rust](https://www.rust-lang.org/) (stable)
- [Node.js](https://nodejs.org/) 和 npm
- [PostgreSQL](https://www.postgresql.org/) 运行中的实例

### 1. 启动后端

```bash
cd backend

# 创建 .env 文件
cat > .env << EOF
DATABASE_URL=postgres://用户名:密码@localhost/voklowave
JWT_SECRET=你的密钥
SMTP_SERVER=smtp.qq.com
SMTP_USERNAME=你的邮箱@qq.com
SMTP_PASSWORD=你的邮箱密码或应用密码
ALLOWED_DOMAINS=gmail.com,outlook.com,qq.com
CLEANUP_INTERVAL_SECS=1800
GUEST_MAX_AGE_HOURS=24
EOF

cargo run
```

后端启动在 `http://0.0.0.0:3000`，首次启动时 SQLx 迁移会自动建表。

### 2. 启动前端

```bash
cd frontend
npm install
npm run dev
```

前端开发服务器运行在 `http://localhost:5173`，自动代理 `/api` 和 `/ws` 到后端。

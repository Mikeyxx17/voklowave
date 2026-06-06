# voklowave

一个轻量级的实时团队聊天应用，类似 Slack / Discord。支持多频道、实时消息推送、用户认证和访客模式。

## 技术栈

**后端 (Rust)**
- [Axum 0.8](https://github.com/tokio-rs/axum) — Web 框架，内置 WebSocket 支持
- [SQLx 0.8](https://github.com/launchbadge/sqlx) — PostgreSQL 驱动，编译期 SQL 校验 + 自动迁移
- [jsonwebtoken](https://github.com/Keats/jsonwebtoken) — JWT 认证（HMAC 签名）
- [bcrypt](https://github.com/Keats/rust-bcrypt) — 密码哈希
- [tokio](https://tokio.rs/) — 异步运行时，broadcast channel 实现消息广播
- [DashMap](https://github.com/xacrimon/dashmap) — 并发安全的 channel 广播注册表

**前端 (JavaScript)**
- [Vue 3](https://vuejs.org/) — Composition API + `<script setup>`
- [Vite 8](https://vitejs.dev/) — 构建工具
- [Tailwind CSS v4](https://tailwindcss.com/) — 原子化 CSS
- [daisyUI v5](https://daisyui.com/) — 组件库，内置多款主题

## 功能特性

- **实时消息** — 基于 WebSocket + tokio broadcast channel，消息即时送达
- **自动重连** — WebSocket 断开后指数退避自动重连，30 秒心跳保活
- **多频道** — 创建和切换文字频道，注册用户可创建新频道
- **消息历史** — 每次连接自动回放最近 50 条消息
- **JWT 认证** — 注册 / 登录，token 有效期 7 天
- **访客模式** — 一键匿名体验，仅限 `general` 频道，到期自动清理
- **访客清理** — 后台定时任务自动删除过期访客账号及消息，事务保证原子性
- **访客提醒** — 聊天界面顶部横幅 + 登录页面提示，告知访客数据将被自动清除
- **14 款主题** — dark、light、cyberpunk、cupcake、synthwave、nord、sunset、winter、coffee、lemonade、luxury、business、autumn、dim
- **会话持久化** — token 和当前频道保存在 sessionStorage，主题保存在 localStorage，刷新页面自动恢复
- **智能滚动** — 新消息自动滚到底部，向上滚动查看历史时不打断
- **移动端适配** — 响应式布局，侧边栏抽屉式展开

## 项目结构

```
voklowave/
├── backend/                      # Rust 后端
│   ├── Cargo.toml
│   ├── .env                      # 数据库连接、JWT 密钥、清理任务配置
│   ├── migrations/               # SQLx 数据库迁移脚本
│   └── src/
│       ├── main.rs               # 入口：路由、CORS、启动清理任务、绑定 :3000
│       ├── state.rs              # AppState：PgPool + DashMap<broadcast::Sender>
│       ├── middleware/
│       │   └── auth.rs           # JWT Claims、AuthUser 提取器（含数据库查活）
│       ├── services/
│       │   └── cleanup.rs        # 后台访客清理任务（定时删除过期账号及消息）
│       ├── handlers/
│       │   ├── auth.rs           # 注册、登录、访客登录、获取当前用户
│       │   ├── channels.rs       # 频道列表、创建频道（含访客权限控制）
│       │   └── ws.rs             # WebSocket 连接管理、消息广播、心跳处理
│       └── models/
│           ├── user.rs           # 用户、认证请求/响应
│           ├── channel.rs        # 频道数据结构
│           └── message.rs        # 聊天消息数据结构
│
├── frontend/                     # Vue 3 前端
│   ├── package.json
│   ├── vite.config.js            # 开发服务器 :5173，代理 /api 和 /ws 到 :3000
│   └── src/
│       ├── main.js               # createApp 入口
│       ├── App.vue               # 根组件：登录 / 聊天布局切换
│       ├── style.css             # Tailwind + daisyUI + 自定义样式
│       ├── composables/
│       │   ├── useAppState.js    # 全局状态：认证、主题、会话持久化
│       │   ├── useChannels.js    # 频道列表、创建频道
│       │   └── useWebSocket.js   # WebSocket 连接、消息收发、自动重连、心跳
│       └── components/
│           ├── ChatLayout.vue     # 聊天主布局（含访客提醒横幅）
│           ├── ChatHeader.vue     # 频道标题
│           ├── MessageList.vue    # 消息列表（智能滚动）
│           ├── MessageBubble.vue  # 单条消息气泡
│           ├── MessageInput.vue   # 消息输入框
│           ├── Sidebar.vue        # 侧边栏：频道列表、主题切换、用户信息
│           ├── LoginView.vue      # 登录/注册/快速体验（含访客提醒）
│           └── CreateChannelModal.vue  # 创建频道弹窗
```

## 数据库迁移

项目使用 [SQLx](https://github.com/launchbadge/sqlx) 管理数据库迁移，迁移脚本位于 `backend/migrations/`，应用启动时自动执行。

### 安装 SQLx CLI

```bash
cargo install sqlx-cli --version 0.8.6
```

> 版本需与 `Cargo.toml` 中 sqlx 的版本一致。

### 常用命令

```bash
# 创建新的迁移脚本
sqlx migrate add <名称>

# 执行所有未执行的迁移
sqlx migrate run

# 回滚最近一次迁移
sqlx migrate revert

# 查看迁移状态
sqlx migrate info

# 使用指定数据库 URL 执行迁移
DATABASE_URL="postgres://用户名:密码@localhost/voklowave" sqlx migrate run
```

## API 端点

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/ws/{channel}` | WebSocket 实时消息（含心跳） | 否 |
| GET | `/api/channels` | 获取频道列表（访客仅见 general） | JWT |
| POST | `/api/channels` | 创建频道（访客禁止） | JWT |
| POST | `/api/register` | 注册账号 | 否 |
| POST | `/api/login` | 登录，返回 JWT（7 天有效） | 否 |
| POST | `/api/guest_login` | 创建临时访客账户，返回 JWT（1 天有效） | 否 |
| GET | `/api/me` | 获取当前用户信息 | JWT |

### WebSocket 协议

连接 `/ws/{channel}` 后：
1. 服务端立即回放该频道最近 50 条历史消息
2. 客户端发送聊天消息 JSON：`{"channel": "name", "username": "user", "content": "text"}`
3. 服务端存入 PostgreSQL 后广播给频道内所有在线客户端
4. 客户端每 30 秒发送心跳：`{"type": "ping"}`，服务端回复 `{"type": "pong"}`

## 快速开始

### 前置条件

- [Rust](https://www.rust-lang.org/) (stable)
- [Node.js](https://nodejs.org/) 和 npm
- [PostgreSQL](https://www.postgresql.org/) 运行中的实例

### 1. 启动后端

```bash
cd backend

# 创建 .env 文件，配置数据库连接和 JWT 密钥
cat > .env << EOF
DATABASE_URL=postgres://用户名:密码@localhost/voklowave
JWT_SECRET=你的密钥
CLEANUP_INTERVAL_SECS=1800
GUEST_MAX_AGE_HOURS=24
EOF

# 首次运行前手动创建数据库（迁移会自动执行）
# createdb voklowave

cargo run
```

后端启动在 `http://0.0.0.0:3000`，首次启动时 SQLx 迁移会自动建表。

### 2. 启动前端

```bash
cd frontend

npm install
npm run dev
```

前端开发服务器启动在 `http://localhost:5173`，Vite 自动将 `/api` 和 `/ws` 请求代理到后端。

### 3. 生产构建

```bash
cd frontend
npm run build
```

构建产物输出到 `frontend/dist/`，可直接部署到静态文件服务器。

## 环境变量

| 变量 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `DATABASE_URL` | 是 | — | PostgreSQL 连接字符串 |
| `JWT_SECRET` | 否 | 开发用硬编码值 | JWT 签名密钥 |
| `CLEANUP_INTERVAL_SECS` | 否 | 1800 | 访客清理任务执行间隔（秒） |
| `GUEST_MAX_AGE_HOURS` | 否 | 24 | 访客账号最大存活时间（小时） |

## License

MIT

import { ref, watch } from 'vue'
import { useAppState } from './useAppState'
import { useChannels } from './useChannels'

// ── 模块级全局单例 ──
const messages = ref([])
const connected = ref(false)
const scrollToId = ref(null)  // 新增：搜索跳转目标消息 ID
const reactions = ref({})     // 新增：{ message_id: { "👍": { count: 3, me: true }, ... } }
let socket = null
let retryCount = 0
let retryTimer = null
let heartbeatTimer = null
let manualDisconnect = false

const { currentChannel, isJoined, token } = useAppState()
const { fetchChannels } = useChannels()

/** 清除重连定时器 + 心跳定时器 */
const clearTimers = () => {
  if (retryTimer) {
    clearTimeout(retryTimer)
    retryTimer = null
  }
  if (heartbeatTimer) {
    clearInterval(heartbeatTimer)
    heartbeatTimer = null
  }
}

/** 启动 30 秒心跳，防止中间代理切断空闲连接 */
const startHeartbeat = () => {
  clearInterval(heartbeatTimer)
  heartbeatTimer = setInterval(() => {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type: 'ping' }))
    }
  }, 30000)
}

/** 指数退避重连：1s → 2s → 4s → ... → 最多 30s */
const scheduleReconnect = () => {
  clearTimers()
  if (!isJoined.value || manualDisconnect) return

  const delay = Math.min(1000 * Math.pow(2, retryCount), 30000)
  retryCount++
  console.log(`WebSocket 将在 ${delay / 1000}s 后重连 (第${retryCount}次)`)
  retryTimer = setTimeout(() => {
    retryTimer = null
    connect()
  }, delay)
}

/**
 * 建立 WebSocket 连接。
 * - 通过 URL 参数 ?token= 传递 JWT（浏览器 WebSocket 不支持自定义 Header）
 * - 连接成功后先接收墓碑列表（删除事件），再回放最近 50 条历史消息
 * - 异常断开自动重连
 */
const connect = () => {
  clearTimers()
  messages.value = []
  if (!isJoined.value) return

  // 关闭旧连接前先摘掉事件处理器，防止旧 onclose 误触发重连
  if (socket) {
    socket.onclose = null
    socket.onerror = null
    socket.close()
    socket = null
  }

  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
  const tok = token.value || sessionStorage.getItem('voklowave-token') || ''
  const url = protocol + '//' + location.host + '/ws/' + currentChannel.value + '?token=' + encodeURIComponent(tok)
  const ws = new WebSocket(url)
  socket = ws
  manualDisconnect = false

  ws.onopen = () => {
    connected.value = true
    retryCount = 0
    fetchChannels()
    startHeartbeat()
  }

  ws.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data)
      // 心跳应答
      if (msg.type === 'pong') return
      // ── 新增：消息删除事件 — 从本地列表移除对应消息 ──
      if (msg.type === 'message_deleted') {
        const idx = messages.value.findIndex(m => m.id === msg.message_id)
        if (idx !== -1) messages.value.splice(idx, 1)
        return
      }
      // ── 新增：账号被管理员删除 → 强制登出 ──
      if (msg.type === 'user_deleted') {
        const { logout } = useAppState()
        logout()
        return
      }
      // ── 新增：表情回应事件 ──
      if (msg.type === 'reaction_toggled') {
        const { message_id, emoji, action, username } = msg
        const { username: myName } = useAppState()
        const bucket = reactions.value[message_id] || (reactions.value[message_id] = {})
        if (!bucket[emoji]) bucket[emoji] = { count: 0, me: false }
        if (action === 'added') {
          bucket[emoji].count++
          if (username === myName.value) bucket[emoji].me = true
        } else {
          bucket[emoji].count = Math.max(0, bucket[emoji].count - 1)
          if (username === myName.value) bucket[emoji].me = false
        }
        if (bucket[emoji].count === 0) delete bucket[emoji]
        return
      }
      // ── 新增：桌面通知（被 @ 提及时） ──
      if (msg.content && !msg.type) {
        const { username: myName } = useAppState()
        if (myName.value && msg.username !== myName.value) {
          const escapedName = myName.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
          const mentionRegex = new RegExp(`@${escapedName}(?=[\\s,，。.!！?？:：;；]|$)`)
          if (mentionRegex.test(msg.content) && Notification.permission === 'granted') {
            new Notification(`${msg.username} 提到了你`, {
              body: msg.content.substring(0, 100),
              icon: '/favicon.svg',
            })
          }
        }
      }
      // 普通聊天消息
      messages.value.push(msg)
    } catch (e) {
      console.error('消息解析失败:', e)
    }
  }

  ws.onclose = () => {
    if (socket !== ws) return
    socket = null
    connected.value = false
    clearTimers()
    if (!manualDisconnect) {
      scheduleReconnect()
    }
  }

  ws.onerror = () => {
    if (socket !== ws) return
    socket = null
    connected.value = false
    clearTimers()
    if (!manualDisconnect) {
      scheduleReconnect()
    }
  }
}

/** 主动断开 WebSocket，不触发自动重连 */
const disconnect = () => {
  clearTimers()
  manualDisconnect = true
  retryCount = 0
  if (socket) {
    socket.onclose = null
    socket.onerror = null
    socket.close()
    socket = null
  }
  connected.value = false
}

// 切换频道 → 清空消息并重连
watch(currentChannel, () => {
  messages.value = []
  retryCount = 0
  if (isJoined.value) {
    connect()
  }
})

// 加入/离开聊天 → 连接/断开
watch(isJoined, (joined) => {
  if (joined) {
    manualDisconnect = false
    retryCount = 0
    connect()
  } else {
    disconnect()
    messages.value = []
  }
})

// 页面刷新后若已登录则立即连接
if (isJoined.value) {
  connect()
}

/**
 * WebSocket 消息收发管理。
 *
 * 模块级变量保证跨组件共享连接和消息列表。
 */
export function useWebSocket() {

  /** 发送聊天消息到当前频道 */
  const sendMessage = (content) => {
    if (!socket || socket.readyState !== WebSocket.OPEN || !content.trim()) return
    const { username } = useAppState()
    socket.send(JSON.stringify({
      channel: currentChannel.value,
      username: username.value,
      content: content.trim(),
    }))
  }

  // ── 新增：删除消息 ──
  /**
   * 通过 REST API 删除自己发送的消息。
   * 后端会写入墓碑表 + 通过 WebSocket 控制通道广播删除事件。
   * @param {number} messageId
   * @returns {Promise<boolean>} 是否成功
   */
  const deleteMessage = async (messageId) => {
    const tok = token.value
    if (!tok) return false
    try {
      const res = await fetch(`/api/messages/${messageId}`, {
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${tok}` },
      })
      return res.ok
    } catch (err) {
      console.error('删除消息失败:', err)
      return false
    }
  }

  return { messages, connected, scrollToId, reactions, sendMessage, deleteMessage, connect, disconnect }
}

import { ref, watch } from 'vue'
import { useAppState } from './useAppState'
import { useChannels } from './useChannels'

// ── 模块级全局单例 ──
const messages = ref([])
const connected = ref(false)
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
 * - 连接成功后回放最近 50 条历史消息
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
      if (msg.type === 'pong') return
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

  return { messages, connected, sendMessage, connect, disconnect }
}

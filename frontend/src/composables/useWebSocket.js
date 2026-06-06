import { ref, watch } from 'vue'
import { useAppState } from './useAppState'
import { useChannels } from './useChannels'

// ── 全局单例状态 ──
const messages = ref([])
const connected = ref(false)
let socket = null
let retryCount = 0
let retryTimer = null
let heartbeatTimer = null
let manualDisconnect = false

const { currentChannel, isJoined, token } = useAppState()
const { fetchChannels } = useChannels()

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

const startHeartbeat = () => {
  clearInterval(heartbeatTimer)
  heartbeatTimer = setInterval(() => {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type: 'ping' }))
    }
  }, 30000)
}

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

const connect = () => {
  clearTimers()
  messages.value = []
  if (!isJoined.value) return

  // 关闭旧连接前摘掉事件处理器，防止旧 socket 的 onclose 误触发重连
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

// ── 模块级 watcher ──
watch(currentChannel, () => {
  messages.value = []
  retryCount = 0
  if (isJoined.value) {
    connect()
  }
})

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

if (isJoined.value) {
  connect()
}

export function useWebSocket() {
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

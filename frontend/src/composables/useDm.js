import { ref } from 'vue'
import { useAppState } from './useAppState'

// ── 模块级单例 ──
const conversations = ref([])
const activeConvId = ref(null)
const messages = ref([])
const connected = ref(false)
let socket = null
let heartbeatTimer = null

const api = async (path, options = {}, token) => {
  const headers = { 'Content-Type': 'application/json' }
  if (token) headers['Authorization'] = `Bearer ${token}`
  const res = await fetch(path, { headers, ...options })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || `HTTP ${res.status}`)
  }
  return res.json()
}

const getToken = () => useAppState().token.value

// ── 每 10 秒刷新私聊列表 ──
let pollTimer = null
const startPolling = () => {
  if (pollTimer) return
  pollTimer = setInterval(() => fetchList(), 10000)
}
const stopPolling = () => {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
}

const fetchList = async () => {
  try {
    const data = await api('/api/dm/list', {}, getToken())
    conversations.value = data.conversations
  } catch (e) {
    console.error('获取私聊列表失败:', e)
  }
}

const startDm = async (userId) => {
  const data = await api('/api/dm/start', {
    method: 'POST',
    body: JSON.stringify({ user_id: userId }),
  }, getToken())
  await fetchList()
  return data.conversation_id
}

const openDm = async (convId) => {
  disconnectDm()
  activeConvId.value = convId
  messages.value = []

  try {
    const data = await api(`/api/dm/${convId}/messages`, {}, getToken())
    messages.value = data.messages
  } catch (e) {
    console.error('加载私聊消息失败:', e)
  }

  connectDm(convId)
}

const connectDm = (convId) => {
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
  const tok = getToken() || sessionStorage.getItem('voklowave-token') || ''
  const url = `${protocol}//${location.host}/ws/dm/${convId}?token=${encodeURIComponent(tok)}`
  const ws = new WebSocket(url)
  socket = ws

  ws.onopen = () => {
    connected.value = true
    startHeartbeat()
  }

  ws.onmessage = (e) => {
    try {
      const msg = JSON.parse(e.data)
      if (msg.type === 'pong') return
      if (msg.type === 'dm_message') {
        messages.value.push(msg)
      }
    } catch {}
  }

  ws.onclose = () => {
    connected.value = false
    clearHeartbeat()
  }
}

const sendDmMessage = (content) => {
  if (!socket || socket.readyState !== WebSocket.OPEN || !connected.value) return
  socket.send(JSON.stringify({ content }))
}

const startDmByUsername = async (otherUsername) => {
  // 先查用户 ID
  const data = await api(`/api/users?q=${encodeURIComponent(otherUsername)}`, {}, getToken())
  const match = data.find(u => u.username === otherUsername)
  if (!match) throw new Error('用户不存在')
  const convId = await startDm(match.id)
  await openDm(convId)
}

const disconnectDm = () => {
  clearHeartbeat()
  if (socket) {
    socket.onclose = null
    socket.close()
    socket = null
  }
  connected.value = false
  activeConvId.value = null
}

const startHeartbeat = () => {
  clearHeartbeat()
  heartbeatTimer = setInterval(() => {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type: 'ping' }))
    }
  }, 30000)
}

const clearHeartbeat = () => {
  if (heartbeatTimer) {
    clearInterval(heartbeatTimer)
    heartbeatTimer = null
  }
}

export const useDm = () => {
  return {
    conversations, activeConvId, messages, connected,
    fetchList, startDm, startDmByUsername, openDm, sendDmMessage, disconnectDm,
    startPolling, stopPolling
  }
}

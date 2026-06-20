import { ref, onUnmounted } from 'vue'
import { useAppState } from './useAppState'

/**
 * 管理后台实时事件推送：连接 /ws/admin，监听 message_created / message_deleted
 * 等事件，驱动管理页面自动刷新数据。
 */
export function useAdminEvents() {
  const { token } = useAppState()
  const lastEvent = ref(null)
  let socket = null
  let reconnectTimer = null
  let retries = 0

  const connect = () => {
    if (!token.value) return
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
    const url = `${protocol}//${location.host}/ws/admin?token=${encodeURIComponent(token.value)}`

    socket = new WebSocket(url)

    socket.onopen = () => {
      retries = 0
    }

    socket.onmessage = (ev) => {
      try {
        const data = JSON.parse(ev.data)
        const eventTypes = [
          'message_created', 'message_deleted',
          'channel_created', 'channel_deleted',
          'user_created', 'user_deleted', 'user_admin_toggled'
        ]
        if (eventTypes.includes(data.type)) {
          lastEvent.value = { ...data, _ts: Date.now() }
        }
      } catch { /* ignore malformed */ }
    }

    socket.onclose = () => {
      if (retries < 10) {
        const delay = Math.min(1000 * Math.pow(2, retries), 30000)
        retries++
        reconnectTimer = setTimeout(connect, delay)
      }
    }
  }

  const disconnect = () => {
    if (reconnectTimer) clearTimeout(reconnectTimer)
    if (socket) { socket.onclose = null; socket.close() }
  }

  // 在 token 可用时连接
  if (token.value) {
    connect()
  }

  onUnmounted(disconnect)

  return { lastEvent, disconnect }
}

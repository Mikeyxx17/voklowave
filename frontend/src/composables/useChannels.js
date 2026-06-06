import { ref } from 'vue'
import { useAppState } from './useAppState'

// ── 全局单例状态 ──
const channels = ref([])
const loading = ref(false)

export function useChannels() {
  const fetchChannels = async () => {
    loading.value = true
    try {
      const { token } = useAppState()
      const headers = {}
      if (token.value) {
        headers['Authorization'] = `Bearer ${token.value}`
      }
      const res = await fetch('/api/channels', { headers })
      if (res.ok) {
        channels.value = await res.json()
      }
    } catch (err) {
      console.error('获取频道列表失败:', err)
    } finally {
      loading.value = false
    }
  }

  const createChannel = async (name) => {
    // 1. 从 useAppState 中把 token 和我们新写的 isGuest 都解构出来 
    const { token, isGuest } = useAppState()

    // 2. 🌟 前端铁面防线：如果是访客，直接拦截，不发送网络请求，返回 false
    if (isGuest.value) {
      console.warn('访客模式下无法创建新频道')
      return false
    }

    const headers = { 'Content-Type': 'application/json' }
    if (token.value) {
      headers['Authorization'] = `Bearer ${token.value}`
    }

    const res = await fetch('/api/channels', {
      method: 'POST',
      headers,
      body: JSON.stringify({ name }),
    })

    if (res.ok) {
      await fetchChannels()
      return true
    }
    return false
  }

  return { channels, loading, fetchChannels, createChannel }
}

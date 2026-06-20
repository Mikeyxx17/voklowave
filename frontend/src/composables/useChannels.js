import { ref } from 'vue'
import { useAppState } from './useAppState'

// 模块级全局单例
const channels = ref([])
const loading = ref(false)

/**
 * 频道列表管理。
 *
 * 模块级 ref 保证多个组件看到的频道列表是同一份数据。
 */
export function useChannels() {

  /** 从后端拉取频道列表，访客仅获得 general */
  const fetchChannels = async () => {
    loading.value = true
    try {
      const { token, logout } = useAppState()
      const headers = {}
      if (token.value) {
        headers['Authorization'] = `Bearer ${token.value}`
      }
      const res = await fetch('/api/channels', { headers })
      if (res.status === 401) {
        logout()
        return
      }
      if (res.ok) {
        channels.value = await res.json()
      }
    } catch (err) {
      console.error('获取频道列表失败:', err)
    } finally {
      loading.value = false
    }
  }

  /** 创建新频道，访客禁止创建 */
  const createChannel = async (name) => {
    const { token, isGuest } = useAppState()

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

    if (res.status === 401) {
      const { logout } = useAppState()
      logout()
      return false
    }
    if (res.ok) {
      await fetchChannels()
      return true
    }
    return false
  }

  return { channels, loading, fetchChannels, createChannel }
}

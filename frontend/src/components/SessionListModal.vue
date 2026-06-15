<template>
  <Teleport to="body">
    <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center bg-black/40" @click.self="$emit('close')">
      <div class="bg-base-200 rounded-xl border border-base-300/50 shadow-2xl w-full max-w-md mx-4 overflow-hidden">
        <!-- 标题栏 -->
        <div class="flex items-center justify-between px-5 py-4 border-b border-base-300/50">
          <div class="flex items-center gap-2">
            <svg class="w-5 h-5 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
            <span class="font-bold text-base">活跃会话</span>
          </div>
          <button class="btn btn-ghost btn-sm btn-square" @click="$emit('close')">✕</button>
        </div>

        <!-- 加载中 -->
        <div v-if="loading" class="flex items-center justify-center py-16">
          <span class="loading loading-spinner loading-lg text-primary"></span>
        </div>

        <!-- 空状态 -->
        <div v-else-if="sessions.length === 0" class="flex flex-col items-center py-12 text-base-content/30 gap-2">
          <svg class="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-xs">未找到活跃会话</p>
        </div>

        <!-- 会话列表 -->
        <div v-else class="max-h-[50vh] overflow-y-auto divide-y divide-base-300/20">
          <div
            v-for="s in sessions"
            :key="s.id"
            class="flex items-center gap-3 px-5 py-3 hover:bg-base-100/40 transition-colors"
          >
            <!-- 设备图标 -->
            <div class="w-9 h-9 rounded-lg flex items-center justify-center shrink-0"
              :class="s.id === currentSessionId ? 'bg-primary/15 text-primary' : 'bg-base-300/50 text-base-content/50'">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9 17.25v1.007a3 3 0 01-.879 2.122L7.5 21h9l-.621-.621A3 3 0 0115 18.257V17.25m6-12V15a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 15V5.25m18 0A2.25 2.25 0 0018.75 3H5.25A2.25 2.25 0 003 5.25m18 0V12a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 12V5.25" />
              </svg>
            </div>

            <!-- 信息 -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-xs font-semibold text-base-content truncate">
                  {{ s.ip_address || '未知设备' }}
                </span>
                <span v-if="s.id === currentSessionId" class="badge badge-xs badge-primary">当前</span>
              </div>
              <p class="text-[10px] text-base-content/40 mt-0.5">
                {{ fmtTime(s.created_at) }} 登录
              </p>
            </div>

            <!-- 踢出按钮（当前会话不显示） -->
            <button
              v-if="s.id !== currentSessionId"
              class="btn btn-ghost btn-xs text-base-content/30 hover:text-error hover:bg-error/10"
              title="踢出此会话"
              :disabled="revoking === s.id"
              @click="doRevoke(s.id)"
            >
              <span v-if="revoking === s.id" class="loading loading-spinner loading-xs"></span>
              <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { ref, watch } from 'vue'
import { useAppState } from '../composables/useAppState'

const props = defineProps({
  show: { type: Boolean, default: false },
})

defineEmits(['close'])

const { token } = useAppState()
const sessions = ref([])
const loading = ref(false)
const revoking = ref(null)
const currentSessionId = ref(null)

const fetchSessions = async () => {
  if (!token.value) return
  loading.value = true
  try {
    const res = await fetch('/api/sessions', {
      headers: { 'Authorization': `Bearer ${token.value}` }
    })
    if (res.ok) {
      sessions.value = await res.json()
      parseCurrentJti()
    }
  } catch (e) {
    console.error('获取会话列表失败:', e)
  } finally {
    loading.value = false
  }
}

// 从当前 JWT 解析 jti，匹配 sessions 列表找出当前会话
const parseCurrentJti = () => {
  const tok = token.value
  if (!tok) return
  try {
    const payload = JSON.parse(atob(tok.split('.')[1]))
    const currentJti = payload.jti
    const match = sessions.value.find(s => s.jti === currentJti)
    currentSessionId.value = match ? match.id : null
  } catch {}
}

const doRevoke = async (sessionId) => {
  if (!confirm('确定要踢出此会话吗？被踢出的设备将立即断开连接。')) return
  revoking.value = sessionId
  try {
    const res = await fetch(`/api/sessions/${sessionId}`, {
      method: 'DELETE',
      headers: { 'Authorization': `Bearer ${token.value}` }
    })
    if (res.ok) {
      sessions.value = sessions.value.filter(s => s.id !== sessionId)
    }
  } catch (e) {
    console.error('踢出会话失败:', e)
  } finally {
    revoking.value = null
  }
}

// 每次打开弹窗自动刷新列表
watch(() => props.show, (visible) => {
  if (visible) fetchSessions()
})

const fmtTime = (iso) => {
  if (!iso) return ''
  return new Date(iso).toLocaleString('zh-CN', {
    month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit'
  })
}
</script>

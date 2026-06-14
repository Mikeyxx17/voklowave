<template>
  <header class="flex items-center gap-3 px-5 py-3 border-b border-base-300 bg-base-100/70 backdrop-blur-sm shrink-0 hidden lg:flex">
    <div class="flex items-center gap-2 min-w-0">
      <span class="text-xl font-bold text-primary/80">#</span>
      <span class="text-base font-bold text-base-content truncate">{{ currentChannel }}</span>
    </div>

    <div class="badge badge-ghost badge-sm gap-1.5 ml-1">
      <span class="w-1.5 h-1.5 rounded-full bg-success animate-pulse" />
      <span v-if="connected">在线</span>
      <span v-else class="text-error">离线</span>
    </div>

    <div class="badge badge-ghost badge-sm">{{ msgCount }} 条消息</div>

    <div class="flex-1" />

    <!-- ── 新增：消息搜索 ── -->
    <div class="relative">
      <div class="flex items-center gap-1">
        <input
          ref="searchInput"
          v-model="query"
          class="input input-bordered input-sm w-44 bg-base-200/60 text-xs transition-all duration-200"
          :class="query ? 'w-56' : 'w-36'"
          placeholder="搜索消息..."
          maxlength="100"
          @keyup.enter="doSearch"
          @input="onInput"
          @focus="onFocus"
        />
        <!-- ── 搜索范围选择（访客只能搜 general，无需显示） ── -->
        <select
          v-if="!isGuest"
          v-model="searchScope"
          class="select select-bordered select-xs bg-base-200/60 text-xs min-w-0 w-auto"
        >
          <option value="all">全部频道</option>
          <option v-for="ch in channels" :key="ch.id" :value="ch.name">#{{ ch.name }}</option>
        </select>
        <button
          class="btn btn-ghost btn-xs btn-square"
          :disabled="!query.trim() || searching"
          @click="doSearch"
        >
          <svg v-if="searching" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
          <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </button>
      </div>
    </div>

    <div class="badge badge-outline gap-1.5 px-3 py-1">
      <img v-if="avatarUrl" :src="avatarUrl" class="w-4 h-4 rounded-full object-cover" @error="avatarUrl = ''" />
      <span v-else class="w-2 h-2 rounded-full bg-primary" />
      {{ displayName || username }}
    </div>
  </header>

  <!-- ── 搜索结果 —— 独立于 header 的固定定位面板，避免 CSS 层叠上下文干扰 ── -->
  <Teleport to="body">
    <div
      v-if="showResults"
      class="fixed inset-0 z-50 flex items-start justify-center pt-20"
      @click.self="showResults = false"
    >
      <div
        class="bg-base-200 rounded-xl border border-base-300/50 shadow-2xl w-[30rem] max-h-[60vh] flex flex-col"
        :style="{ transform: `translate(${dragX}px, ${dragY}px)` }"
      >
        <div v-if="results.length === 0" class="px-4 py-8 text-center text-xs text-base-content/40">
          未找到匹配的消息
        </div>
        <div v-else>
          <!-- ── 标题栏固定在面板顶部，不随结果滚动 ── -->
          <div
            class="sticky top-0 z-10 bg-base-200 rounded-t-xl px-4 py-2.5 text-[10px] font-semibold text-base-content/40 uppercase tracking-wider border-b border-base-300/50 flex items-center justify-between cursor-move select-none"
            @mousedown="onDragStart"
          >
            <span>找到 {{ total }} 条结果</span>
            <button class="btn btn-ghost btn-xs cursor-pointer" @click="showResults = false" @mousedown.stop>✕</button>
          </div>
          <div class="max-h-[50vh] overflow-y-auto">
            <div
              v-for="(msg, idx) in results"
            :key="msg.id"
            class="px-4 py-3 hover:bg-base-100/50 transition-colors cursor-pointer"
            :class="idx < results.length - 1 ? 'border-b border-base-300/20' : ''"
            @click="jumpTo(msg)"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="text-[10px] font-semibold text-primary/80">#{{ msg.channel }}</span>
              <span class="text-[10px] text-base-content/50">{{ msg.username }}</span>
              <span class="text-[10px] text-base-content/30 ml-auto">{{ fmtTime(msg.created_at) }}</span>
            </div>
            <p class="text-xs text-base-content/80 line-clamp-2 break-words">{{ msg.content }}</p>
          </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { useAppState } from '../composables/useAppState'
import { useWebSocket } from '../composables/useWebSocket'
import { useChannels } from '../composables/useChannels'  // 新增

const { username, displayName, avatarUrl, currentChannel, token, switchChannel, isGuest } = useAppState()
const { messages, connected, scrollToId } = useWebSocket()
const { channels } = useChannels()  // 新增：频道列表用于搜索范围

const msgCount = computed(() => messages.value.length)

// ── 搜索状态 ──
const query = ref('')
const results = ref([])
const total = ref(0)
const searching = ref(false)
const showResults = ref(false)
const searchScope = ref('all')  // 搜索范围：'all' = 全部频道，或具体频道名
const searchInput = ref(null)
let debounceTimer = null

// ── 输入时 300ms 防抖自动搜索 ──
const onInput = () => {
  clearTimeout(debounceTimer)
  if (!query.value.trim()) {
    results.value = []
    total.value = 0
    showResults.value = false
    return
  }
  debounceTimer = setTimeout(doSearch, 300)
}

// ── 获得焦点时如果已有结果则显示 ──
const onFocus = () => {
  if (results.value.length > 0) {
    showResults.value = true
  }
}

// ── 搜索结果窗口拖拽状态 ──
const dragX = ref(0)
const dragY = ref(0)
let dragStartX = 0
let dragStartY = 0

const onDragStart = (e) => {
  dragStartX = e.clientX - dragX.value
  dragStartY = e.clientY - dragY.value
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
}

const onDragMove = (e) => {
  dragX.value = e.clientX - dragStartX
  dragY.value = e.clientY - dragStartY
}

const onDragEnd = () => {
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragEnd)
}

// ── 失焦时不再自动隐藏，改由点击外部关闭 ──

// ── 执行搜索 ──
const doSearch = async () => {
  const q = query.value.trim()
  if (!q) return

    // 构建带范围参数的 URL
    const scopeParam = searchScope.value !== 'all' ? `&channel=${encodeURIComponent(searchScope.value)}` : ''
    searching.value = true
  try {
    const headers = {}
    if (token.value) {
      headers['Authorization'] = `Bearer ${token.value}`
    }
    const res = await fetch(`/api/messages/search?q=${encodeURIComponent(q)}${scopeParam}&limit=20`, { headers })
    if (res.ok) {
      const data = await res.json()
      results.value = data.results
      total.value = data.total
      showResults.value = true
    }
  } catch (err) {
    console.error('搜索失败:', err)
  } finally {
    searching.value = false
  }
}

// ── 点击结果：切换到对应频道 ──
const jumpTo = (msg) => {
  console.log('[搜索点击]', msg.id, msg.channel, msg.content.substring(0, 20))
  // 不关闭面板 — 让用户可以继续点击其他结果进行比较
  // 如果结果在其他频道，自动切换过去
  if (msg.channel !== currentChannel.value) {
    switchChannel(msg.channel)
  }
  // 通知 MessageList 滚动到该消息
  scrollToId.value = msg.id
}

// ── 高亮搜索关键词 ──
const highlight = (text) => {
  if (!query.value.trim()) return escapeHtml(text)
  const escapedQuery = query.value.trim().replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${escapedQuery})`, 'gi')
  return escapeHtml(text).replace(regex, '<mark class="bg-warning/30 text-warning-content rounded px-0.5">$1</mark>')
}

const escapeHtml = (str) => {
  const div = document.createElement('div')
  div.textContent = str
  return div.innerHTML
}

const fmtTime = (iso) => {
  if (!iso) return ''
  const d = new Date(iso)
  const now = new Date()
  const diff = now - d
  if (diff < 864e5 && d.getDate() === now.getDate()) {
    return d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  }
  return d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}
</script>

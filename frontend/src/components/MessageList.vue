<template>
  <div ref="container" class="flex-1 min-h-0 overflow-y-auto scroll-smooth" @scroll="onScroll">
    <!-- 空状态 -->
    <div v-if="messages.length === 0" class="flex flex-col items-center justify-center h-full text-base-content/20 gap-4">
      <svg class="w-20 h-20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.2"
          d="M8.625 9.75a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H8.25m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H12m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0h-.375m-13.5 3.01c0 1.6 1.123 2.994 2.707 3.227 1.087.16 2.185.283 3.293.369V21l4.184-4.183a1.14 1.14 0 01.778-.332 48.294 48.294 0 005.83-.498c1.585-.233 2.708-1.626 2.708-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0012 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018z" />
      </svg>
      <p class="text-sm font-medium">还没有消息，发送第一条吧</p>
    </div>

    <!-- 消息列表 -->
    <div v-else class="px-4 py-3">
      <template v-for="(msg, i) in messages" :key="msg.id || i">
        <!-- 日期分隔 -->
        <div v-if="showDateSep(i, msg)" class="flex items-center gap-3 py-3">
          <div class="flex-1 h-px bg-base-300" />
          <span class="text-[11px] text-base-content/35 font-semibold whitespace-nowrap">{{ fmtDate(msg.created_at) }}</span>
          <div class="flex-1 h-px bg-base-300" />
        </div>

        <!-- 系统消息 -->
        <div v-if="msg.username === '系统'" class="flex justify-center py-1">
          <span class="text-[11px] text-base-content/45 bg-base-200/80 px-3 py-1 rounded-full">{{ msg.content }}</span>
        </div>

        <!-- 普通消息 -->
        <MessageBubble v-else :message="msg" :isMine="msg.username === username" />
      </template>

      <div ref="bottom" class="h-2" />
    </div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick, onMounted } from 'vue'
import { useAppState } from '../composables/useAppState'
import { useWebSocket } from '../composables/useWebSocket'
import MessageBubble from './MessageBubble.vue'

const { username, currentChannel } = useAppState()
const { messages } = useWebSocket()
const container = ref(null)
const bottom = ref(null)
let sticky = true

// 滚到底部
const goBottom = () => {
  bottom.value?.scrollIntoView({ behavior: 'instant' })
}

// 用户手动滚上去了 → 停止自动追随；滚回底部 → 重新追随
const onScroll = () => {
  if (!container.value) return
  const el = container.value
  sticky = el.scrollHeight - el.scrollTop - el.clientHeight < 50
}

// 有新消息时：自己发的消息强制滚到底部，否则仅当 sticky 才追随
watch(messages, () => {
  const latest = messages.value[messages.value.length - 1]
  if (latest && latest.username === username.value) {
    sticky = true
    nextTick(goBottom)
    return
  }
  if (sticky) nextTick(goBottom)
}, { deep: true })

// 切频道 → 强制追随
watch(currentChannel, () => {
  sticky = true
  nextTick(goBottom)
})

onMounted(() => {
  sticky = true
  nextTick(goBottom)
})

// 日期分隔线
let lastDate = ''
const showDateSep = (i, msg) => {
  const cur = msg.created_at ? new Date(msg.created_at).toDateString() : ''
  if (i === 0) { lastDate = cur; return true }
  if (cur !== lastDate) { lastDate = cur; return true }
  return false
}

const fmtDate = (iso) => {
  if (!iso) return ''
  const d = new Date(iso)
  const now = new Date()
  const diff = now - d
  if (diff < 864e5 && d.getDate() === now.getDate()) return '今天'
  if (diff < 1728e5 && d.getDate() === now.getDate() - 1) return '昨天'
  return d.toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric', weekday: 'short' })
}
</script>

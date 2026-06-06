<template>
  <div
    class="chat py-0.5 group"
    :class="isMine ? 'chat-end' : 'chat-start'"
  >
    <!-- 头像 -->
    <div class="chat-image avatar">
      <div class="w-9 rounded-full !rounded-btn" :style="{ background: avatarColor }">
        <span class="text-white text-sm font-bold">{{ initial }}</span>
      </div>
    </div>

    <!-- 头部 -->
    <div class="chat-header flex items-center gap-2 mb-0.5" :class="isMine ? 'flex-row-reverse' : ''">
      <span class="text-xs font-semibold text-base-content/75">{{ message.username }}</span>
      <time class="text-[10px] text-base-content/25 opacity-0 group-hover:opacity-100 transition-opacity">
        {{ fmtTime(message.created_at) }}
      </time>
    </div>

    <!-- 气泡 -->
    <div
      class="chat-bubble text-sm leading-relaxed max-w-lg break-words"
      :class="isMine ? 'chat-bubble-primary' : 'chat-bubble-secondary/60'"
    >
      {{ message.content }}
    </div>

    <!-- 底部（仅自己显示已读状态） -->
    <div v-if="isMine" class="chat-footer text-[10px] text-base-content/25 mt-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
      已发送
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  message: { type: Object, required: true },
  isMine: { type: Boolean, default: false },
})

const initial = computed(() => props.message.username?.charAt(0).toUpperCase() || '?')

const avatarColor = computed(() => {
  const colors = [
    '#6366f1', '#8b5cf6', '#d946ef', '#ec4899',
    '#f43f5e', '#f97316', '#eab308', '#22c55e',
    '#14b8a6', '#06b6d4', '#3b82f6',
  ]
  let hash = 0
  const name = props.message.username || ''
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash)
  }
  return colors[Math.abs(hash) % colors.length]
})

const fmtTime = (iso) => {
  if (!iso) return ''
  return new Date(iso).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
}
</script>

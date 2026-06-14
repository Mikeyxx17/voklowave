<template>
  <div
    class="w-fit max-w-full"
    :class="isMine ? 'ml-auto' : ''"
    @mouseenter="hovering = true"
    @mouseleave="onBubbleLeave"
  >
    <div
      class="chat py-0.5"
      :class="isMine ? 'chat-end' : 'chat-start'"
    >
      <!-- 头像 -->
      <div class="chat-image avatar">
        <div class="w-9 rounded-full !rounded-btn overflow-hidden" :style="message.avatar_url ? {} : { background: avatarColor }">
          <img v-if="message.avatar_url" :src="message.avatar_url" class="w-full h-full object-cover" @error="message.avatar_url = null" />
          <span v-else class="text-white text-sm font-bold">{{ initial }}</span>
        </div>
      </div>

      <!-- 头部 -->
      <div class="chat-header flex items-center gap-2 mb-0.5" :class="isMine ? 'flex-row-reverse' : ''">
        <span class="text-xs font-semibold text-base-content/75">{{ message.display_name || message.username }}</span>
        <time class="text-[10px] text-base-content/25 transition-opacity" :class="hovering ? 'opacity-100' : 'opacity-0'">
          {{ fmtTime(message.created_at) }}
        </time>
      </div>

      <!-- 气泡 -->
      <div
        class="chat-bubble text-sm leading-relaxed max-w-lg break-words"
        :class="isMine ? 'chat-bubble-primary' : 'chat-bubble-secondary/60'"
      >
        <div v-html="renderedContent" class="markdown-body" />
      </div>

      <!-- 底部（仅自己显示已读状态 + 删除按钮） -->
      <div v-if="isMine" class="chat-footer flex items-center gap-2 mt-0.5 transition-opacity" :class="hovering ? 'opacity-100' : 'opacity-0'">
        <span class="text-[10px] text-base-content/25">已发送</span>
        <button
          class="btn btn-ghost btn-xs text-base-content/30 hover:text-error hover:bg-error/10 px-1"
          title="删除消息"
          @click="$emit('delete', message.id)"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      </div>
    </div>

    <!-- ── 反应展示（一行） ── -->
    <div v-if="msgReactions" class="flex flex-wrap gap-1 mt-0.5" :class="isMine ? 'justify-end' : 'pl-[3.25rem]'">
      <button
        v-for="(info, emoji) in msgReactions"
        :key="emoji"
        class="btn btn-xs px-1.5 py-0 h-auto min-h-0 text-[12px] leading-none rounded-field transition-all"
        :class="info.me ? 'bg-primary/20 text-primary' : 'bg-base-300/50 text-base-content/60 hover:bg-base-300'"
        @click="toggleReaction(emoji)"
      >
        {{ emoji }}&nbsp;<span class="text-[10px]">{{ info.count }}</span>
      </button>
    </div>
    <!-- ── 表情触发按钮（hover 出现，点击展开选择器） ── -->
    <div v-show="hovering" class="mt-0.5 inline-block relative" :class="isMine ? 'text-right' : 'pl-[3.25rem]'">
      <button
        class="btn btn-xs btn-ghost px-1 h-auto min-h-0 text-sm"
        @click.stop="showPicker = !showPicker"
      >😊</button>
      <!-- 弹出面板 -->
      <div
        v-if="showPicker"
        data-picker
        class="absolute bottom-full mb-1 bg-base-200 border border-base-300/50 rounded-full px-2 py-1 shadow z-30 flex gap-1"
        :class="isMine ? 'right-0' : 'left-0'"
        style="white-space: nowrap"
      >
        <button
          v-for="e in EMOJIS"
          :key="e"
          class="text-sm hover:scale-125 transition-transform"
          @click="pickEmoji(e)"
        >{{ e }}</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { renderMarkdown } from '../composables/useMarkdown'
import { useWebSocket } from '../composables/useWebSocket'
import { useAppState } from '../composables/useAppState'

// ── 预设表情 ──
const EMOJIS = ['👍', '❤️', '😂', '😮', '😢', '🙏']

// ── 表情选择器展开状态 ──
const showPicker = ref(false)

// ── 点击面板外部自动关闭 ──
const closePickerOnOutside = (e) => {
  if (!showPicker.value) return
  // 延迟检查，确保 Vue 已更新 DOM
  setTimeout(() => {
    if (showPicker.value && !e.target.closest('[data-picker]')) {
      showPicker.value = false
    }
  }, 100)
}
document.addEventListener('click', closePickerOnOutside)
// ── 鼠标是否悬停在气泡区域（精确到 chat 组件） ──
const hovering = ref(false)

// ── 鼠标离开气泡：仅在选择器未展开时隐藏 ──
const onBubbleLeave = () => {
  if (!showPicker.value) hovering.value = false
}

const props = defineProps({
  message: { type: Object, required: true },
  isMine: { type: Boolean, default: false },
})

defineEmits(['delete'])

const { reactions } = useWebSocket()
const { token } = useAppState()

// ── 当前消息的反应 ──
const msgReactions = computed(() => {
  const r = reactions.value[props.message.id]
  return r && Object.keys(r).length > 0 ? r : null
})

// ── 从弹出面板选表情 → 关闭面板后切换 ──
const pickEmoji = async (emoji) => {
  showPicker.value = false
  await toggleReaction(emoji)
}

// ── 切换表情 ──
const toggleReaction = async (emoji) => {
  if (!props.message.id) return
  await fetch(`/api/messages/${props.message.id}/react`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token.value}`,
    },
    body: JSON.stringify({ emoji }),
  })
}

// ── Markdown 渲染 + @提及高亮 ──
const renderedContent = computed(() => {
  let html = renderMarkdown(props.message.content)
  // 在已渲染的 HTML 中高亮 @用户名（避免触碰 <a> 等标签内的内容）
  html = html.replace(/(^|\s)@(\w{3,30})(?=[\s,，。.!！?？:：;；]|$)/g,
    '$1<span class="text-primary font-semibold bg-primary/10 rounded px-1">@$2</span>')
  return html
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

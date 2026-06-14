<template>
  <div class="px-4 py-3 border-t border-base-300 bg-base-200/60 shrink-0">
    <div class="flex gap-2.5 items-end max-w-4xl mx-auto">
      <!-- 输入框 + @ 补全 -->
      <div class="flex-1 relative">
        <textarea
          ref="inputEl"
          v-model="text"
          class="textarea textarea-bordered w-full bg-base-100 focus:input-primary transition-all duration-200 text-sm resize-none"
          :class="text.length > 450 ? 'textarea-warning' : ''"
          placeholder="输入消息... (Enter 发送，@ 提及用户)"
          maxlength="500"
          rows="1"
          @input="onTextInput"
          @keydown="onKeyDown"
          @blur="onBlurInput"
        />
        <span
          class="absolute right-3 bottom-3 text-[10px] transition-colors pointer-events-none"
          :class="text.length > 450 ? 'text-warning' : 'text-base-content/25'"
        >
          {{ text.length }}/500
        </span>

        <!-- @ 提及补全下拉 -->
        <div
          v-if="mentionUsers.length > 0"
          class="absolute bottom-full left-0 mb-1 bg-base-200 rounded-lg border border-base-300/50 shadow-xl z-40 w-56 max-h-48 overflow-y-auto"
        >
          <div
            v-for="u in mentionUsers"
            :key="u.username"
            class="px-3 py-2 hover:bg-base-100/50 cursor-pointer text-sm flex items-center gap-2"
            :class="highlightIdx === mentionIdx(u) ? 'bg-primary/10' : ''"
            @mousedown.prevent="selectMention(u)"
          >
            <span class="text-base-content/70">@{{ u.username }}</span>
            <span v-if="u.display_name" class="text-xs text-base-content/40">{{ u.display_name }}</span>
          </div>
        </div>
      </div>

      <!-- 发送按钮 -->
      <button
        class="btn btn-primary btn-circle btn-sm shadow-lg shadow-primary/25 hover:shadow-primary/40 transition-all duration-200 hover:scale-105 active:scale-95 shrink-0"
        :disabled="!text.trim()"
        @click="doSend"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M6 12L3.269 3.126A59.768 59.768 0 0121.485 12 59.77 59.77 0 013.27 20.876L5.999 12zm0 0h7.5" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick, onMounted } from 'vue'
import { useWebSocket } from '../composables/useWebSocket'
import { useAppState } from '../composables/useAppState'

const { sendMessage } = useWebSocket()
const { token } = useAppState()
const text = ref('')
const inputEl = ref(null)
const mentionUsers = ref([])
const highlightIdx = ref(0)
let mentionStart = -1
let searchTimer = null

onMounted(() => {
  inputEl.value?.focus()
})

// 输入时检测 @ 触发补全
const onTextInput = () => {
  const val = text.value
  const cursor = inputEl.value?.selectionStart || val.length
  // @ 触发检测
  // 找到光标前最近的 @ 位置
  const atIdx = val.lastIndexOf('@', cursor - 1)
  if (atIdx >= 0 && (atIdx === 0 || val[atIdx - 1] === ' ' || val[atIdx - 1] === '\n')) {
    const query = val.slice(atIdx + 1, cursor)
    if (!query.includes(' ')) {
      mentionStart = atIdx
      clearTimeout(searchTimer)
      searchTimer = setTimeout(() => searchUsers(query), 150)
      return
    }
  }
  mentionStart = -1
  mentionUsers.value = []
  highlightIdx.value = 0
}

// 搜索用户
const searchUsers = async (q) => {
  // 查询匹配用户
  if (!q) { mentionUsers.value = []; return }
  try {
    const headers = {}
    if (token.value) headers['Authorization'] = `Bearer ${token.value}`
    const res = await fetch(`/api/users?q=${encodeURIComponent(q)}`, { headers })
    if (res.ok) {
      mentionUsers.value = await res.json()
      highlightIdx.value = 0
    }
  } catch {}
}

// 键盘导航
const onKeyDown = (e) => {
  // Enter：有下拉时选人，无下拉时发送；Shift+Enter 换行
  if (e.key === 'Enter' && !e.shiftKey) {
    if (mentionUsers.value.length > 0) {
      e.preventDefault()
      selectMention(mentionUsers.value[highlightIdx.value])
      return
    }
    e.preventDefault()
    doSend()
    return
  }
  if (mentionUsers.value.length === 0) return
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    highlightIdx.value = (highlightIdx.value + 1) % mentionUsers.value.length
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    highlightIdx.value = (highlightIdx.value - 1 + mentionUsers.value.length) % mentionUsers.value.length
  } else if (e.key === 'Tab') {
    e.preventDefault()
    selectMention(mentionUsers.value[highlightIdx.value])
  } else if (e.key === 'Escape') {
    mentionUsers.value = []
  }
}

// 选中用户
const selectMention = (user) => {
  const before = text.value.slice(0, mentionStart)
  const after = text.value.slice(inputEl.value?.selectionStart || mentionStart + 1)
  text.value = before + '@' + user.username + ' ' + after
  mentionUsers.value = []
  mentionStart = -1
  nextTick(() => inputEl.value?.focus())
}

// 失焦时关闭
const onBlurInput = () => {
  setTimeout(() => { mentionUsers.value = [] }, 200)
}

// 计算高亮下标对应数组中的位置
const mentionIdx = (u) => mentionUsers.value.indexOf(u)

// 发送
const doSend = () => {
  if (!text.value.trim()) return
  sendMessage(text.value)
  text.value = ''
  mentionUsers.value = []
  inputEl.value?.focus()
}
</script>

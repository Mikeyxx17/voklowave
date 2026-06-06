<template>
  <div class="px-4 py-3 border-t border-base-300 bg-base-200/60 shrink-0">
    <div class="flex gap-2.5 items-end max-w-4xl mx-auto">
      <!-- 输入框 -->
      <div class="flex-1 relative">
        <input
          ref="inputEl"
          v-model="text"
          class="input input-bordered w-full bg-base-100 focus:input-primary transition-all duration-200 text-sm"
          :class="text.length > 450 ? 'input-warning' : ''"
          placeholder="输入消息... (Enter 发送)"
          maxlength="500"
          @keyup.enter="doSend"
        />
        <span
          class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] transition-colors pointer-events-none"
          :class="text.length > 450 ? 'text-warning' : 'text-base-content/25'"
        >
          {{ text.length }}/500
        </span>
      </div>

      <!-- 发送按钮 -->
      <button
        class="btn btn-primary btn-circle btn-sm shadow-lg shadow-primary/25 hover:shadow-primary/40 transition-all duration-200 hover:scale-105 active:scale-95"
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
import { ref, onMounted } from 'vue'
import { useWebSocket } from '../composables/useWebSocket'

const { sendMessage } = useWebSocket()
const text = ref('')
const inputEl = ref(null)

onMounted(() => {
  inputEl.value?.focus()
})

const doSend = () => {
  if (!text.value.trim()) return
  sendMessage(text.value)
  text.value = ''
  inputEl.value?.focus()
}
</script>

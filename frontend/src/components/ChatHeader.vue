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

    <div class="badge badge-outline gap-1.5 px-3 py-1">
      <span class="w-2 h-2 rounded-full bg-primary" />
      {{ username }}
    </div>
  </header>
</template>

<script setup>
import { computed } from 'vue'
import { useAppState } from '../composables/useAppState'
import { useWebSocket } from '../composables/useWebSocket'

const { username, currentChannel } = useAppState()
const { messages, connected } = useWebSocket()

const msgCount = computed(() => messages.value.length)
</script>

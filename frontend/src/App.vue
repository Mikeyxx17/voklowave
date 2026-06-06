<template>
  <div v-if="initializing" class="min-h-screen flex items-center justify-center bg-base-100">
    <span class="loading loading-spinner loading-lg text-primary"></span>
  </div>
  <LoginView v-else-if="!isJoined" />
  <ChatLayout v-else />
</template>

<script setup>
import { useAppState } from './composables/useAppState'
import LoginView from './components/LoginView.vue'
import ChatLayout from './components/ChatLayout.vue'

const { isJoined, initializing } = useAppState()
</script>

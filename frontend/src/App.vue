<template>
  <div v-if="initializing" class="min-h-screen flex items-center justify-center bg-base-100">
    <span class="loading loading-spinner loading-lg text-primary"></span>
  </div>
  <router-view v-else />
</template>

<script setup>
import { useAppState } from './composables/useAppState'
const { initializing } = useAppState()
</script>

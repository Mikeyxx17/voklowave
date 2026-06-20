<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">📊 仪表盘</h1>
    <div v-if="loading" class="text-center py-12"><span class="loading loading-spinner loading-lg"></span></div>
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>
    <div v-else class="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <div class="card bg-base-100 shadow">
        <div class="card-body">
          <h3 class="text-base-content/60 text-sm">总用户数</h3>
          <p class="text-3xl font-bold">{{ data.total_users }}</p>
        </div>
      </div>
      <div class="card bg-base-100 shadow">
        <div class="card-body">
          <h3 class="text-base-content/60 text-sm">总消息数</h3>
          <p class="text-3xl font-bold">{{ data.total_messages }}</p>
        </div>
      </div>
      <div class="card bg-base-100 shadow">
        <div class="card-body">
          <h3 class="text-base-content/60 text-sm">频道数</h3>
          <p class="text-3xl font-bold">{{ data.total_channels }}</p>
        </div>
      </div>
      <div class="card bg-base-100 shadow">
        <div class="card-body">
          <h3 class="text-base-content/60 text-sm">今日消息</h3>
          <p class="text-3xl font-bold">{{ data.today_messages }}</p>
        </div>
      </div>
      <div class="card bg-base-100 shadow col-span-2 lg:col-span-4">
        <div class="card-body">
          <h3 class="text-base-content/60 text-sm">访客账号数</h3>
          <p class="text-xl">{{ data.guest_count }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, onMounted } from 'vue'
import { useAdmin } from '../../composables/useAdmin'
import { useAdminEvents } from '../../composables/useAdminEvents'

const { dashboard } = useAdmin()
const { lastEvent } = useAdminEvents()
const data = ref({})
const loading = ref(true)
const error = ref('')

let refreshTimer = null
watch(lastEvent, (ev) => {
  if (!ev) return
  if (refreshTimer) clearTimeout(refreshTimer)
  // 用户/频道/消息变更 → 仪表盘数字实时刷新
  refreshTimer = setTimeout(() => fetch(), 1000)
})

const fetch = async () => {
  try {
    data.value = await dashboard()
    loading.value = false
  } catch (e) {
    error.value = e.message
    loading.value = false
  }
}

onMounted(() => fetch())
</script>

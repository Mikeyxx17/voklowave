<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">📋 操作日志</h1>

    <div v-if="loading" class="text-center py-12"><span class="loading loading-spinner loading-lg"></span></div>
    <div v-else-if="error" class="alert alert-error mb-4">{{ error }}</div>
    <template v-else>
      <div class="overflow-x-auto">
        <table class="table table-zebra">
          <thead>
            <tr><th>ID</th><th>操作者</th><th>动作</th><th>目标</th><th>时间</th></tr>
          </thead>
          <tbody>
            <tr v-for="l in logs" :key="l.id">
              <td>{{ l.id }}</td>
              <td class="font-medium">{{ l.admin_name }}</td>
              <td>
                <span :class="actionBadge(l.action)">{{ l.action }}</span>
              </td>
              <td class="text-sm">{{ l.target }}</td>
              <td class="text-sm text-base-content/50">{{ fmt(l.created_at) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="flex justify-between items-center mt-4">
        <span class="text-sm text-base-content/50">共 {{ total }} 条</span>
        <div class="join">
          <button class="join-item btn btn-sm" :disabled="page <= 0" @click="page--;fetch()">上一页</button>
          <button class="join-item btn btn-sm" :disabled="logs.length < 20" @click="page++;fetch()">下一页</button>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useAdmin } from '../../composables/useAdmin'

const { auditLogs } = useAdmin()
const logs = ref([])
const total = ref(0)
const page = ref(0)
const loading = ref(false)
const error = ref('')

const fetch = async () => {
  loading.value = true; error.value = ''
  try {
    const data = await auditLogs(page.value)
    logs.value = data.logs; total.value = data.total
  } catch (e) { error.value = e.message } finally { loading.value = false }
}

const actionBadge = (action) => {
  const map = {
    delete_user: 'badge badge-error',
    toggle_admin: 'badge badge-info',
    delete_channel: 'badge badge-warning',
    force_delete_message: 'badge badge-error',
  }
  return (map[action] || 'badge') + ' badge-sm'
}

const fmt = (ts) => ts ? new Date(ts).toLocaleString() : '-'

fetch()
</script>

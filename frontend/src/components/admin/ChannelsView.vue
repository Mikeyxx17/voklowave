<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">💬 频道管理</h1>

    <div v-if="loading" class="text-center py-12"><span class="loading loading-spinner loading-lg"></span></div>
    <div v-else-if="error" class="alert alert-error mb-4">{{ error }}</div>
    <template v-else>
      <div class="overflow-x-auto">
        <table class="table table-zebra">
          <thead>
            <tr><th>ID</th><th>频道名</th><th>消息数</th><th>创建时间</th><th>操作</th></tr>
          </thead>
          <tbody>
            <tr v-for="c in channels" :key="c.id">
              <td>{{ c.id }}</td><td class="font-medium"># {{ c.name }}</td>
              <td>{{ c.msg_count }}</td>
              <td class="text-sm text-base-content/50">{{ fmt(c.created_at) }}</td>
              <td>
                <button v-if="isSuperAdmin" @click="delChannel(c)" class="btn btn-error btn-xs btn-outline" :disabled="deleting === c.id">
                  删除
                </button>
                <span v-else class="text-xs text-base-content/30">—</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import { useAdmin } from '../../composables/useAdmin'
import { useAdminEvents } from '../../composables/useAdminEvents'
import { useAppState } from '../../composables/useAppState'

const { listChannels, deleteChannel } = useAdmin()
const { isSuperAdmin } = useAppState()
const { lastEvent } = useAdminEvents()
const channels = ref([])
const loading = ref(false)
const error = ref('')
const deleting = ref(null)

let refreshTimer = null
watch(lastEvent, (ev) => {
  if (!ev) return
  if (ev.type !== 'channel_created' && ev.type !== 'channel_deleted') return
  if (refreshTimer) clearTimeout(refreshTimer)
  refreshTimer = setTimeout(() => fetch(), 1000)
})

const fetch = async () => {
  loading.value = true; error.value = ''
  try { const data = await listChannels(); channels.value = data.channels } catch (e) { error.value = e.message } finally { loading.value = false }
}

const delChannel = async (c) => {
  if (!confirm(`确定删除频道 "#${c.name}"？该频道所有消息将被移除。`)) return
  deleting.value = c.id
  try { await deleteChannel(c.id); fetch() } catch (e) { error.value = e.message } finally { deleting.value = null }
}

const fmt = (ts) => ts ? new Date(ts).toLocaleString() : '-'

fetch()
</script>

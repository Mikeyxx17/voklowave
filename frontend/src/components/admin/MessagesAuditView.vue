<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">📝 消息审计</h1>

    <div class="flex gap-2 mb-4">
      <input v-model="search" @keyup.enter="fetch" placeholder="搜索消息内容..."
        class="input input-bordered flex-1 max-w-sm" />
      <button @click="fetch" class="btn btn-primary btn-sm">搜索</button>
      <span class="text-xs text-base-content/40 self-center ml-2">🔴 实时监听中</span>
    </div>

    <div v-if="loading" class="text-center py-12"><span class="loading loading-spinner loading-lg"></span></div>
    <div v-else-if="error" class="alert alert-error mb-4">{{ error }}</div>
    <template v-else>
      <div class="overflow-x-auto">
        <table class="table table-zebra">
          <thead>
            <tr><th>ID</th><th>频道</th><th>用户名</th><th>内容</th><th>时间</th><th>操作</th></tr>
          </thead>
          <tbody>
            <tr v-for="m in messages" :key="m.id">
              <td>{{ m.id }}</td>
              <td># {{ m.channel }}</td>
              <td>{{ m.username }}</td>
              <td class="max-w-xs truncate text-sm">{{ m.content }}</td>
              <td class="text-sm text-base-content/50">{{ fmt(m.created_at) }}</td>
              <td>
                <button @click="delMsg(m)" class="btn btn-error btn-xs btn-outline" :disabled="deleting === m.id">
                  删除
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="flex justify-between items-center mt-4">
        <span class="text-sm text-base-content/50">共 {{ total }} 条</span>
        <div class="join">
          <button class="join-item btn btn-sm" :disabled="page <= 0" @click="page--;fetch()">上一页</button>
          <button class="join-item btn btn-sm" :disabled="messages.length < 20" @click="page++;fetch()">下一页</button>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, watch, onUnmounted } from 'vue'
import { useAdmin } from '../../composables/useAdmin'
import { useAdminEvents } from '../../composables/useAdminEvents'

const { auditMessages, deleteMessage } = useAdmin()
const { lastEvent } = useAdminEvents()
const messages = ref([])
const total = ref(0)
const page = ref(0)
const search = ref('')
const loading = ref(false)
const error = ref('')
const deleting = ref(null)

// 防抖定时器：收到事件后最多 2 秒刷新一次
let refreshTimer = null

watch(lastEvent, (ev) => {
  if (!ev) return
  if (refreshTimer) clearTimeout(refreshTimer)
  refreshTimer = setTimeout(() => {
    // 仅在第一页且无搜索词时自动刷新（新消息出现在第一页）
    if (page.value === 0 && !search.value) {
      fetch()
    }
  }, 2000)
})

onUnmounted(() => {
  if (refreshTimer) clearTimeout(refreshTimer)
})

const fetch = async () => {
  loading.value = true; error.value = ''
  try {
    const data = await auditMessages(search.value, page.value)
    messages.value = data.messages; total.value = data.total
  } catch (e) { error.value = e.message } finally { loading.value = false }
}

const delMsg = async (m) => {
  if (!confirm(`确定删除消息 #${m.id}？`)) return
  deleting.value = m.id
  try { await deleteMessage(m.id); fetch() } catch (e) { error.value = e.message } finally { deleting.value = null }
}

const fmt = (ts) => ts ? new Date(ts).toLocaleString() : '-'

fetch()
</script>

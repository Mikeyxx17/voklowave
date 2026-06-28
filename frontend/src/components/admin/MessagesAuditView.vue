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
      <!-- 批量操作栏 -->
      <div v-if="selected.length > 0" class="flex items-center gap-2 mb-3 p-2 bg-base-200/50 rounded-lg">
        <span class="text-sm font-medium">已选 {{ selected.length }} 条</span>
        <button @click="batchDelete" class="btn btn-error btn-sm" :disabled="batchDeleting">批量删除</button>
        <button @click="selected=[]" class="btn btn-ghost btn-sm">取消选择</button>
      </div>
      <div class="overflow-x-auto">
        <table class="table table-zebra">
          <thead>
            <tr>
              <th><input type="checkbox" @change="toggleAll" :checked="allSelected"></th>
              <th>ID</th><th>频道</th><th>用户名</th><th>内容</th><th>时间</th><th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="m in messages" :key="m.id" :class="{ 'bg-base-200/30': selected.includes(m.id) }">
              <td><input type="checkbox" :checked="selected.includes(m.id)" @change="toggleSelect(m.id)"></td>
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
import { ref, watch, onUnmounted, computed } from 'vue'
import { useAdmin } from '../../composables/useAdmin'
import { useAdminEvents } from '../../composables/useAdminEvents'

const { auditMessages, deleteMessage, batchDeleteMessages } = useAdmin()
const { lastEvent } = useAdminEvents()
const messages = ref([])
const total = ref(0)
const page = ref(0)
const search = ref('')
const loading = ref(false)
const error = ref('')
const deleting = ref(null)
const selected = ref([])
const batchDeleting = ref(false)

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

const allSelected = computed(() => messages.value.length > 0 && selected.value.length === messages.value.length)

const toggleSelect = (id) => {
  const idx = selected.value.indexOf(id)
  if (idx >= 0) selected.value.splice(idx, 1)
  else selected.value.push(id)
}

const toggleAll = () => {
  if (allSelected.value) selected.value = []
  else selected.value = messages.value.map(m => m.id)
}

const batchDelete = async () => {
  if (!confirm(`确定批量删除 ${selected.value.length} 条消息？`)) return
  batchDeleting.value = true
  try {
    await batchDeleteMessages(selected.value)
    selected.value = []
    fetch()
  } catch (e) { error.value = e.message } finally { batchDeleting.value = false }
}

const delMsg = async (m) => {
  if (!confirm(`确定删除消息 #${m.id}？`)) return
  deleting.value = m.id
  try { await deleteMessage(m.id); fetch() } catch (e) { error.value = e.message } finally { deleting.value = null }
}

const fmt = (ts) => ts ? new Date(ts).toLocaleString() : '-'

fetch()
</script>

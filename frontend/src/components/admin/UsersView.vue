<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">👥 用户管理</h1>

    <!-- 搜索栏 -->
    <div class="flex gap-2 mb-4">
      <input v-model="search" @keyup.enter="fetchUsers" placeholder="搜索用户名或邮箱..."
        class="input input-bordered flex-1 max-w-sm" />
      <button @click="fetchUsers" class="btn btn-primary btn-sm">搜索</button>
    </div>

    <div v-if="loading" class="text-center py-12"><span class="loading loading-spinner loading-lg"></span></div>
    <div v-else-if="error" class="alert alert-error mb-4">{{ error }}</div>
    <template v-else>
      <!-- 批量操作栏 -->
      <div v-if="selected.length > 0" class="flex items-center gap-2 mb-3 p-2 bg-base-200/50 rounded-lg">
        <span class="text-sm font-medium">已选 {{ selected.length }} 人</span>
        <button v-if="isSuperAdmin" @click="batchDeleteUsers" class="btn btn-error btn-sm">批量删除</button>
        <button v-if="isSuperAdmin" @click="batchToggleAdmin" class="btn btn-warning btn-sm">批量升降</button>
        <button @click="selected=[]" class="btn btn-ghost btn-sm">取消选择</button>
      </div>
      <div class="overflow-x-auto">
        <table class="table table-zebra">
          <thead>
            <tr>
              <th><input type="checkbox" @change="toggleAllUsers" :checked="allUsersSelected"></th>
              <th>ID</th><th>用户名</th><th>邮箱</th><th>状态</th><th>角色</th><th v-if="isSuperAdmin">升降</th><th>注册时间</th><th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="u in users" :key="u.id" :class="{ 'bg-base-200/30': selected.includes(u.id) }">
              <td><input type="checkbox" :checked="selected.includes(u.id)" @change="toggleSelectUser(u.id)" :disabled="u.is_owner"></td>
              <td>{{ u.id }}</td>
              <td class="font-medium">
                {{ u.username }}
                <span v-if="u.is_guest" class="badge badge-ghost badge-xs">访客</span>
              </td>
              <td class="text-sm text-base-content/70">{{ u.email }}</td>
              <td>
                <span :class="u.is_verified ? 'badge badge-success badge-sm' : 'badge badge-warning badge-sm'">
                  {{ u.is_verified ? '已验证' : '未验证' }}
                </span>
              </td>
              <td>
                <span :class="roleBadge(u)">{{ roleName(u) }}</span>
              </td>
              <td v-if="isSuperAdmin">
                <span v-if="u.is_guest || u.is_owner" class="text-xs text-base-content/30">—</span>
                <input v-else type="checkbox" class="toggle toggle-sm" :checked="u.is_admin"
                  @change="toggleAdmin(u)" :disabled="toggling === u.id" />
              </td>
              <td class="text-sm text-base-content/50">{{ fmt(u.created_at) }}</td>
              <td class="flex gap-1">
                <button v-if="!u.is_owner" @click="doMute(u)" class="btn btn-warning btn-xs btn-outline" :disabled="muting === u.id">
                  {{ u.muted_until && new Date(u.muted_until) > new Date() ? '解除' : '禁言' }}
                </button>
                <button v-if="isSuperAdmin && !u.is_superadmin && !u.is_owner" @click="delUser(u)" class="btn btn-error btn-xs btn-outline" :disabled="deleting === u.id">
                  删除
                </button>
                <span v-if="u.is_owner" class="text-xs text-base-content/30">—</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="flex justify-between items-center mt-4">
        <span class="text-sm text-base-content/50">共 {{ total }} 条</span>
        <div class="join">
          <button class="join-item btn btn-sm" :disabled="page <= 0" @click="page--;fetchUsers()">上一页</button>
          <button class="join-item btn btn-sm" :disabled="users.length < 20" @click="page++;fetchUsers()">下一页</button>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, watch, computed } from 'vue'
import { useAdmin } from '../../composables/useAdmin'
import { useAppState } from '../../composables/useAppState'
import { useAdminEvents } from '../../composables/useAdminEvents'

const { listUsers, deleteUser, toggleAdmin: apiToggleAdmin, muteUser: apiMuteUser, batchDeleteUsers: apiBatchDeleteUsers, batchToggleAdmin: apiBatchToggleAdmin } = useAdmin()
const { isSuperAdmin } = useAppState()
const { lastEvent } = useAdminEvents()
const users = ref([])
const total = ref(0)
const page = ref(0)
const search = ref('')
const loading = ref(false)
const error = ref('')
const deleting = ref(null)
const toggling = ref(null)
const muting = ref(null)
const selected = ref([])
const batchLoading = ref(false)

let refreshTimer = null
watch(lastEvent, (ev) => {
  if (!ev) return
  if (ev.type !== 'user_created' && ev.type !== 'user_deleted' && ev.type !== 'user_admin_toggled') return
  // 管理员 A 的操作 → 管理员 B 自动刷新
  if (refreshTimer) clearTimeout(refreshTimer)
  refreshTimer = setTimeout(() => fetchUsers(), 1000)
})

const fetchUsers = async () => {
  loading.value = true; error.value = ''
  try {
    const data = await listUsers(search.value, page.value)
    users.value = data.users; total.value = data.total
  } catch (e) { error.value = e.message } finally { loading.value = false }
}

const delUser = async (u) => {
  if (!confirm(`确定删除用户 "${u.username}"？其所有消息将被移除。`)) return
  deleting.value = u.id
  try { await deleteUser(u.id); fetchUsers() } catch (e) { error.value = e.message; fetchUsers() } finally { deleting.value = null }
}

const toggleAdmin = async (u) => {
  const action = u.is_admin ? '撤销管理员权限' : '升级为管理员'
  if (!confirm(`确定要将 "${u.username}" ${action}？`)) return
  toggling.value = u.id
  try { await apiToggleAdmin(u.id); fetchUsers() } catch (e) { error.value = e.message; fetchUsers() } finally { toggling.value = null }
}

const allUsersSelected = computed(() => {
  const selectable = users.value.filter(u => !u.is_owner)
  return selectable.length > 0 && selectable.every(u => selected.value.includes(u.id))
})

const toggleSelectUser = (id) => {
  const idx = selected.value.indexOf(id)
  if (idx >= 0) selected.value.splice(idx, 1)
  else selected.value.push(id)
}

const toggleAllUsers = () => {
  if (allUsersSelected.value) {
    selected.value = []
  } else {
    selected.value = users.value.filter(u => !u.is_owner).map(u => u.id)
  }
}

const batchDeleteUsers = async () => {
  if (!confirm(`确定批量删除 ${selected.value.length} 个用户？`)) return
  batchLoading.value = true
  try {
    const res = await apiBatchDeleteUsers(selected.value)
    alert(`已删除 ${res.deleted} 个用户` + (res.skipped_owners ? `，${res.skipped_owners} 个 Owner 被跳过` : ''))
    selected.value = []
    fetchUsers()
  } catch (e) { error.value = e.message } finally { batchLoading.value = false }
}

const batchToggleAdmin = async () => {
  if (!confirm(`确定批量升降 ${selected.value.length} 个用户的管理员身份？`)) return
  batchLoading.value = true
  try {
    const res = await apiBatchToggleAdmin(selected.value)
    alert(`已处理 ${res.toggled} 个用户` + (res.skipped ? `，${res.skipped} 个被跳过` : ''))
    selected.value = []
    fetchUsers()
  } catch (e) { error.value = e.message } finally { batchLoading.value = false }
}

const doMute = async (u) => {
  if (u.muted_until && new Date(u.muted_until) > new Date()) {
    if (!confirm(`确定解除 "${u.username}" 的禁言？`)) return
    muting.value = u.id
    try { await apiMuteUser(u.id, null); fetchUsers() } catch (e) { error.value = e.message; fetchUsers() } finally { muting.value = null }
  } else {
    const mins = prompt(`禁言 "${u.username}" 多少分钟？`, '10')
    if (!mins || isNaN(parseInt(mins))) return
    muting.value = u.id
    try { await apiMuteUser(u.id, parseInt(mins)); fetchUsers() } catch (e) { error.value = e.message; fetchUsers() } finally { muting.value = null }
  }
}

const roleName = (u) => {
  if (u.is_owner) return 'Owner'
  if (u.is_superadmin) return '超级管理员'
  if (u.is_admin) return '管理员'
  if (u.is_guest) return '访客'
  return '普通用户'
}
const roleBadge = (u) => {
  if (u.is_owner) return 'badge badge-warning badge-sm'
  if (u.is_superadmin) return 'badge badge-error badge-sm'
  if (u.is_admin) return 'badge badge-info badge-sm'
  if (u.is_guest) return 'badge badge-ghost badge-sm'
  return 'badge badge-outline badge-sm'
}

const fmt = (ts) => ts ? new Date(ts).toLocaleString() : '-'

fetchUsers()
</script>

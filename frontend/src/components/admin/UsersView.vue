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
      <div class="overflow-x-auto">
        <table class="table table-zebra">
          <thead>
            <tr>
              <th>ID</th><th>用户名</th><th>邮箱</th><th>状态</th><th>角色</th><th v-if="isSuperAdmin">升降</th><th>注册时间</th><th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="u in users" :key="u.id">
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
                <span v-if="u.is_guest" class="text-xs text-base-content/30">—</span>
                <input v-else type="checkbox" class="toggle toggle-sm" :checked="u.is_admin"
                  @change="toggleAdmin(u)" :disabled="toggling === u.id" />
              </td>
              <td class="text-sm text-base-content/50">{{ fmt(u.created_at) }}</td>
              <td>
                <button v-if="isSuperAdmin && !u.is_superadmin" @click="delUser(u)" class="btn btn-error btn-xs btn-outline" :disabled="deleting === u.id">
                  删除
                </button>
                <span v-else class="text-xs text-base-content/30">—</span>
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
import { ref, watch } from 'vue'
import { useAdmin } from '../../composables/useAdmin'
import { useAppState } from '../../composables/useAppState'
import { useAdminEvents } from '../../composables/useAdminEvents'

const { listUsers, deleteUser, toggleAdmin: apiToggleAdmin } = useAdmin()
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

const roleName = (u) => {
  if (u.is_superadmin) return '超级管理员'
  if (u.is_admin) return '管理员'
  if (u.is_guest) return '访客'
  return '普通用户'
}
const roleBadge = (u) => {
  if (u.is_superadmin) return 'badge badge-error badge-sm'
  if (u.is_admin) return 'badge badge-info badge-sm'
  if (u.is_guest) return 'badge badge-ghost badge-sm'
  return 'badge badge-outline badge-sm'
}

const fmt = (ts) => ts ? new Date(ts).toLocaleString() : '-'

fetchUsers()
</script>

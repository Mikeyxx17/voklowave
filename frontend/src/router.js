import { createRouter, createWebHashHistory } from 'vue-router'
import { useAppState } from './composables/useAppState'
import LoginView from './components/LoginView.vue'
import ChatLayout from './components/ChatLayout.vue'
import AdminLayout from './components/admin/AdminLayout.vue'
import DashboardView from './components/admin/DashboardView.vue'
import UsersView from './components/admin/UsersView.vue'
import ChannelsView from './components/admin/ChannelsView.vue'
import MessagesAuditView from './components/admin/MessagesAuditView.vue'
import AuditLogsView from './components/admin/AuditLogsView.vue'
import SettingsView from './components/admin/SettingsView.vue'

const routes = [
  { path: '/', component: ChatLayout, meta: { auth: true } },
  { path: '/login', component: LoginView },
  {
    path: '/admin',
    component: AdminLayout,
    meta: { auth: true, admin: true },
    children: [
      { path: '', redirect: '/admin/dashboard' },
      { path: 'dashboard', component: DashboardView },
      { path: 'users', component: UsersView },
      { path: 'channels', component: ChannelsView },
      { path: 'messages', component: MessagesAuditView },
      { path: 'logs', component: AuditLogsView },
      { path: 'settings', component: SettingsView },
    ],
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

router.beforeEach(async (to, _from) => {
  const { token, isJoined, isGuest } = useAppState()

  // 需要认证的路由：检查 token
  if (to.meta.auth && !token.value) {
    return '/login'
  }

  // 管理后台：校验管理员身份
  if (to.meta.admin && token.value) {
    try {
      const res = await fetch('/api/me', {
        headers: { Authorization: `Bearer ${token.value}` },
      })
      const data = await res.json()
      if (!data.is_admin) {
        // 非管理员直接回聊天页
        isJoined.value = true
        return '/'
      }
    } catch {
      return '/login'
    }
  }
})

export default router

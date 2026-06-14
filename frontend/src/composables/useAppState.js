import { computed, ref, watch } from 'vue'

// ── 模块级全局单例（所有组件共享同一份状态） ──
const username = ref('')
const email = ref('')
const token = ref(sessionStorage.getItem('voklowave-token') || '')
const isJoined = ref(false)
const currentChannel = ref(sessionStorage.getItem('voklowave-channel') || 'general')
const theme = ref(localStorage.getItem('voklowave-theme') || 'dark')
const showCreateModal = ref(false)
const authError = ref('')
const initializing = ref(true)
const pendingEmail = ref('')
const isGuestFlag = ref(false)

// ── 新增：用户资料字段 ──
const displayName = ref('')
const avatarUrl = ref('')
const bio = ref('')

// 是否为访客账号
const isGuest = computed(() => isGuestFlag.value)

// 主题变更 → localStorage + <html data-theme>
watch(theme, (val) => {
  localStorage.setItem('voklowave-theme', val)
  document.documentElement.setAttribute('data-theme', val)
}, { immediate: true })

// token 变更 → sessionStorage
watch(token, (val) => {
  if (val) {
    sessionStorage.setItem('voklowave-token', val)
  } else {
    sessionStorage.removeItem('voklowave-token')
  }
})

// 当前频道变更 → sessionStorage
watch(currentChannel, (val) => {
  sessionStorage.setItem('voklowave-channel', val)
})

// ── 新增：资料字段 → sessionStorage 持久化（可选，刷新后保留编辑结果） ──
watch(displayName, (val) => {
  if (val) sessionStorage.setItem('voklowave-display-name', val)
  else sessionStorage.removeItem('voklowave-display-name')
})
watch(avatarUrl, (val) => {
  if (val) sessionStorage.setItem('voklowave-avatar-url', val)
  else sessionStorage.removeItem('voklowave-avatar-url')
})

// 页面加载时尝试用 sessionStorage 中的 token 恢复会话
const initAuth = async () => {
  // 恢复持久化的资料（在 /api/me 返回前先用本地缓存）
  displayName.value = sessionStorage.getItem('voklowave-display-name') || ''
  avatarUrl.value = sessionStorage.getItem('voklowave-avatar-url') || ''

  const saved = sessionStorage.getItem('voklowave-token')
  if (!saved) {
    initializing.value = false
    return
  }
  try {
    const res = await fetch('/api/me', {
      headers: { Authorization: `Bearer ${saved}` },
    })
      if (res.ok) {
        const data = await res.json()
        token.value = saved
        username.value = data.username
        email.value = data.email
        isGuestFlag.value = data.is_guest || false
        // ── 新增：从服务器恢复资料字段，服务端数据优先 ──
        if (data.display_name) displayName.value = data.display_name
        if (data.avatar_url) avatarUrl.value = data.avatar_url
        isJoined.value = true
        requestNotify()
      } else {
        sessionStorage.removeItem('voklowave-token')
        token.value = ''
      }
  } catch {
    // 网络不通时保留 token，刷新页面后重试
  }
  initializing.value = false
}

// ── 请求桌面通知权限 ──
const requestNotify = () => {
  if ('Notification' in window && Notification.permission === 'default') {
    Notification.requestPermission()
  }
}

initAuth()

/**
 * 全局认证与设置状态管理
 *
 * 模块级 ref 保证跨组件共享一份数据，无需 provide/inject。
 * 提供 login / register / verifyEmail / guest / logout / updateProfile 等方法。
 */
export function useAppState() {

  /**
   * 注册新账号。
   * 成功后后端会发送验证邮件，前端切换到验证码输入界面。
   */
  const register = async (regUsername, regEmail, password) => {
    authError.value = ''
    try {
      const res = await fetch('/api/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: regUsername, email: regEmail, password }),
      })
      if (res.status === 201) {
        return { ok: true }
      }
      const body = await res.text()
      authError.value = body || '注册失败，请稍后重试'
      return { ok: false, error: body || '注册失败' }
    } catch {
      authError.value = '网络错误，请检查连接'
      return { ok: false, error: '网络错误' }
    }
  }

  /**
   * 邮箱 + 密码登录。
   * 未验证邮箱的账号会返回 needVerify，引导至验证界面。
   */
  const login = async (loginEmail, password) => {
    authError.value = ''
    try {
      const res = await fetch('/api/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: loginEmail, password }),
      })
      if (res.ok) {
        const data = await res.json()
        token.value = data.token
        username.value = data.username
        email.value = loginEmail
        isGuestFlag.value = data.is_guest || false
        // ── 新增：登录时同步资料字段 ──
        if (data.display_name) displayName.value = data.display_name
        if (data.avatar_url) avatarUrl.value = data.avatar_url
        isJoined.value = true
        requestNotify()  // 新增：登录时请求通知权限
        return { ok: true }
      }
      const body = await res.text()
      if (res.status === 403 && body.includes('尚未通过邮箱验证')) {
        pendingEmail.value = loginEmail
        return { ok: false, needVerify: true, error: body }
      }
      authError.value = body || '登录失败，请稍后重试'
      return { ok: false, error: body || '登录失败' }
    } catch {
      authError.value = '网络错误，请检查连接'
      return { ok: false, error: '网络错误' }
    }
  }

  /** 提交 6 位验证码激活账号 */
  const verifyEmail = async (targetEmail, code) => {
    authError.value = ''
    try {
      const res = await fetch('/api/verify_email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: targetEmail, code }),
      })
      const body = await res.text()
      if (res.ok) {
        return { ok: true, message: body }
      }
      return { ok: false, error: body }
    } catch {
      return { ok: false, error: '网络错误，请检查连接' }
    }
  }

  /** 重新发送验证邮件（每日 3 次，60s 冷却） */
  const resendVerification = async (targetEmail) => {
    authError.value = ''
    try {
      const res = await fetch('/api/resend_verification', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: targetEmail }),
      })
      const body = await res.text()
      if (res.ok) {
        return { ok: true, message: body }
      }
      return { ok: false, error: body }
    } catch {
      return { ok: false, error: '网络错误，请检查连接' }
    }
  }

  /** （旧版）纯本地快速加入，不走后端认证 */
  const join = (name) => {
    username.value = name.trim()
    if (username.value) {
      token.value = ''
      email.value = ''
      isGuestFlag.value = false
      isJoined.value = true
    }
  }

  /** 快速体验：调用后端访客接口，获得临时 JWT */
  const quickExperience = async () => {
    authError.value = ''
    try {
      const res = await fetch('/api/guest_login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      })

      if (res.ok) {
        const data = await res.json()
        token.value = data.token
        username.value = data.username
        isGuestFlag.value = true
        currentChannel.value = 'general'  // 访客强制回到 general 频道
        isJoined.value = true
        requestNotify()  // 新增：访客登录时请求通知权限
        return { ok: true }
      } else {
        const body = await res.text()
        authError.value = body || '访客登录失败'
        return { ok: false, error: body || '访客登录失败' }
      }
    } catch {
      authError.value = '网络错误，请检查连接'
      return { ok: false, error: '网络错误' }
    }
  }

  /** 退出登录：清除认证状态但保留主题偏好 */
  const logout = () => {
    token.value = ''
    username.value = ''
    email.value = ''
    isGuestFlag.value = false
    isJoined.value = false
    displayName.value = ''      // 新增
    avatarUrl.value = ''        // 新增
    bio.value = ''              // 新增
    currentChannel.value = 'general'   // 新增：退出时重置频道，防止下次登录残留旧频道
    sessionStorage.removeItem('voklowave-token')
    sessionStorage.removeItem('voklowave-channel')
    sessionStorage.removeItem('voklowave-display-name')   // 新增
    sessionStorage.removeItem('voklowave-avatar-url')     // 新增
  }

  /** 切换当前频道 */
  const switchChannel = (name) => {
    currentChannel.value = name
  }

  // ── 新增：忘记密码流程 ──
  const mode = ref('login')

  const forgotPassword = async (targetEmail) => {
    authError.value = ''
    try {
      const res = await fetch('/api/forgot_password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: targetEmail }),
      })
      const body = await res.text()
      if (res.ok) {
        return { ok: true, message: body }
      }
      return { ok: false, error: body }
    } catch {
      return { ok: false, error: '网络错误，请检查连接' }
    }
  }

  const resetPassword = async (targetEmail, code, newPassword) => {
    authError.value = ''
    try {
      const res = await fetch('/api/reset_password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: targetEmail, code, new_password: newPassword }),
      })
      const body = await res.text()
      if (res.ok) {
        return { ok: true, message: body }
      }
      return { ok: false, error: body }
    } catch {
      return { ok: false, error: '网络错误，请检查连接' }
    }
  }

  // ── 新增：用户资料编辑 ──
  /**
   * 更新当前用户的个人资料。
   * @param {Object} fields - { display_name?, avatar_url?, bio? } 仅传要更新的字段
   */
  const saveProfile = async (fields) => {
    authError.value = ''
    try {
      const res = await fetch('/api/me', {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token.value}`,
        },
        body: JSON.stringify(fields),
      })
      if (res.ok) {
        const data = await res.json()
        displayName.value = data.display_name || ''
        avatarUrl.value = data.avatar_url || ''
        bio.value = data.bio || ''
        return { ok: true, data }
      }
      const body = await res.text()
      authError.value = body
      return { ok: false, error: body }
    } catch {
      authError.value = '网络错误，请检查连接'
      return { ok: false, error: '网络错误' }
    }
  }

  return {
    username,
    email,
    token,
    isJoined,
    currentChannel,
    theme,
    showCreateModal,
    authError,
    initializing,
    pendingEmail,
    isGuest,
    // ── 新增：资料字段 ──
    displayName,
    avatarUrl,
    bio,
    mode,
    register,
    login,
    verifyEmail,
    resendVerification,
    join,
    quickExperience,
    logout,
    switchChannel,
    forgotPassword,
    resetPassword,
    saveProfile,  // 新增
  }
}

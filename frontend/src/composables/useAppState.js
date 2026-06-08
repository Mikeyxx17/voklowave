import { computed, ref, watch } from 'vue'

// ── 全局单例状态 ──
const username = ref('')
const email = ref('')
const token = ref(sessionStorage.getItem('voklowave-token') || '')
const isJoined = ref(false)
const currentChannel = ref(sessionStorage.getItem('voklowave-channel') || 'general')
const theme = ref(localStorage.getItem('voklowave-theme') || 'dark')
const showCreateModal = ref(false)
const authError = ref('')
const initializing = ref(true)
const pendingEmail = ref('')   // 记住需要验证的邮箱，方便登录后取值

const isGuest = computed(() => username.value.startsWith('Guest_'))

// 主题持久化 + 应用到 <html>
watch(theme, (val) => {
  localStorage.setItem('voklowave-theme', val)
  document.documentElement.setAttribute('data-theme', val)
}, { immediate: true })

// token 持久化
watch(token, (val) => {
  if (val) {
    sessionStorage.setItem('voklowave-token', val)
  } else {
    sessionStorage.removeItem('voklowave-token')
  }
})

// 当前频道持久化
watch(currentChannel, (val) => {
  sessionStorage.setItem('voklowave-channel', val)
})

// ── 启动时尝试恢复会话 ──
const initAuth = async () => {
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
      isJoined.value = true
    } else {
      sessionStorage.removeItem('voklowave-token')
      token.value = ''
    }
  } catch {
    // 网络不通时不清除 token，下次刷新再试
  }
  initializing.value = false
}

initAuth()

export function useAppState() {
  // ── 注册 ──
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

  // ── 登录 ──
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
        isJoined.value = true
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

  // ── 邮箱验证 ──
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

  // ── 重新发送验证码 ──
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

  // ── 快速体验（原本地保留版本，按需可删）──
  const join = (name) => {
    username.value = name.trim()
    if (username.value) {
      token.value = ''
      email.value = ''
      isJoined.value = true
    }
  }

  // ── 快速体验（连接后端API新版）──
  const quickExperience = async () => {
    authError.value = ''
    try {
      // 调用后端游客专用接口
      const res = await fetch('/api/guest_login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      })

      if (res.ok) {
        const data = await res.json()

        // 像登录成功一样同步状态
        token.value = data.token
        username.value = data.username

        // 标记已加入聊天
        isJoined.value = true
        return { ok: true }
      } else {
        authError.value = '快速体验生成失败，请重试'
        return { ok: false, error: '快速体验失败' }
      }
    } catch (err) {
      authError.value = '网络错误，请检查连接'
      return { ok: false, error: '网络错误' }
    }
  }

  // ── 登出 ──
  const logout = () => {
    token.value = ''
    username.value = ''
    email.value = ''
    isJoined.value = false
    currentChannel.value = 'general'
  }

  const switchChannel = (channel) => {
    currentChannel.value = channel
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
    isGuest,
    pendingEmail,
    login,
    register,
    verifyEmail,
    resendVerification,
    join,
    logout,
    switchChannel,
    quickExperience
  }
}
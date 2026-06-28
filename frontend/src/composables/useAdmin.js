import { useAppState } from './useAppState'

const BASE = '/api/admin'

/** 通用请求封装 */
async function api(path, options = {}) {
  const { token, logout } = useAppState()
  const res = await fetch(`${BASE}${path}`, {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token.value}`,
      ...options.headers,
    },
    ...options,
  })
  if (res.status === 401) {
    logout()
    throw new Error('登录已过期，请重新登录')
  }
  const text = await res.text()
  let data = null
  try { data = JSON.parse(text) } catch { data = text }
  if (!res.ok) throw new Error(typeof data === 'string' ? data : JSON.stringify(data))
  return data
}

export function useAdmin() {
  return {
    dashboard:  ()                    => api('/dashboard'),
    listUsers:  (q = '', page = 0)    => api(`/users?q=${encodeURIComponent(q)}&page=${page}`),
    deleteUser: (id)                  => api(`/users/${id}`, { method: 'DELETE' }),
    toggleAdmin:(id)                  => api(`/users/${id}/toggle-admin`, { method: 'PATCH' }),
    muteUser:  (id, mins = null)      => api(`/users/${id}/mute`, { method: 'PATCH', body: JSON.stringify({ duration_minutes: mins }) }),
    listChannels:()                   => api('/channels'),
    deleteChannel:(id)                => api(`/channels/${id}`, { method: 'DELETE' }),
    auditMessages:(q = '', page = 0)  => api(`/messages?q=${encodeURIComponent(q)}&page=${page}`),
    deleteMessage:(id)                => api(`/messages/${id}`, { method: 'DELETE' }),
    batchDeleteMessages:(ids)         => api('/messages/batch-delete', { method: 'POST', body: JSON.stringify({ ids }) }),
    batchDeleteUsers:(ids)            => api('/users/batch-delete', { method: 'POST', body: JSON.stringify({ ids }) }),
    batchToggleAdmin:(ids)            => api('/users/batch-toggle-admin', { method: 'POST', body: JSON.stringify({ ids }) }),
    auditLogs:   (page = 0)           => api(`/audit-logs?page=${page}`),
  }
}

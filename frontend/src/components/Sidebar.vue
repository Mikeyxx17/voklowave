<template>
  <aside class="w-[270px] h-full flex flex-col bg-base-200/80 border-r border-base-300">
    <!-- 品牌区 -->
    <div class="px-4 py-4 border-b border-base-300/60">
      <div class="flex items-center gap-3">
        <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-indigo-500 via-purple-500 to-pink-500 flex items-center justify-center shadow-md shrink-0">
          <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.625 9.75a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H8.25m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H12m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0h-.375m-13.5 3.01c0 1.6 1.123 2.994 2.707 3.227 1.087.16 2.185.283 3.293.369V21l4.184-4.183a1.14 1.14 0 01.778-.332 48.294 48.294 0 005.83-.498c1.585-.233 2.708-1.626 2.708-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0012 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018z" />
          </svg>
        </div>
        <div class="min-w-0">
          <h2 class="text-sm font-bold text-base-content">VokloWave</h2>
          <p class="text-[10px] text-base-content/40">团队即时通讯</p>
        </div>
      </div>
    </div>

    <!-- 频道标题 -->
    <div class="flex items-center justify-between px-4 pt-4 pb-2">
      <span class="text-[11px] font-semibold uppercase tracking-[0.15em] text-base-content/45">频道</span>
      <button
        v-if="!isGuest"
        class="btn btn-ghost btn-xs btn-square hover:bg-primary/20"
        @click="showCreateModal = true"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>

    <!-- 频道列表 -->
    <div class="flex-1 overflow-y-auto px-3 pb-2 space-y-0.5">
      <div
        v-for="ch in channels"
        :key="ch.id"
        class="flex items-center gap-2.5 px-3 py-2 rounded-lg cursor-pointer transition-all duration-150 group"
        :class="ch.name === currentChannel
          ? 'bg-primary/15 text-primary font-semibold'
          : 'text-base-content/65 hover:bg-base-100/60 hover:text-base-content'"
        @click="selectChannel(ch.name)"
      >
        <span class="text-lg shrink-0 font-medium"
          :class="ch.name === currentChannel ? 'text-primary' : 'text-base-content/25 group-hover:text-base-content/50'"
        >#</span>
        <span class="truncate text-[13px]">{{ ch.name }}</span>
        <span
          v-if="ch.name === currentChannel"
          class="ml-auto w-1.5 h-1.5 rounded-full bg-primary animate-pulse shrink-0"
        />
      </div>

      <!-- 空状态 -->
      <div v-if="channels.length === 0 && !loading" class="flex flex-col items-center py-10 text-base-content/25 gap-2">
        <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
        </svg>
        <p class="text-xs">暂无频道</p>
      </div>
    </div>

    <!-- 底部：主题 + 用户 -->
    <div class="border-t border-base-300/60 bg-base-300/40">
      <!-- 主题切换 -->
      <div class="px-4 pt-3 pb-2">
        <div class="dropdown dropdown-top dropdown-end w-full">
          <div tabindex="0" role="button" class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs text-base-content/60 hover:bg-base-100/60 transition-colors cursor-pointer w-full">
            <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z" />
            </svg>
            <span class="truncate">{{ themeLabel }}</span>
            <svg class="w-3 h-3 ml-auto shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.5 15.75l7.5-7.5 7.5 7.5" />
            </svg>
          </div>
          <ul tabindex="0" class="dropdown-content z-50 menu p-1.5 shadow-xl bg-base-200 rounded-xl border border-base-300/50 w-48 mb-1">
            <li v-for="t in themes" :key="t.value">
              <a
                class="text-xs py-2 rounded-lg"
                :class="theme === t.value ? 'bg-primary/15 text-primary font-semibold' : ''"
                @click="selectTheme(t.value)"
              >
                <span class="w-3 h-3 rounded-full border-2 shrink-0" :style="{ background: t.color, borderColor: t.border }" />
                {{ t.label }}
              </a>
            </li>
          </ul>
        </div>
      </div>

      <!-- 用户信息 -->
      <div class="px-4 py-3">
        <!-- ── 新增：整个用户区可点击，打开资料编辑弹窗 ── -->
        <div class="flex items-center gap-3 cursor-pointer hover:bg-base-100/50 rounded-lg p-1.5 -m-1.5 transition-colors" @click="profileModal?.open()">
          <div class="avatar placeholder shrink-0">
            <div class="w-8 rounded-full !rounded-btn text-xs font-bold text-white" :style="{ background: avatarBg }">
              <!-- ── 新增：有头像链接时显示图片 ── -->
              <img v-if="avatarUrl" :src="avatarUrl" class="w-full h-full object-cover !rounded-btn" @error="avatarUrl = ''" />
              {{ username ? username.charAt(0).toUpperCase() : '?' }}
            </div>
          </div>
          <div class="min-w-0 flex-1">
            <!-- ── 新增：优先显示昵称，回退用户名 ── -->
            <p class="text-xs font-semibold text-base-content truncate">{{ displayName || username || '未登录' }}</p>
            <p v-if="bio" class="text-[10px] text-base-content/40 truncate mt-0.5">{{ bio }}</p>
            <p class="text-[10px] text-success/80 flex items-center gap-1">
              <span class="w-1.5 h-1.5 rounded-full bg-success" />
              在线
            </p>
          </div>
        </div>
        <!-- ── 退出按钮移到用户区下方，独立点击 ── -->
        <button
          v-if="token"
          class="btn btn-ghost btn-xs text-base-content/30 hover:text-error mt-1.5 w-full justify-center"
          title="退出登录"
          @click="logout"
        >退出登录</button>
      </div>
    </div>

    <!-- ── 资料编辑弹窗（Teleport 到 body，避免影响侧边栏布局） ── -->
    <Teleport to="body">
      <ProfileEditModal ref="profileModal" />
    </Teleport>
  </aside>
</template>

<script setup>
import { computed, ref } from 'vue'
import { useAppState } from '../composables/useAppState'
import { useChannels } from '../composables/useChannels'
import ProfileEditModal from './ProfileEditModal.vue'

const { username, token, currentChannel, theme, switchChannel, showCreateModal, logout, isGuest, displayName, avatarUrl, bio } = useAppState()
const { channels, loading } = useChannels()

// ── 新增：资料编辑弹窗引用 ──
const profileModal = ref(null)

const selectChannel = (name) => {
  const toggle = document.getElementById('sidebar-toggle')
  if (toggle) toggle.checked = false
  switchChannel(name)
}

const selectTheme = (value) => {
  theme.value = value
  document.activeElement?.blur()
}

const themes = [
  { value: 'dark', label: '深色', color: '#1d232a', border: '#4b5563' },
  { value: 'light', label: '浅色', color: '#ffffff', border: '#d1d5db' },
  { value: 'cyberpunk', label: '赛博朋克', color: '#ffee00', border: '#ff00ff' },
  { value: 'cupcake', label: '纸杯蛋糕', color: '#faf7f5', border: '#e8d5c4' },
  { value: 'synthwave', label: '合成波', color: '#2d1b69', border: '#e779c1' },
  { value: 'nord', label: '北欧风', color: '#3b4252', border: '#88c0d0' },
  { value: 'sunset', label: '日落', color: '#1e1e2e', border: '#f38ba8' },
  { value: 'winter', label: '冬季', color: '#e0e7ff', border: '#a5b4fc' },
  { value: 'coffee', label: '咖啡', color: '#20161f', border: '#8b5a2b' },
  { value: 'lemonade', label: '柠檬水', color: '#fcf9e8', border: '#e9c46a' },
  { value: 'luxury', label: '奢华金', color: '#09090b', border: '#ca8a04' },
  { value: 'business', label: '商务', color: '#1e293b', border: '#64748b' },
  { value: 'autumn', label: '秋季', color: '#fef3c7', border: '#d97706' },
  { value: 'dim', label: '暗紫', color: '#2a1e3a', border: '#a78bfa' },
]

const themeLabel = computed(() => {
  const t = themes.find(t => t.value === theme.value)
  return t ? t.label : '深色'
})

const avatarBg = computed(() => {
  const colors = [
    '#6366f1', '#8b5cf6', '#d946ef', '#ec4899',
    '#f43f5e', '#f97316', '#eab308', '#22c55e',
    '#14b8a6', '#06b6d4', '#3b82f6',
  ]
  let hash = 0
  const name = username.value || '?'
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash)
  }
  return colors[Math.abs(hash) % colors.length]
})
</script>
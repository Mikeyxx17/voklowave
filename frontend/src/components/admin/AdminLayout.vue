<template>
  <div class="flex min-h-screen bg-base-200">
    <!-- 侧边栏 -->
    <aside class="w-56 bg-base-100 border-r border-base-300 flex flex-col shrink-0">
      <div class="p-4 border-b border-base-300">
        <h2 class="text-lg font-bold">⚙️ 管理后台</h2>
        <p class="text-xs text-base-content/50 mt-1">{{ username }}</p>
      </div>
      <nav class="flex-1 p-2 space-y-1">
        <router-link to="/admin/dashboard" class="block px-3 py-2 rounded hover:bg-base-200 transition-colors"
          :class="$route.path === '/admin/dashboard' ? 'bg-primary/10 text-primary font-medium' : ''">
          📊 仪表盘
        </router-link>
        <router-link to="/admin/users" class="block px-3 py-2 rounded hover:bg-base-200 transition-colors"
          :class="$route.path === '/admin/users' ? 'bg-primary/10 text-primary font-medium' : ''">
          👥 用户管理
        </router-link>
        <router-link to="/admin/channels" class="block px-3 py-2 rounded hover:bg-base-200 transition-colors"
          :class="$route.path === '/admin/channels' ? 'bg-primary/10 text-primary font-medium' : ''">
          💬 频道管理
        </router-link>
        <router-link to="/admin/messages" class="block px-3 py-2 rounded hover:bg-base-200 transition-colors"
          :class="$route.path === '/admin/messages' ? 'bg-primary/10 text-primary font-medium' : ''">
          📝 消息审计
        </router-link>
        <router-link to="/admin/logs" class="block px-3 py-2 rounded hover:bg-base-200 transition-colors"
          :class="$route.path === '/admin/logs' ? 'bg-primary/10 text-primary font-medium' : ''">
          📋 操作日志
        </router-link>
        <router-link to="/admin/settings" class="block px-3 py-2 rounded hover:bg-base-200 transition-colors"
          :class="$route.path === '/admin/settings' ? 'bg-primary/10 text-primary font-medium' : ''">
          🔧 系统设置
        </router-link>
      </nav>
      <div class="p-4 border-t border-base-300">
        <router-link to="/" class="btn btn-ghost btn-sm w-full">← 返回聊天</router-link>
      </div>
    </aside>

    <!-- 内容区 -->
    <main class="flex-1 p-6 overflow-auto">
      <router-view />
    </main>
  </div>
</template>

<script setup>
import { useAppState } from '../../composables/useAppState'
const { username } = useAppState()
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold mb-6">🔧 系统设置</h1>

    <div class="space-y-4 max-w-xl">
      <div v-if="isSuperAdmin" class="card bg-base-100 shadow">
        <div class="card-body">
          <h3 class="font-bold text-lg">超级管理员信息</h3>
          <div class="text-sm space-y-2 mt-2">
            <p><span class="text-base-content/60">用户名：</span> {{ username }}</p>
            <p><span class="text-base-content/60">角色：</span> Super Admin（超级管理员）</p>
            <p class="text-base-content/50">超级管理员账号在系统初始化时创建，不可注册、不可删除、不可降级。</p>
          </div>
        </div>
      </div>

      <div class="card bg-base-100 shadow">
        <div class="card-body">
          <h3 class="font-bold text-lg">管理员层级说明</h3>
          <div class="text-sm space-y-2 mt-2 text-base-content/70">
            <p>🔴 <strong>超级管理员</strong>（{{ username }}）：系统唯一，可升降普通管理员。</p>
            <p>🟡 <strong>普通管理员</strong>：由超级管理员授予，可管理用户/频道/消息，但不能升降其他管理员。</p>
            <p>⚪ <strong>普通用户</strong>：无管理权限。</p>
          </div>
        </div>
      </div>

      <div class="card bg-base-100 shadow">
        <div class="card-body">
          <h3 class="font-bold text-lg">环境信息</h3>
          <div class="text-sm space-y-2 mt-2">
            <p><span class="text-base-content/60">后端地址：</span> <code class="bg-base-200 px-2 py-0.5 rounded text-xs">http://127.0.0.1:3000</code></p>
            <p><span class="text-base-content/60">前端地址：</span> <code class="bg-base-200 px-2 py-0.5 rounded text-xs">http://localhost:5173</code></p>
            <p><span class="text-base-content/60">数据库：</span> PostgreSQL</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { useAppState } from '../../composables/useAppState'
const { username, isSuperAdmin } = useAppState()
</script>

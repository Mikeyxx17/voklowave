<template>
  <dialog ref="modalEl" class="modal" @dblclick.prevent="onBackdropDblClick">
    <div class="modal-box max-w-sm select-none">
      <h3 class="text-lg font-bold mb-4 select-none">编辑个人资料</h3>

      <!-- 头像区 -->
      <div class="flex flex-col items-center mb-4">
        <div class="avatar mb-3">
          <div class="w-20 rounded-full ring ring-primary/20 ring-offset-base-100 ring-offset-2">
            <img v-if="form.avatar_url" :src="form.avatar_url" alt="头像" @error="onAvatarError" />
            <div v-else class="w-full h-full flex items-center justify-center text-2xl font-bold text-white" :style="{ background: avatarBg }">
              {{ userInitial }}
            </div>
          </div>
        </div>
      </div>

      <!-- 昵称 -->
      <div class="form-control mb-3">
        <label class="label pb-1">
          <span class="label-text text-xs font-semibold select-none">昵称</span>
          <span class="label-text-alt text-base-content/40">{{ (form.display_name || '').length }}/50</span>
        </label>
        <input
          v-model="form.display_name"
          class="input input-bordered input-sm bg-base-200/60 select-auto"
          placeholder="输入昵称"
          maxlength="50"
        />
      </div>

      <!-- 头像链接 -->
      <div class="form-control mb-3">
        <label class="label pb-1">
          <span class="label-text text-xs font-semibold select-none">头像链接</span>
        </label>
        <input
          v-model="form.avatar_url"
          class="input input-bordered input-sm bg-base-200/60 select-auto"
          placeholder="https://..."
          type="url"
        />
      </div>

      <!-- 个性签名 -->
      <div class="form-control mb-4">
        <label class="label pb-1">
          <span class="label-text text-xs font-semibold select-none">个性签名</span>
          <span class="label-text-alt text-base-content/40">{{ (form.bio || '').length }}/500</span>
        </label>
        <textarea
          v-model="form.bio"
          class="textarea textarea-bordered textarea-sm bg-base-200/60 h-20 resize-none select-auto"
          placeholder="写点什么..."
          maxlength="500"
        />
      </div>

      <!-- 操作按钮 -->
      <div class="modal-action mt-0">
        <button class="btn btn-ghost btn-sm" @click="close">取消</button>
        <button
          class="btn btn-primary btn-sm"
          :disabled="saving"
          @click="save"
        >
          <span v-if="saving" class="loading loading-spinner loading-sm"></span>
          <span v-else>保存</span>
        </button>
      </div>
    </div>
  </dialog>
</template>

<script setup>
import { ref, reactive, computed, watch, nextTick } from 'vue'
import { useAppState } from '../composables/useAppState'

const { username, displayName, avatarUrl, bio, saveProfile } = useAppState()

const modalEl = ref(null)
const saving = ref(false)

// 表单数据：从当前状态初始化
const form = reactive({
  display_name: '',
  avatar_url: '',
  bio: '',
})

// ── 原始值：用于判断是否已修改 ──
const original = reactive({ display_name: '', avatar_url: '', bio: '' })

// ── 是否已修改表单 ──
const isModified = computed(() => {
  return form.display_name !== original.display_name
    || form.avatar_url !== original.avatar_url
    || form.bio !== original.bio
})

// 首字母（纯文本回头像兜底）
const userInitial = computed(() => username.value?.charAt(0).toUpperCase() || '?')

// 头像背景色（与 Sidebar 一致）
const avatarBg = computed(() => {
  const colors = [
    '#6366f1', '#8b5cf6', '#d946ef', '#ec4899',
    '#f43f5e', '#f97316', '#eab308', '#22c55e',
    '#14b8a6', '#06b6d4', '#3b82f6',
  ]
  let hash = 0
  const name = username.value || ''
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash)
  }
  return colors[Math.abs(hash) % colors.length]
})

// 头像加载失败 → 清空链接，回退到首字母圆
const onAvatarError = (e) => {
  form.avatar_url = ''
  e.target.style.display = 'none'
}

// 打开弹窗 → 初始化表单为当前值，记录原始值
const open = () => {
  form.display_name = displayName.value || ''
  form.avatar_url = avatarUrl.value || ''
  form.bio = bio.value || ''
  original.display_name = form.display_name
  original.avatar_url = form.avatar_url
  original.bio = form.bio
  modalEl.value?.showModal()
}

const close = () => {
  modalEl.value?.close()
}

// ── 双击空白处：仅未修改时关闭 ──
const onBackdropDblClick = (e) => {
  if (e.target === modalEl.value && !isModified.value) {
    close()
  }
}

// 保存
const save = async () => {
  saving.value = true
  const result = await saveProfile({
    // 用 ?? 而非 ||：空字符串代表"清空"，应发送空串而非 null
    display_name: form.display_name ?? null,
    avatar_url: form.avatar_url ?? null,
    bio: form.bio ?? null,
  })
  saving.value = false
  if (result.ok) {
    close()
  }
}

// 暴露 open 方法给父组件
defineExpose({ open })
</script>

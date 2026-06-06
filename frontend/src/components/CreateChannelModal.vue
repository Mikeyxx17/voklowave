<template>
  <div v-if="show" class="modal modal-open" @click.self="$emit('close')">
    <div class="modal-box w-96">
      <h3 class="text-lg font-bold flex items-center gap-2">
        <svg class="w-5 h-5 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        创建频道
      </h3>
      <p class="text-xs text-base-content/50 mt-1">频道是团队沟通的专属空间，围绕某个话题展开讨论。</p>

      <div class="form-control mt-4">
        <label class="label pb-1">
          <span class="label-text text-xs font-medium">频道名称</span>
        </label>
        <label class="input input-bordered flex items-center gap-2 focus-within:border-primary/60 transition-colors">
          <span class="text-lg text-base-content/35 font-bold">#</span>
          <input
            ref="inputEl"
            v-model="name"
            type="text"
            class="grow text-sm bg-transparent outline-none"
            placeholder="例如：项目讨论"
            maxlength="30"
            @keyup.enter="doCreate"
          />
        </label>
      </div>

      <div v-if="error" class="text-xs text-error mt-2 flex items-center gap-1">
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        {{ error }}
      </div>

      <div class="modal-action mt-5">
        <button class="btn btn-ghost btn-sm" @click="$emit('close')">取消</button>
        <button
          class="btn btn-primary btn-sm gap-2"
          :disabled="!name.trim() || creating"
          @click="doCreate"
        >
          <span v-if="creating" class="loading loading-spinner loading-xs" />
          <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          创建
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick } from 'vue'
import { useChannels } from '../composables/useChannels'

const props = defineProps({
  show: Boolean,
})

const emit = defineEmits(['close', 'created'])

const { createChannel } = useChannels()
const name = ref('')
const error = ref('')
const creating = ref(false)
const inputEl = ref(null)

watch(() => props.show, (val) => {
  if (val) {
    name.value = ''
    error.value = ''
    nextTick(() => inputEl.value?.focus())
  }
})

const doCreate = async () => {
  const n = name.value.trim()
  if (!n) return

  creating.value = true
  error.value = ''

  const ok = await createChannel(n)
  creating.value = false

  if (ok) {
    emit('created')
  } else {
    error.value = '创建失败，频道可能已存在'
  }
}
</script>

<template>
    <Teleport to="body">
        <Transition name="modal-fade">
            <div
                v-if="show"
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/10"
                @click.self="$emit('close')"
            >
                <div
                    class="bg-base-100 rounded-2xl border border-base-200 shadow-2xl w-full max-w-sm mx-4 p-8 overflow-hidden modal-box-inner"
                >
                    <h3
                        class="text-xl font-bold flex items-center gap-2 text-base-content select-none"
                    >
                        <svg
                            class="w-6 h-6 text-primary"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M12 4v16m8-8H4"
                            />
                        </svg>
                        创建频道
                    </h3>
                    <p
                        class="text-xs text-base-content/50 mt-2 mb-6 select-none"
                    >
                        频道是团队沟通的专属空间，围绕某个话题展开讨论。
                    </p>

                    <div class="form-control">
                        <label class="label py-1">
                            <span
                                class="label-text text-xs font-bold uppercase tracking-wider text-base-content/70 select-none"
                                >频道名称</span
                            >
                        </label>
                        <div class="relative flex items-center">
                            <span
                                class="absolute left-4 text-base-content/30 font-bold select-none"
                                >#</span
                            >
                            <input
                                ref="inputEl"
                                v-model="name"
                                type="text"
                                class="input w-full bg-base-200/50 border-transparent focus:bg-base-100 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 transition-all rounded-xl pl-9 text-sm"
                                placeholder="例如：项目讨论"
                                maxlength="30"
                                @keyup.enter="doCreate"
                            />
                        </div>
                    </div>

                    <div
                        v-if="error"
                        class="text-xs text-error mt-3 flex items-center gap-1.5 font-medium bg-error/10 p-2 rounded-lg"
                    >
                        <svg
                            class="w-4 h-4 shrink-0"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"
                            />
                        </svg>
                        {{ error }}
                    </div>

                    <div class="flex justify-end gap-3 mt-8">
                        <button
                            class="btn btn-ghost rounded-xl text-sm font-medium hover:bg-base-200/50"
                            @click="$emit('close')"
                        >
                            取消
                        </button>
                        <button
                            class="btn btn-primary rounded-xl text-sm font-medium gap-2 shadow-sm shadow-primary/20 hover:shadow-md hover:shadow-primary/30 transition-shadow"
                            :disabled="!name.trim() || creating"
                            @click="doCreate"
                        >
                            <span
                                v-if="creating"
                                class="loading loading-spinner loading-xs"
                            />
                            <svg
                                v-else
                                class="w-4 h-4"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M5 13l4 4L19 7"
                                />
                            </svg>
                            创建
                        </button>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<script setup>
import { ref, watch, nextTick } from "vue";
import { useChannels } from "../composables/useChannels";

const props = defineProps({
    show: Boolean,
});

const emit = defineEmits(["close", "created"]);

const { createChannel } = useChannels();
const name = ref("");
const error = ref("");
const creating = ref(false);
const inputEl = ref(null);

watch(
    () => props.show,
    (val) => {
        if (val) {
            name.value = "";
            error.value = "";
            nextTick(() => inputEl.value?.focus());
        }
    },
);

const doCreate = async () => {
    const n = name.value.trim();
    if (!n) return;

    creating.value = true;
    error.value = "";

    const ok = await createChannel(n);
    creating.value = false;

    if (ok) {
        emit("created");
    } else {
        error.value = "创建失败，频道可能已存在";
    }
};
</script>

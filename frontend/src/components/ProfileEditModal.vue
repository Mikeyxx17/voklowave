<template>
    <Teleport to="body">
        <Transition name="modal-fade">
            <div
                v-if="show"
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/10"
                @click.self="onBackdropClick"
            >
                <div
                    class="bg-base-100 rounded-2xl shadow-2xl border border-base-200 w-full max-w-sm mx-4 p-8 modal-box-inner"
                >
                    <h3
                        class="text-xl font-bold mb-6 text-center text-base-content select-none"
                    >
                        编辑资料
                    </h3>

                    <!-- 头像区 -->
                    <div class="flex flex-col items-center mb-6">
                        <div class="avatar mb-2 group relative cursor-pointer">
                            <div
                                class="w-24 h-24 rounded-[2rem] ring-4 ring-primary/20 ring-offset-base-100 ring-offset-4 shadow-lg transition-transform duration-300 group-hover:scale-105 group-hover:ring-primary/40"
                            >
                                <img
                                    v-if="form.avatar_url"
                                    :src="form.avatar_url"
                                    alt="头像"
                                    class="object-cover"
                                    @error="onAvatarError"
                                />
                                <div
                                    v-else
                                    class="w-full h-full flex items-center justify-center text-3xl font-extrabold text-white"
                                    :style="{ background: avatarBg }"
                                >
                                    {{ userInitial }}
                                </div>
                            </div>
                            <div
                                class="absolute -bottom-1 -right-1 w-6 h-6 bg-success rounded-full border-4 border-base-100 z-10"
                            />
                        </div>
                    </div>

                    <!-- 昵称 -->
                    <div class="form-control mb-4">
                        <label class="label py-1">
                            <span
                                class="label-text text-xs font-bold uppercase tracking-wider text-base-content/70"
                                >昵称</span
                            >
                            <span
                                class="label-text-alt text-[10px] text-base-content/40 font-medium"
                                >{{ (form.display_name || "").length }}/50</span
                            >
                        </label>
                        <input
                            v-model="form.display_name"
                            class="input bg-base-200/50 border-transparent focus:border-primary focus:bg-base-100 focus:shadow-sm transition-all duration-200 rounded-xl text-sm"
                            placeholder="起个好听的名字"
                            maxlength="50"
                        />
                    </div>

                    <!-- 头像链接 -->
                    <div class="form-control mb-4">
                        <label class="label py-1">
                            <span
                                class="label-text text-xs font-bold uppercase tracking-wider text-base-content/70"
                                >头像链接</span
                            >
                        </label>
                        <input
                            v-model="form.avatar_url"
                            class="input bg-base-200/50 border-transparent focus:border-primary focus:bg-base-100 focus:shadow-sm transition-all duration-200 rounded-xl text-sm"
                            placeholder="https://..."
                            type="url"
                        />
                    </div>

                    <!-- 个性签名 -->
                    <div class="form-control mb-6">
                        <label class="label py-1">
                            <span
                                class="label-text text-xs font-bold uppercase tracking-wider text-base-content/70"
                                >个性签名</span
                            >
                            <span
                                class="label-text-alt text-[10px] text-base-content/40 font-medium"
                                >{{ (form.bio || "").length }}/500</span
                            >
                        </label>
                        <textarea
                            v-model="form.bio"
                            class="textarea bg-base-200/50 border-transparent focus:border-primary focus:bg-base-100 focus:shadow-sm transition-all duration-200 rounded-xl text-sm h-24 resize-none leading-relaxed"
                            placeholder="介绍一下你自己..."
                            maxlength="500"
                        />
                    </div>

                    <!-- 操作按钮 -->
                    <div class="flex gap-3 mt-6">
                        <button
                            class="btn btn-ghost flex-1 rounded-xl bg-base-200/50 hover:bg-base-300 transition-colors"
                            @click="close"
                        >
                            取消
                        </button>
                        <button
                            class="btn btn-primary flex-1 rounded-xl shadow-lg shadow-primary/30 hover:shadow-primary/50 transition-all hover:-translate-y-0.5 active:translate-y-0"
                            :disabled="saving"
                            @click="save"
                        >
                            <span
                                v-if="saving"
                                class="loading loading-spinner loading-sm"
                            ></span>
                            <span v-else class="font-bold">保存修改</span>
                        </button>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<script setup>
import { ref, reactive, computed } from "vue";
import { useAppState } from "../composables/useAppState";

const { username, displayName, avatarUrl, bio, saveProfile } = useAppState();

const show = ref(false);
const saving = ref(false);

const form = reactive({
    display_name: "",
    avatar_url: "",
    bio: "",
});

const original = reactive({ display_name: "", avatar_url: "", bio: "" });

const isModified = computed(() => {
    return (
        form.display_name !== original.display_name ||
        form.avatar_url !== original.avatar_url ||
        form.bio !== original.bio
    );
});

const userInitial = computed(
    () => username.value?.charAt(0).toUpperCase() || "?",
);

const avatarBg = computed(() => {
    const colors = [
        "#6366f1",
        "#8b5cf6",
        "#d946ef",
        "#ec4899",
        "#f43f5e",
        "#f97316",
        "#eab308",
        "#22c55e",
        "#14b8a6",
        "#06b6d4",
        "#3b82f6",
    ];
    let hash = 0;
    const name = username.value || "";
    for (let i = 0; i < name.length; i++) {
        hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    return colors[Math.abs(hash) % colors.length];
});

const onAvatarError = (e) => {
    form.avatar_url = "";
    e.target.style.display = "none";
};

const open = () => {
    form.display_name = displayName.value || "";
    form.avatar_url = avatarUrl.value || "";
    form.bio = bio.value || "";
    original.display_name = form.display_name;
    original.avatar_url = form.avatar_url;
    original.bio = form.bio;
    show.value = true;
};

const close = () => {
    show.value = false;
};

const onBackdropClick = () => {
    if (!isModified.value) close();
};

const save = async () => {
    saving.value = true;
    const result = await saveProfile({
        display_name: form.display_name ?? null,
        avatar_url: form.avatar_url ?? null,
        bio: form.bio ?? null,
    });
    saving.value = false;
    if (result.ok) close();
};

defineExpose({ open });
</script>

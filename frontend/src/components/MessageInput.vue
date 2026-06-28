<template>
    <div
        class="px-6 py-4 bg-base-100/90 backdrop-blur-md shrink-0 relative z-10"
    >
        <div class="flex gap-3 items-end max-w-5xl mx-auto group transition-all duration-300 focus-within:-translate-y-0.5">
            <!-- 输入框 + @ 补全 -->
            <div class="flex-1 relative">
                <div
                    class="absolute inset-0 bg-base-200/50 rounded-2xl border border-base-content/15 transition-all duration-300 group-focus-within:bg-base-200 group-focus-within:border-primary/40 group-focus-within:shadow-lg group-focus-within:ring-2 group-focus-within:ring-primary/20"
                />
                <textarea
                    ref="inputEl"
                    v-model="text"
                    class="textarea w-full bg-transparent border-0 focus:outline-none focus:ring-0 transition-all duration-300 text-[15px] resize-none px-4 py-3 min-h-[48px] max-h-[200px] relative z-10 scrollbar-hide"
                    :class="
                        text.length > 450 ? 'text-warning' : 'text-base-content'
                    "
                    :placeholder="isMuted ? '你已被禁言至 ' + new Date(mutedUntil).toLocaleTimeString() : '发送消息到本频道... (Enter 发送，Shift+Enter 换行，@ 提及用户)'"
                    :disabled="isMuted"
                    maxlength="500"
                    rows="1"
                    @input="onTextInput"
                    @keydown="onKeyDown"
                    @blur="onBlurInput"
                />
                <div
                    class="absolute right-3 bottom-2.5 flex items-center gap-2 z-10 pointer-events-none"
                >
                    <span
                        class="text-[10px] font-medium transition-colors"
                        :class="
                            text.length > 450
                                ? 'text-warning'
                                : 'text-base-content/20 opacity-0 group-focus-within:opacity-100'
                        "
                    >
                        {{ text.length }}/500
                    </span>
                </div>

                <!-- @ 提及补全下拉 -->
                <div
                    v-if="mentionUsers.length > 0"
                    class="absolute bottom-full left-0 mb-2 bg-base-100 rounded-xl border border-base-200 shadow-2xl z-40 w-64 max-h-56 overflow-y-auto overflow-hidden"
                >
                    <div
                        class="px-3 py-2 text-[10px] font-bold text-base-content/40 uppercase tracking-wider bg-base-200/50 sticky top-0 backdrop-blur-md"
                    >
                        提及用户
                    </div>
                    <div
                        v-for="u in mentionUsers"
                        :key="u.username"
                        class="px-3 py-2 hover:bg-base-200 cursor-pointer flex items-center gap-3 transition-colors"
                        :class="
                            highlightIdx === mentionIdx(u)
                                ? 'bg-primary/10 border-l-2 border-primary'
                                : 'border-l-2 border-transparent'
                        "
                        @mousedown.prevent="selectMention(u)"
                    >
                        <div class="avatar placeholder">
                            <div
                                class="w-6 h-6 rounded-full bg-base-300 text-base-content/50 text-[10px] font-bold"
                            >
                                <img v-if="u.avatar_url" :src="u.avatar_url" />
                                <span v-else>{{
                                    u.username.charAt(0).toUpperCase()
                                }}</span>
                            </div>
                        </div>
                        <div class="flex flex-col min-w-0">
                            <span
                                class="text-sm font-semibold text-base-content truncate"
                                >{{ u.display_name || u.username }}</span
                            >
                            <span
                                v-if="u.display_name"
                                class="text-[10px] text-base-content/50 truncate"
                                >@{{ u.username }}</span
                            >
                        </div>
                    </div>
                </div>
            </div>

            <!-- 发送按钮 -->
            <button
                class="btn btn-primary btn-circle h-12 w-12 border-2 border-primary/30 shadow-md shadow-primary/20 hover:shadow-lg hover:shadow-primary/30 transition-all duration-300 hover:-translate-y-0.5 active:translate-y-0 shrink-0 group-focus-within:shadow-xl group-focus-within:shadow-primary/40 group-focus-within:border-primary/60 group-focus-within:brightness-110"
                :class="{ 'opacity-50 scale-95': !text.trim() }"
                :disabled="!text.trim()"
                @click="doSend"
            >
                <svg
                    class="w-5 h-5 ml-0.5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2.5"
                        d="M12 19V5m0 0l-6 6m6-6l6 6"
                    />
                </svg>
            </button>
        </div>
    </div>
</template>

<script setup>
import { ref, nextTick, onMounted } from "vue";
import { useWebSocket } from "../composables/useWebSocket";
import { useAppState } from "../composables/useAppState";

const { sendMessage } = useWebSocket();
const { token, isMuted, mutedUntil } = useAppState();
const text = ref("");
const inputEl = ref(null);
const mentionUsers = ref([]);
const highlightIdx = ref(0);
let mentionStart = -1;
let searchTimer = null;

onMounted(() => {
    inputEl.value?.focus();
});

// 输入时检测 @ 触发补全
const onTextInput = () => {
    const val = text.value;
    const cursor = inputEl.value?.selectionStart || val.length;
    // @ 触发检测
    // 找到光标前最近的 @ 位置
    const atIdx = val.lastIndexOf("@", cursor - 1);
    if (
        atIdx >= 0 &&
        (atIdx === 0 || val[atIdx - 1] === " " || val[atIdx - 1] === "\n")
    ) {
        const query = val.slice(atIdx + 1, cursor);
        if (!query.includes(" ")) {
            mentionStart = atIdx;
            clearTimeout(searchTimer);
            searchTimer = setTimeout(() => searchUsers(query), 150);
            return;
        }
    }
    mentionStart = -1;
    mentionUsers.value = [];
    highlightIdx.value = 0;
};

// 搜索用户
const searchUsers = async (q) => {
    // 查询匹配用户
    if (!q) {
        mentionUsers.value = [];
        return;
    }
    try {
        const headers = {};
        if (token.value) headers["Authorization"] = `Bearer ${token.value}`;
        const res = await fetch(`/api/users?q=${encodeURIComponent(q)}`, {
            headers,
        });
        if (res.ok) {
            mentionUsers.value = await res.json();
            highlightIdx.value = 0;
        }
    } catch {}
};

// 键盘导航
const onKeyDown = (e) => {
    // Enter：有下拉时选人，无下拉时发送；Shift+Enter 换行
    if (e.key === "Enter" && !e.shiftKey) {
        if (mentionUsers.value.length > 0) {
            e.preventDefault();
            selectMention(mentionUsers.value[highlightIdx.value]);
            return;
        }
        e.preventDefault();
        doSend();
        return;
    }
    if (mentionUsers.value.length === 0) return;
    if (e.key === "ArrowDown") {
        e.preventDefault();
        highlightIdx.value =
            (highlightIdx.value + 1) % mentionUsers.value.length;
    } else if (e.key === "ArrowUp") {
        e.preventDefault();
        highlightIdx.value =
            (highlightIdx.value - 1 + mentionUsers.value.length) %
            mentionUsers.value.length;
    } else if (e.key === "Tab") {
        e.preventDefault();
        selectMention(mentionUsers.value[highlightIdx.value]);
    } else if (e.key === "Escape") {
        mentionUsers.value = [];
    }
};

// 选中用户
const selectMention = (user) => {
    const before = text.value.slice(0, mentionStart);
    const after = text.value.slice(
        inputEl.value?.selectionStart || mentionStart + 1,
    );
    text.value = before + "@" + user.username + " " + after;
    mentionUsers.value = [];
    mentionStart = -1;
    nextTick(() => inputEl.value?.focus());
};

// 失焦时关闭
const onBlurInput = () => {
    setTimeout(() => {
        mentionUsers.value = [];
    }, 200);
};

// 计算高亮下标对应数组中的位置
const mentionIdx = (u) => mentionUsers.value.indexOf(u);

// 发送
const doSend = () => {
    if (!text.value.trim()) return;
    sendMessage(text.value);
    text.value = "";
    mentionUsers.value = [];
    inputEl.value.style.height = "auto"; // 重置高度
    inputEl.value?.focus();
};

// 自动调整高度
import { watch } from "vue";
watch(text, () => {
    nextTick(() => {
        if (inputEl.value) {
            inputEl.value.style.height = "auto";
            inputEl.value.style.height =
                Math.min(inputEl.value.scrollHeight, 200) + "px";
        }
    });
});
</script>

<template>
    <div
        class="w-full flex"
        :class="isMine ? 'justify-end' : 'justify-start'"
        @mouseenter="hovering = true"
        @mouseleave="onBubbleLeave"
    >
        <div
            class="flex max-w-[85%] group"
            :class="isMine ? 'flex-row-reverse' : 'flex-row'"
        >
            <!-- 头像 -->
            <div class="flex-shrink-0 mt-auto mb-1 mx-2">
                <div class="avatar placeholder">
                    <div
                        class="w-8 h-8 rounded-2xl shadow-sm text-[11px] font-bold text-white transition-transform group-hover:scale-105"
                        :style="
                            message.avatar_url
                                ? {}
                                : { background: avatarColor }
                        "
                    >
                        <img
                            v-if="message.avatar_url"
                            :src="message.avatar_url"
                            class="w-full h-full object-cover"
                            @error="message.avatar_url = null"
                        />
                        <span v-else>{{ initial }}</span>
                    </div>
                </div>
            </div>

            <!-- 消息主体 -->
            <div
                class="flex flex-col min-w-0"
                :class="isMine ? 'items-end' : 'items-start'"
            >
                <!-- 头部信息 -->
                <div
                    class="flex items-center gap-2 mb-1 px-1"
                    :class="isMine ? 'flex-row-reverse' : 'flex-row'"
                >
                    <span class="text-[13px] font-bold text-base-content/80 cursor-pointer hover:text-primary hover:underline transition-colors"
                        @click="startDmChat"
                        :title="'私聊 ' + (message.display_name || message.username)">{{
                        message.display_name || message.username
                    }}</span>
                    <time
                        class="text-[10px] font-medium text-base-content/30 opacity-0 group-hover:opacity-100 transition-opacity duration-300"
                    >
                        {{ fmtTime(message.created_at) }}
                    </time>
                </div>

                <!-- 气泡 -->
                <div class="relative max-w-full">
                    <!-- 删除按钮（仅自己） -->
                    <button
                        v-if="isMine"
                        class="absolute top-0 right-full mr-2 p-1.5 rounded-full bg-base-200 text-base-content/40 hover:text-error hover:bg-error/10 opacity-0 group-hover:opacity-100 transition-all duration-200 hover:scale-110"
                        title="删除消息"
                        @click="$emit('delete', message.id)"
                    >
                        <svg
                            class="w-3.5 h-3.5"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                            />
                        </svg>
                    </button>

                    <div
                        class="text-[14px] leading-relaxed break-words px-4 py-2.5 shadow-sm"
                        :class="
                            isMine
                                ? 'bg-primary text-primary-content rounded-[20px] rounded-br-sm'
                                : 'bg-base-200/80 text-base-content rounded-[20px] rounded-bl-sm border border-base-200/50'
                        "
                    >
                        <div
                            v-html="renderedContent"
                            class="markdown-body"
                            :class="isMine ? 'text-primary-content' : ''"
                        />
                    </div>
                </div>

                <!-- 反应展示区 -->
                <div
                    class="flex items-center gap-1.5 mt-1"
                    :class="isMine ? 'flex-row-reverse' : 'flex-row'"
                >
                    <!-- 反应列表 -->
                    <div
                        v-if="msgReactions"
                        class="flex flex-wrap gap-1"
                        :class="isMine ? 'flex-row-reverse' : 'flex-row'"
                    >
                        <button
                            v-for="(info, emoji) in msgReactions"
                            :key="emoji"
                            class="flex items-center gap-1 px-1.5 py-0.5 rounded-full border text-[11px] font-medium transition-colors cursor-pointer hover:scale-105 active:scale-95"
                            :class="
                                info.me
                                    ? 'bg-primary/10 border-primary/30 text-primary'
                                    : 'bg-base-200/50 border-base-200 text-base-content/60 hover:bg-base-200'
                            "
                            @click="toggleReaction(emoji)"
                        >
                            <span>{{ emoji }}</span>
                            <span>{{ info.count }}</span>
                        </button>
                    </div>

                    <!-- 添加反应按钮 -->
                    <div
                        class="relative opacity-0 group-hover:opacity-100 transition-opacity duration-200"
                    >
                        <button
                            class="p-1 rounded-full text-base-content/40 hover:bg-base-200 hover:text-base-content/70 transition-colors"
                            title="添加反应"
                            @click.stop="showPicker = !showPicker"
                        >
                            <svg
                                class="w-4 h-4"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                />
                            </svg>
                        </button>

                        <!-- 表情弹出面板 -->
                        <div
                            v-if="showPicker"
                            data-picker
                            class="absolute bottom-full mb-2 bg-base-100/90 backdrop-blur-md border border-base-200 shadow-xl rounded-full px-2 py-1.5 z-30 flex gap-1 animate-in fade-in slide-in-from-bottom-2 duration-200"
                            :class="
                                isMine
                                    ? 'right-0 origin-bottom-right'
                                    : 'left-0 origin-bottom-left'
                            "
                            style="white-space: nowrap"
                        >
                            <button
                                v-for="e in EMOJIS"
                                :key="e"
                                class="w-7 h-7 flex items-center justify-center text-lg hover:bg-base-200 rounded-full hover:scale-110 transition-all active:scale-95"
                                @click="pickEmoji(e)"
                            >
                                {{ e }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
import { computed, ref } from "vue";
import { renderMarkdown } from "../composables/useMarkdown";
import { useWebSocket } from "../composables/useWebSocket";
import { useAppState } from "../composables/useAppState";
import { useDm } from "../composables/useDm";

// ── 预设表情 ──
const EMOJIS = ["👍", "❤️", "😂", "😮", "😢", "🙏"];

// ── 表情选择器展开状态 ──
const showPicker = ref(false);

// ── 点击面板外部自动关闭 ──
const closePickerOnOutside = (e) => {
    if (!showPicker.value) return;
    // 延迟检查，确保 Vue 已更新 DOM
    setTimeout(() => {
        if (showPicker.value && !e.target.closest("[data-picker]")) {
            showPicker.value = false;
        }
    }, 100);
};
document.addEventListener("click", closePickerOnOutside);
// ── 鼠标是否悬停在气泡区域（精确到 chat 组件） ──
const hovering = ref(false);

// ── 鼠标离开气泡：仅在选择器未展开时隐藏 ──
const onBubbleLeave = () => {
    if (!showPicker.value) hovering.value = false;
};

const props = defineProps({
    message: { type: Object, required: true },
    isMine: { type: Boolean, default: false },
});

defineEmits(["delete"]);

const { reactions } = useWebSocket();
const { token, isGuest, username } = useAppState();
const { startDmByUsername } = useDm();

const startDmChat = async () => {
    if (isGuest.value || props.isMine) return;
    try {
        await startDmByUsername(props.message.username);
    } catch (e) {
        console.error('发起私聊失败:', e);
    }
};

// ── 当前消息的反应 ──
const msgReactions = computed(() => {
    const r = reactions.value[props.message.id];
    return r && Object.keys(r).length > 0 ? r : null;
});

// ── 从弹出面板选表情 → 关闭面板后切换 ──
const pickEmoji = async (emoji) => {
    showPicker.value = false;
    await toggleReaction(emoji);
};

// ── 切换表情 ──
const toggleReaction = async (emoji) => {
    if (!props.message.id) return;
    await fetch(`/api/messages/${props.message.id}/react`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token.value}`,
        },
        body: JSON.stringify({ emoji }),
    });
};

// ── Markdown 渲染 + @提及高亮 ──
const renderedContent = computed(() => {
    let html = renderMarkdown(props.message.content);
    // 在已渲染的 HTML 中高亮 @用户名（避免触碰 <a> 等标签内的内容）
    html = html.replace(
        /(^|\s)@(\w{3,30})(?=[\s,，。.!！?？:：;；]|$)/g,
        '$1<span class="text-primary font-semibold bg-primary/10 rounded px-1">@$2</span>',
    );
    return html;
});
const initial = computed(
    () => props.message.username?.charAt(0).toUpperCase() || "?",
);

const avatarColor = computed(() => {
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
    const name = props.message.username || "";
    for (let i = 0; i < name.length; i++) {
        hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    return colors[Math.abs(hash) % colors.length];
});

const fmtTime = (iso) => {
    if (!iso) return "";
    return new Date(iso).toLocaleTimeString("zh-CN", {
        hour: "2-digit",
        minute: "2-digit",
        hour12: false,
    });
};
</script>

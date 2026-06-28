<template>
    <aside
        class="w-[280px] h-full flex flex-col bg-base-200/20 border-r-2 border-base-content/20"
    >
        <!-- 品牌区 -->
        <div class="px-5 py-5 border-b border-base-content/20">
            <div class="flex items-center gap-3">
                <div
                    class="w-10 h-10 rounded-2xl bg-gradient-to-br from-indigo-500 via-purple-500 to-pink-500 flex items-center justify-center shadow-lg shadow-purple-500/30 shrink-0"
                >
                    <svg
                        class="w-5 h-5 text-white"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2.5"
                            d="M8.625 9.75a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H8.25m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H12m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0h-.375m-13.5 3.01c0 1.6 1.123 2.994 2.707 3.227 1.087.16 2.185.283 3.293.369V21l4.184-4.183a1.14 1.14 0 01.778-.332 48.294 48.294 0 005.83-.498c1.585-.233 2.708-1.626 2.708-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0012 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018z"
                        />
                    </svg>
                </div>
                <div class="min-w-0">
                    <h2
                        class="text-base font-extrabold text-base-content tracking-tight"
                    >
                        VokloWave
                    </h2>
                    <p class="text-[11px] font-medium text-base-content/50">
                        团队即时通讯
                    </p>
                </div>
            </div>
        </div>

        <!-- 频道标题 -->
        <div class="flex items-center justify-between px-6 pt-2 pb-2 mt-4">
            <span
                class="text-[11px] font-bold uppercase tracking-[0.15em] text-base-content/40"
                >Channels</span
            >
            <button
                v-if="!isGuest"
                class="text-base-content/30 hover:text-base-content transition-colors p-1"
                @click="showCreateModal = true"
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
                        stroke-width="2.5"
                        d="M12 4v16m8-8H4"
                    />
                </svg>
            </button>
        </div>

        <!-- 频道列表 -->
        <div
            class="flex-1 overflow-y-auto px-3 pb-4 space-y-0.5 scrollbar-hide"
        >
            <div
                v-for="ch in channels"
                :key="ch.id"
                class="flex items-center gap-3 px-3 py-2 rounded-xl cursor-pointer transition-all duration-150 group relative"
                :class="
                    ch.name === currentChannel
                        ? 'bg-base-200/80 text-base-content font-medium shadow-sm'
                        : 'text-base-content/60 hover:bg-base-200/50 hover:text-base-content'
                "
                @click="selectChannel(ch.name)"
            >
                <span class="text-base shrink-0 font-medium opacity-40">#</span>
                <span class="truncate text-[14px]">{{ ch.name }}</span>
                <span
                    v-if="ch.name === currentChannel"
                    class="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-4 bg-base-content rounded-r-full"
                />
            </div>

            <!-- 空状态 -->
            <div
                v-if="channels.length === 0 && !loading"
                class="flex flex-col items-center py-10 text-base-content/25 gap-2"
            >
                <svg
                    class="w-8 h-8"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="1.5"
                        d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
                    />
                </svg>
                <p class="text-xs">暂无频道</p>
            </div>
        </div>

        <!-- ── 私聊列表 ── -->
        <template v-if="!isGuest">
            <div class="flex items-center justify-between px-6 pt-2 pb-2">
                <span class="text-[11px] font-bold uppercase tracking-[0.15em] text-base-content/40">私聊</span>
            </div>
            <div class="flex-1 overflow-y-auto px-3 pb-2 space-y-0.5 scrollbar-hide max-h-[30vh]">
                <div v-for="c in dmConversations" :key="c.conversation_id"
                    class="flex items-center gap-3 px-3 py-2 rounded-xl cursor-pointer transition-all duration-150 group"
                    :class="activeDmId === c.conversation_id ? 'bg-base-200/80 text-base-content font-medium shadow-sm' : 'text-base-content/60 hover:bg-base-200/50 hover:text-base-content'"
                    @click="selectDm(c.conversation_id)">
                    <div class="avatar placeholder shrink-0">
                        <div class="w-7 h-7 rounded-full bg-base-300 text-[10px] font-bold"
                            :style="{ background: avatarColor(c.other_username) }">
                            <img v-if="c.other_avatar_url" :src="c.other_avatar_url" class="w-full h-full object-cover rounded-full" />
                            <span v-else>{{ (c.other_display_name || c.other_username).charAt(0).toUpperCase() }}</span>
                        </div>
                    </div>
                    <div class="min-w-0 flex-1">
                        <p class="text-[13px] font-medium truncate">{{ c.other_display_name || c.other_username }}</p>
                        <p class="text-[11px] text-base-content/40 truncate">{{ c.last_message || '开始私聊' }}</p>
                    </div>
                </div>
                <div v-if="dmConversations.length === 0" class="text-center py-3 text-xs text-base-content/30">
                    暂无私聊
                </div>
            </div>
        </template>

        <!-- 底部：用户区 -->
        <div class="p-3 border-t border-base-content/20 bg-base-100 shrink-0">
            <div class="dropdown dropdown-top w-full">
                <!-- 用户身份悬浮卡片 -->
                <div
                    tabindex="0"
                    role="button"
                    class="flex items-center gap-2.5 px-3 py-2 rounded-xl hover:bg-base-200/50 transition-colors cursor-pointer group"
                >
                    <div class="avatar placeholder shrink-0">
                        <div
                            class="w-8 h-8 rounded-full bg-base-200 border border-base-300 shadow-sm text-xs font-bold text-base-content transition-transform group-hover:scale-105"
                            :style="{ background: avatarBg }"
                        >
                            <!-- ── 新增：有头像链接时显示图片 ── -->
                            <img
                                v-if="avatarUrl"
                                :src="avatarUrl"
                                class="w-full h-full object-cover"
                                @error="avatarUrl = ''"
                            />
                            <span v-else>{{
                                username
                                    ? username.charAt(0).toUpperCase()
                                    : "?"
                            }}</span>
                        </div>
                        <!-- 在线绿点 -->
                        <div
                            class="absolute bottom-0 right-0 w-2 h-2 bg-success rounded-full ring-2 ring-base-100"
                        ></div>
                    </div>
                    <div
                        class="min-w-0 flex-1 flex flex-col justify-center text-left"
                    >
                        <!-- ── 新增：优先显示昵称，回退用户名 ── -->
                        <p
                            class="text-[13px] font-semibold text-base-content truncate leading-tight"
                        >
                            {{ displayName || username || "未登录" }}
                        </p>
                        <p
                            v-if="bio"
                            class="text-[11px] text-base-content/40 font-medium truncate mt-[1px]"
                        >
                            {{ bio }}
                        </p>
                    </div>
                </div>

                <!-- 弹出菜单 -->
                <ul
                    tabindex="0"
                    class="dropdown-content z-50 menu p-1.5 shadow-2xl bg-base-100/95 backdrop-blur-xl rounded-2xl border border-base-200/80 w-full mb-2 space-y-0.5"
                >
                    <!-- 资料编辑 -->
                    <li>
                        <a
                            @click="profileModal?.open()"
                            class="rounded-xl text-[13px] font-medium py-2"
                        >
                            <svg
                                class="w-4 h-4 opacity-50"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                                />
                            </svg>
                            编辑资料
                        </a>
                    </li>

                    <!-- 外观设置 -->
                    <li>
                        <a
                            @click="themeModal?.open()"
                            class="rounded-xl text-[13px] font-medium py-2"
                        >
                            <svg
                                class="w-4 h-4 opacity-50"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01"
                                ></path>
                            </svg>
                            外观设置
                        </a>
                    </li>

                    <div class="divider my-0 opacity-30 h-[1px]"></div>

                    <!-- 活跃会话 -->
                    <li v-if="token && !isGuest">
                        <a
                            @click="sessionModal?.open()"
                            class="rounded-xl text-[13px] font-medium py-2"
                        >
                            <svg
                                class="w-4 h-4 opacity-50"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                                />
                            </svg>
                            活跃会话
                        </a>
                    </li>

                    <!-- 管理后台 -->
                    <li v-if="isAdmin">
                        <a
                            href="/#/admin"
                            class="rounded-xl text-[13px] font-medium py-2"
                        >
                            <svg
                                class="w-4 h-4 opacity-50"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                                />
                            </svg>
                            管理后台
                        </a>
                    </li>

                    <!-- 退出登录 -->
                    <li v-if="token">
                        <a
                            @click="confirmLogout"
                            class="rounded-xl text-[13px] font-medium py-2 text-error hover:bg-error/10 hover:text-error"
                        >
                            <svg
                                class="w-4 h-4 opacity-70"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                                />
                            </svg>
                            退出登录
                        </a>
                    </li>
                </ul>
            </div>
        </div>

        <!-- ── 资料编辑弹窗 ── -->
        <ProfileEditModal ref="profileModal" />

        <!-- ── 会话管理弹窗 ── -->
        <SessionListModal ref="sessionModal" />

        <!-- ── 外观设置弹窗 ── -->
        <ThemeSettingsModal ref="themeModal" />
    </aside>
</template>

<script setup>
import { computed, ref, onMounted } from "vue";
import { useAppState } from "../composables/useAppState";
import { useChannels } from "../composables/useChannels";
import ProfileEditModal from "./ProfileEditModal.vue";
import SessionListModal from "./SessionListModal.vue";
import ThemeSettingsModal from "./ThemeSettingsModal.vue";
import { useDm } from "../composables/useDm";

const {
    username,
    token,
    currentChannel,
    theme,
    switchChannel,
    showCreateModal,
    logout,
    isGuest,
    displayName,
    avatarUrl,
    bio,
    isAdmin,
} = useAppState();
const { channels, loading } = useChannels();

const profileModal = ref(null);
const themeModal = ref(null);
const sessionModal = ref(null);

const { conversations: dmConversations, activeConvId: activeDmId, fetchList: fetchDmList, openDm, startPolling } = useDm();

const confirmLogout = () => {
    if (confirm("确定要退出登录吗？")) {
        logout();
    }
};

const selectChannel = (name) => {
    const toggle = document.getElementById("sidebar-toggle");
    if (toggle) toggle.checked = false;
    switchChannel(name);
};

const selectDm = (convId) => {
    const toggle = document.getElementById("sidebar-toggle");
    if (toggle) toggle.checked = false;
    openDm(convId);
};

const avatarColor = (name) => {
    let hash = 0;
    for (let i = 0; i < (name || '').length; i++) {
        hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue = Math.abs(hash) % 360;
    return `hsl(${hue}, 50%, 55%)`;
};

const selectTheme = (value) => {
    theme.value = value;
    document.activeElement?.blur();
};

const themes = [
    { value: "dark", label: "深色", emoji: "🌙" },
    { value: "light", label: "浅色", emoji: "☀️" },
    { value: "cyberpunk", label: "赛博朋克", emoji: "🤖" },
    { value: "cupcake", label: "纸杯蛋糕", emoji: "🧁" },
    { value: "synthwave", label: "合成波", emoji: "🎆" },
    { value: "nord", label: "北欧风", emoji: "🏔️" },
    { value: "sunset", label: "日落", emoji: "🌅" },
    { value: "winter", label: "冬季", emoji: "❄️" },
    { value: "coffee", label: "咖啡", emoji: "☕" },
    { value: "lemonade", label: "柠檬水", emoji: "🍋" },
    { value: "luxury", label: "奢华金", emoji: "💎" },
    { value: "business", label: "商务", emoji: "👔" },
    { value: "autumn", label: "秋季", emoji: "🍁" },
    { value: "dim", label: "暗紫", emoji: "🔮" },
];

const themeLabel = computed(() => {
    const t = themes.find((t) => t.value === theme.value);
    return t ? t.label : "深色";
});

onMounted(() => {
    fetchDmList();
    startPolling();
});

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
    const name = username.value || "?";
    for (let i = 0; i < name.length; i++) {
        hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    return colors[Math.abs(hash) % colors.length];
});
</script>

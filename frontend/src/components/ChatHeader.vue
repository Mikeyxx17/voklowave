<template>
    <header
        class="flex items-center gap-4 px-6 py-4 border-b border-base-content/20 bg-base-100/80 backdrop-blur-xl shrink-0 hidden lg:flex z-10 sticky top-0 shadow-sm"
    >
        <div class="flex items-center gap-3 min-w-0">
            <div
                class="w-8 h-8 rounded-xl bg-primary/10 flex items-center justify-center shrink-0"
            >
                <span class="text-xl font-bold text-primary">#</span>
            </div>
            <span class="text-lg font-bold text-base-content truncate">{{
                currentChannel
            }}</span>
        </div>

        <div class="flex items-center gap-3 ml-2">
            <div
                class="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-base-200/50 border border-base-200 text-xs font-medium"
            >
                <span class="relative flex h-2 w-2">
                    <span
                        v-if="connected"
                        class="animate-ping absolute inline-flex h-full w-full rounded-full bg-success opacity-75"
                    ></span>
                    <span
                        class="relative inline-flex rounded-full h-2 w-2"
                        :class="connected ? 'bg-success' : 'bg-error'"
                    ></span>
                </span>
                <span
                    :class="connected ? 'text-base-content/80' : 'text-error'"
                    >{{ connected ? "在线" : "离线" }}</span
                >
            </div>

            <div
                class="px-2.5 py-1 rounded-full bg-base-200/50 border border-base-200 text-xs font-medium text-base-content/80"
            >
                {{ msgCount }} 条消息
            </div>
        </div>

        <div class="flex-1" />

        <!-- ── 消息搜索 ── -->
        <div class="relative group">
            <div
                class="flex items-center gap-2 bg-base-200/50 border border-base-content/20 group-focus-within:border-primary/50 group-focus-within:bg-base-100 group-focus-within:shadow-md transition-all duration-300 rounded-full px-1.5 py-1"
            >
                <svg
                    class="w-4 h-4 text-base-content/40 ml-2"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                    />
                </svg>
                <input
                    ref="searchInput"
                    v-model="query"
                    class="bg-transparent border-none outline-none text-sm w-44 transition-all duration-300 placeholder:text-base-content/30 focus:w-64"
                    placeholder="搜索消息..."
                    maxlength="100"
                    @keyup.enter="doSearch"
                    @input="onInput"
                    @focus="onFocus"
                />
                <!-- ── 搜索范围选择（访客只能搜 general，无需显示） ── -->
                <div v-if="!isGuest" class="relative">
                    <button
                        class="text-xs min-w-0 w-auto bg-transparent border-none outline-none hover:bg-base-200 pl-2 pr-2 h-7 min-h-7 rounded-full flex items-center gap-1 cursor-pointer"
                        @click.stop="toggleScopeDropdown"
                        @blur="onScopeBlur"
                    >
                        <span class="truncate max-w-[80px]">{{
                            scopeLabel
                        }}</span>
                        <svg
                            class="w-3 h-3 opacity-40 shrink-0"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 9l-7 7-7-7"
                            />
                        </svg>
                    </button>
                    <div
                        v-show="scopeOpen"
                        class="absolute top-full left-0 mt-1 bg-base-100 border border-base-200 rounded-xl shadow-lg z-50 min-w-[130px] max-h-40 overflow-y-auto"
                    >
                        <button
                            class="w-full text-left px-3 py-1.5 text-xs hover:bg-base-200 transition-colors first:rounded-t-xl"
                            :class="
                                searchScope === 'all'
                                    ? 'text-primary font-semibold bg-primary/5'
                                    : 'text-base-content'
                            "
                            @click="selectScope('all')"
                        >
                            全部频道
                        </button>
                        <button
                            v-for="ch in channels"
                            :key="ch.id"
                            class="w-full text-left px-3 py-1.5 text-xs hover:bg-base-200 transition-colors last:rounded-b-xl"
                            :class="
                                searchScope === ch.name
                                    ? 'text-primary font-semibold bg-primary/5'
                                    : 'text-base-content'
                            "
                            @click="selectScope(ch.name)"
                        >
                            #{{ ch.name }}
                        </button>
                    </div>
                </div>
                <button
                    class="btn btn-primary btn-sm btn-circle h-7 w-7 min-h-7 mr-0.5"
                    :class="{ 'opacity-50': !query.trim() || searching }"
                    :disabled="!query.trim() || searching"
                    @click="doSearch"
                >
                    <svg
                        v-if="searching"
                        class="w-3.5 h-3.5 animate-spin"
                        fill="none"
                        viewBox="0 0 24 24"
                    >
                        <circle
                            class="opacity-25"
                            cx="12"
                            cy="12"
                            r="10"
                            stroke="currentColor"
                            stroke-width="4"
                        />
                        <path
                            class="opacity-75"
                            fill="currentColor"
                            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                        />
                    </svg>
                    <svg
                        v-else
                        class="w-3.5 h-3.5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M9 5l7 7-7 7"
                        />
                    </svg>
                </button>
            </div>
        </div>

        <div class="hidden"></div>
    </header>

    <!-- ── 搜索结果 —— 独立于 header 的固定定位面板，避免 CSS 层叠上下文干扰 ── -->
    <Teleport to="body">
        <div
            v-if="showResults"
            class="fixed inset-0 z-[100] flex items-start justify-center pt-24 bg-base-300/5"
            @click.self="showResults = false"
        >
            <div
                class="bg-base-100 rounded-2xl border border-base-200 shadow-2xl shadow-base-content/10 w-[36rem] max-h-[65vh] flex flex-col"
                :style="{ transform: `translate(${dragX}px, ${dragY}px)` }"
            >
                <!-- ── 标题栏始终显示 ── -->
                <div
                    class="sticky top-0 z-10 bg-base-200/80 backdrop-blur-md px-5 py-3 border-b border-base-200 flex items-center justify-between cursor-move select-none rounded-t-2xl"
                    @mousedown="onDragStart"
                >
                    <div class="flex items-center gap-2">
                        <span class="text-xs font-bold text-base-content/70">搜索结果</span>
                        <span v-if="total > 0" class="badge badge-sm badge-primary">{{ total }}</span>
                        <span v-else class="text-xs text-base-content/40">无匹配</span>
                    </div>
                    <button
                        class="btn btn-ghost btn-sm btn-circle"
                        @click="showResults = false"
                        @mousedown.stop
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                        </svg>
                    </button>
                </div>
                <!-- ── 结果列表 ── -->
                <div
                    v-if="results.length === 0"
                    class="px-6 py-12 flex flex-col items-center justify-center text-base-content/40 flex-1"
                >
                    <svg
                        class="w-12 h-12 mb-4 opacity-50"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="1.5"
                            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                        />
                    </svg>
                    <p class="text-sm font-medium">未找到匹配的消息</p>
                    <p class="text-xs mt-1">尝试换个关键词试试</p>
                </div>
                <div v-else class="overflow-y-auto flex-1 min-h-0 p-2">
                        <div
                            v-for="(msg, idx) in results"
                            :key="msg.id"
                            class="px-4 py-3 m-1 rounded-xl hover:bg-base-200 transition-colors cursor-pointer border border-transparent hover:border-base-300"
                            @click="jumpTo(msg)"
                        >
                            <div class="flex items-center gap-2.5 mb-1.5">
                                <span
                                    class="badge badge-ghost badge-sm text-[10px] font-bold text-primary"
                                    >#{{ msg.channel }}</span
                                >
                                <span
                                    class="text-xs font-bold text-base-content/80"
                                    >{{ msg.username }}</span
                                >
                                <span
                                    class="text-[10px] font-medium text-base-content/40 ml-auto"
                                    >{{ fmtTime(msg.created_at) }}</span
                                >
                            </div>
                            <p
                                class="text-sm text-base-content/90 line-clamp-2 break-words leading-relaxed"
                                v-html="highlight(msg.content)"
                            ></p>
                        </div>
                        <!-- ── 加载更多 ── -->
                        <div
                            v-if="results.length < total"
                            class="px-4 py-2 flex justify-center"
                        >
                            <button
                                class="btn btn-ghost btn-sm gap-1.5"
                                :disabled="searchingMore"
                                @click="loadMore"
                            >
                                <svg
                                    v-if="searchingMore"
                                    class="w-3.5 h-3.5 animate-spin"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                >
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
                                </svg>
                                <svg
                                    v-else
                                    class="w-3.5 h-3.5"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                                </svg>
                                加载更多（{{ total - results.length }} 条剩余）
                            </button>
                        </div>
                </div>
            </div>
        </div>
    </Teleport>
</template>

<script setup>
import { ref, computed, watch } from "vue";
import { useAppState } from "../composables/useAppState";
import { useWebSocket } from "../composables/useWebSocket";
import { useChannels } from "../composables/useChannels"; // 新增

const {
    username,
    displayName,
    avatarUrl,
    currentChannel,
    token,
    switchChannel,
    isGuest,
} = useAppState();
const { messages, connected, scrollToId } = useWebSocket();
const { channels } = useChannels(); // 新增：频道列表用于搜索范围

const msgCount = computed(() => messages.value.length);

// ── 搜索状态 ──
const query = ref("");
const results = ref([]);
const total = ref(0);
const searching = ref(false);
const searchingMore = ref(false);
const showResults = ref(false);
const offset = ref(0);
const searchScope = ref("all"); // 搜索范围：'all' = 全部频道，或具体频道名
const searchInput = ref(null);
const scopeOpen = ref(false);
let debounceTimer = null;
let scopeBlurTimer = null;

const scopeLabel = computed(() => {
    if (searchScope.value === "all") return "全部频道";
    return "#" + searchScope.value;
});

const toggleScopeDropdown = () => {
    scopeOpen.value = !scopeOpen.value;
};

const onScopeBlur = () => {
    // 延迟关闭，让 click 事件先触发
    scopeBlurTimer = setTimeout(() => {
        scopeOpen.value = false;
    }, 150);
};

const selectScope = (value) => {
    clearTimeout(scopeBlurTimer);
    searchScope.value = value;
    scopeOpen.value = false;
    // 切换范围后若有搜索词则重新搜索
    if (query.value.trim()) {
        doSearch();
    }
};

// ── 输入时 300ms 防抖自动搜索 ──
const onInput = () => {
    clearTimeout(debounceTimer);
    if (!query.value.trim()) {
        results.value = [];
        total.value = 0;
        showResults.value = false;
        return;
    }
    debounceTimer = setTimeout(doSearch, 300);
};

// ── 获得焦点时如果已有结果则显示 ──
const onFocus = () => {
    if (results.value.length > 0) {
        showResults.value = true;
    }
};

// ── 搜索结果窗口拖拽状态 ──
const dragX = ref(0);
const dragY = ref(0);
let dragStartX = 0;
let dragStartY = 0;

const onDragStart = (e) => {
    dragStartX = e.clientX - dragX.value;
    dragStartY = e.clientY - dragY.value;
    document.addEventListener("mousemove", onDragMove);
    document.addEventListener("mouseup", onDragEnd);
};

const onDragMove = (e) => {
    dragX.value = e.clientX - dragStartX;
    dragY.value = e.clientY - dragStartY;
};

const onDragEnd = () => {
    document.removeEventListener("mousemove", onDragMove);
    document.removeEventListener("mouseup", onDragEnd);
};

// ── 失焦时不再自动隐藏，改由点击外部关闭 ──

// ── 执行搜索 ──
const doSearch = async (append = false) => {
    const q = query.value.trim();
    if (!q) return;

    // 构建带范围参数的 URL
    const scopeParam =
        searchScope.value !== "all"
            ? `&channel=${encodeURIComponent(searchScope.value)}`
            : "";
    const currentOffset = append ? offset.value : 0;

    if (append) {
        searchingMore.value = true;
    } else {
        searching.value = true;
    }

    try {
        const headers = {};
        if (token.value) {
            headers["Authorization"] = `Bearer ${token.value}`;
        }
        const res = await fetch(
            `/api/messages/search?q=${encodeURIComponent(q)}${scopeParam}&limit=20&offset=${currentOffset}`,
            { headers },
        );
        if (res.ok) {
            const data = await res.json();
            if (append) {
                results.value.push(...data.results);
            } else {
                results.value = data.results;
            }
            total.value = data.total;
            offset.value = currentOffset + data.results.length;
            showResults.value = true;
        }
    } catch (err) {
        console.error("搜索失败:", err);
    } finally {
        if (append) {
            searchingMore.value = false;
        } else {
            searching.value = false;
        }
    }
};

// ── 加载更多搜索结果 ──
const loadMore = () => {
    doSearch(true);
};

// ── 点击结果：切换到对应频道 ──
const jumpTo = (msg) => {
    console.log(
        "[搜索点击]",
        msg.id,
        msg.channel,
        msg.content.substring(0, 20),
    );
    // 不关闭面板 — 让用户可以继续点击其他结果进行比较
    // 如果结果在其他频道，自动切换过去
    if (msg.channel !== currentChannel.value) {
        switchChannel(msg.channel);
    }
    // 通知 MessageList 滚动到该消息
    scrollToId.value = msg.id;
};

// ── 高亮搜索关键词 ──
const highlight = (text) => {
    if (!query.value.trim()) return escapeHtml(text);
    const escapedQuery = query.value
        .trim()
        .replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(`(${escapedQuery})`, "gi");
    return escapeHtml(text).replace(
        regex,
        '<mark class="bg-warning/30 text-warning-content rounded px-0.5">$1</mark>',
    );
};

const escapeHtml = (str) => {
    const div = document.createElement("div");
    div.textContent = str;
    return div.innerHTML;
};

const fmtTime = (iso) => {
    if (!iso) return "";
    const d = new Date(iso);
    const now = new Date();
    const diff = now - d;
    if (diff < 864e5 && d.getDate() === now.getDate()) {
        return d.toLocaleTimeString("zh-CN", {
            hour: "2-digit",
            minute: "2-digit",
        });
    }
    return d.toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
};
</script>

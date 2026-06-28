<template>
    <div class="flex min-h-screen bg-base-100">
        <!-- 侧边栏 -->
        <aside
            class="w-64 bg-base-100/80 backdrop-blur-xl border-r border-base-200/60 flex flex-col shrink-0 shadow-sm z-10"
        >
            <div class="p-6 border-b border-base-200/60">
                <h2
                    class="text-xl font-bold flex items-center gap-2 text-base-content"
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
                            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                        ></path>
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                        ></path>
                    </svg>
                    管理后台
                </h2>
                <p
                    class="text-xs text-base-content/50 mt-1.5 font-medium flex items-center gap-1"
                >
                    <span class="w-1.5 h-1.5 rounded-full bg-success"></span>
                    {{ username }}
                </p>
            </div>
            <nav class="flex-1 p-4 space-y-1.5 overflow-y-auto">
                <router-link
                    to="/admin/dashboard"
                    class="flex items-center gap-3 px-3 py-2.5 rounded-xl hover:bg-base-200/50 transition-colors text-sm font-medium text-base-content/70"
                    :class="
                        $route.path === '/admin/dashboard'
                            ? 'bg-primary/10 text-primary font-bold shadow-sm shadow-primary/5'
                            : ''
                    "
                >
                    <svg
                        class="w-5 h-5 opacity-70"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"
                        ></path>
                    </svg>
                    仪表盘
                </router-link>
                <router-link
                    to="/admin/users"
                    class="flex items-center gap-3 px-3 py-2.5 rounded-xl hover:bg-base-200/50 transition-colors text-sm font-medium text-base-content/70"
                    :class="
                        $route.path === '/admin/users'
                            ? 'bg-primary/10 text-primary font-bold shadow-sm shadow-primary/5'
                            : ''
                    "
                >
                    <svg
                        class="w-5 h-5 opacity-70"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
                        ></path>
                    </svg>
                    用户管理
                </router-link>
                <router-link
                    to="/admin/channels"
                    class="flex items-center gap-3 px-3 py-2.5 rounded-xl hover:bg-base-200/50 transition-colors text-sm font-medium text-base-content/70"
                    :class="
                        $route.path === '/admin/channels'
                            ? 'bg-primary/10 text-primary font-bold shadow-sm shadow-primary/5'
                            : ''
                    "
                >
                    <svg
                        class="w-5 h-5 opacity-70"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z"
                        ></path>
                    </svg>
                    频道管理
                </router-link>
                <router-link
                    to="/admin/messages"
                    class="flex items-center gap-3 px-3 py-2.5 rounded-xl hover:bg-base-200/50 transition-colors text-sm font-medium text-base-content/70"
                    :class="
                        $route.path === '/admin/messages'
                            ? 'bg-primary/10 text-primary font-bold shadow-sm shadow-primary/5'
                            : ''
                    "
                >
                    <svg
                        class="w-5 h-5 opacity-70"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                        ></path>
                    </svg>
                    消息审计
                </router-link>
                <router-link
                    to="/admin/logs"
                    class="flex items-center gap-3 px-3 py-2.5 rounded-xl hover:bg-base-200/50 transition-colors text-sm font-medium text-base-content/70"
                    :class="
                        $route.path === '/admin/logs'
                            ? 'bg-primary/10 text-primary font-bold shadow-sm shadow-primary/5'
                            : ''
                    "
                >
                    <svg
                        class="w-5 h-5 opacity-70"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"
                        ></path>
                    </svg>
                    操作日志
                </router-link>
                <router-link
                    to="/admin/settings"
                    class="flex items-center gap-3 px-3 py-2.5 rounded-xl hover:bg-base-200/50 transition-colors text-sm font-medium text-base-content/70"
                    :class="
                        $route.path === '/admin/settings'
                            ? 'bg-primary/10 text-primary font-bold shadow-sm shadow-primary/5'
                            : ''
                    "
                >
                    <svg
                        class="w-5 h-5 opacity-70"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"
                        ></path>
                    </svg>
                    系统设置
                </router-link>
            </nav>
            <div class="p-6 border-t border-base-200/60">
                <router-link
                    to="/"
                    class="btn btn-ghost w-full rounded-xl gap-2 font-medium hover:bg-base-200/50"
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
                            d="M10 19l-7-7m0 0l7-7m-7 7h18"
                        ></path>
                    </svg>
                    返回聊天
                </router-link>
            </div>
        </aside>

        <!-- 内容区 -->
        <main class="flex-1 overflow-auto bg-base-200/30">
            <router-view />
        </main>
    </div>
</template>

<script setup>
import { useAppState } from "../../composables/useAppState";
const { username } = useAppState();
</script>

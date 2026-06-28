<template>
    <div class="drawer lg:drawer-open h-screen bg-base-100">
        <input id="sidebar-toggle" type="checkbox" class="drawer-toggle" />

        <!-- 主内容区 -->
        <div class="drawer-content flex flex-col overflow-hidden relative">
            <!-- 移动端顶部栏 -->
            <div
                class="flex items-center justify-between px-5 py-3 bg-base-100/80 backdrop-blur-xl border-b border-base-200 lg:hidden sticky top-0 z-20 shadow-sm"
            >
                <div class="flex items-center gap-3">
                    <label
                        for="sidebar-toggle"
                        class="btn btn-ghost btn-sm btn-square drawer-button hover:bg-base-200/50"
                    >
                        <svg
                            class="w-5 h-5 text-base-content/80"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
                            />
                        </svg>
                    </label>
                    <div class="flex items-center gap-1.5">
                        <span class="text-xl font-bold text-primary">#</span>
                        <span class="font-bold text-base text-base-content">{{
                            currentChannel
                        }}</span>
                    </div>
                </div>
            </div>

            <ChatHeader />
            <div
                v-if="isGuest && !guestBannerDismissed"
                class="alert rounded-none border-0 px-4 py-2 text-sm"
                :class="bannerConfirming ? 'alert-error' : 'alert-warning'"
            >
                <svg
                    class="w-5 h-5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z"
                    />
                </svg>
                <span v-if="!bannerConfirming" class="flex-1"
                    >你正在以访客身份体验，账号和消息将在 24
                    小时后自动清除。</span
                >
                <span v-else class="flex-1"
                    >关闭后将不再显示该提醒，确定关闭吗？</span
                >
                <template v-if="!bannerConfirming">
                    <button
                        class="btn btn-ghost btn-xs"
                        @click="bannerConfirming = true"
                    >
                        ✕
                    </button>
                </template>
                <template v-else>
                    <button
                        class="btn btn-ghost btn-xs"
                        @click="bannerConfirming = false"
                    >
                        取消
                    </button>
                    <button
                        class="btn btn-sm btn-xs"
                        @click="guestBannerDismissed = true"
                    >
                        确认关闭
                    </button>
                </template>
            </div>
            <div
                v-if="
                    !isGuest &&
                    currentChannel === 'general' &&
                    !generalNoticeDismissed
                "
                class="alert alert-info rounded-none border-0 px-4 py-2 text-sm"
            >
                <svg
                    class="w-5 h-5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M11.25 11.25l.041-.02a.75.75 0 011.063.852l-.708 2.836a.75.75 0 001.063.853l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z"
                    />
                </svg>
                <span class="flex-1"
                    >#general
                    为公开频道，访客可查看此频道的消息，请注意信息安全。</span
                >
                <button
                    class="btn btn-ghost btn-xs"
                    @click="generalNoticeDismissed = true"
                >
                    ✕
                </button>
            </div>
            <!-- 私聊面板 -->
            <template v-if="activeDmId">
                <div class="flex items-center gap-3 px-6 py-3 border-b border-base-content/10 bg-base-100/80 backdrop-blur-xl shrink-0">
                    <button @click="closeDm" class="btn btn-ghost btn-sm btn-circle text-lg">←</button>
                    <span class="font-bold text-base-content truncate">@{{ dmOtherName }}</span>
                    <span class="text-xs" :class="dmConnected ? 'text-success' : 'text-error'">{{ dmConnected ? '●' : '○' }}</span>
                </div>
                <div ref="dmMsgList" class="flex-1 overflow-y-auto px-4 py-3 space-y-3 scrollbar-hide">
                    <div v-for="msg in dmMessages" :key="msg.id" class="flex gap-2" :class="msg.sender_username === username ? 'justify-end' : ''">
                        <div class="max-w-[70%] rounded-2xl px-4 py-2 text-sm" :class="msg.sender_username === username ? 'bg-primary text-primary-content ml-12' : 'bg-base-200 text-base-content mr-12'">
                            <p class="text-[10px] font-bold opacity-70 mb-0.5">{{ msg.sender_username }}</p>
                            <p class="break-words">{{ msg.content }}</p>
                        </div>
                    </div>
                </div>
                <div class="px-4 py-3 border-t border-base-content/10 bg-base-100/80 backdrop-blur-md shrink-0">
                    <div class="flex gap-2">
                        <input v-model="dmText" @keyup.enter="sendDm" placeholder="发送私聊消息..." maxlength="2000"
                            class="input input-bordered flex-1 input-sm" :disabled="!dmConnected" />
                        <button @click="sendDm" class="btn btn-primary btn-sm" :disabled="!dmText.trim() || !dmConnected">发送</button>
                    </div>
                </div>
            </template>
            <template v-else>
            <MessageList />
            <MessageInput />
            </template>
        </div>

        <!-- 侧边栏 -->
        <div class="drawer-side z-40">
            <label for="sidebar-toggle" class="drawer-overlay"></label>
            <div class="h-full bg-base-100/95 backdrop-blur-xl shadow-2xl">
                <Sidebar />
            </div>
        </div>

        <CreateChannelModal
            :show="showCreateModal"
            @close="showCreateModal = false"
            @created="showCreateModal = false"
        />
    </div>
</template>

<script setup>
import { ref, computed } from "vue";
import { useAppState } from "../composables/useAppState";
import ChatHeader from "./ChatHeader.vue";
import MessageList from "./MessageList.vue";
import MessageInput from "./MessageInput.vue";
import Sidebar from "./Sidebar.vue";
import CreateChannelModal from "./CreateChannelModal.vue";
import { useDm } from "../composables/useDm";

const { currentChannel, showCreateModal, isGuest, username } = useAppState();
const { activeConvId: activeDmId, messages: dmMessages, connected: dmConnected, conversations: dmConvs, sendDmMessage, disconnectDm } = useDm();
const guestBannerDismissed = ref(false);
const bannerConfirming = ref(false);
const generalNoticeDismissed = ref(false);
const dmText = ref('');

const dmOtherName = computed(() => {
  if (!activeDmId.value) return '';
  const c = dmConvs.value.find(c => c.conversation_id === activeDmId.value);
  return c ? (c.other_display_name || c.other_username) : '';
});

const closeDm = () => {
  disconnectDm();
};

const sendDm = () => {
  const text = dmText.value.trim();
  if (!text || !dmConnected.value) return;
  sendDmMessage(text);
  dmText.value = '';
};
</script>

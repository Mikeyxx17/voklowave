<template>
    <Teleport to="body">
        <Transition name="modal-fade">
            <div
                v-if="show"
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/10"
                @click.self="close"
            >
                <div
                    class="bg-base-100 rounded-2xl shadow-2xl border border-base-200 w-full max-w-lg mx-4 p-6 modal-box-inner"
                >
                    <h3
                        class="text-lg font-bold mb-4 text-center text-base-content select-none flex items-center justify-center gap-2"
                    >
                        <div
                            class="w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center text-primary"
                        >
                            <svg
                                class="w-3 h-3"
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
                        </div>
                        外观设置
                    </h3>

                    <p
                        class="text-[10px] text-base-content/60 font-medium mb-2 select-none text-center"
                    >
                        选择适合您的界面主题风格
                    </p>
                    <div class="grid grid-cols-7 gap-0.5">
                        <button
                            v-for="t in themes"
                            :key="t.value"
                            class="flex flex-col items-center justify-center gap-0.5 aspect-square py-0.5 px-0.5 rounded-xl border text-center transition-all duration-300 hover:bg-base-200/50 focus:outline-none group"
                            :class="
                                theme === t.value
                                    ? 'border-primary bg-primary/5 shadow-sm ring-1 ring-primary/20'
                                    : 'border-base-200 hover:border-base-300 hover:shadow-sm'
                            "
                            @click="selectTheme(t.value)"
                        >
                            <div
                                class="w-5 h-5 rounded-full shadow-sm shrink-0 border border-base-200 bg-base-100 flex items-center justify-center text-xs transition-transform duration-300 group-hover:scale-110"
                            >
                                {{ t.emoji }}
                            </div>
                            <span
                                class="text-[9px] font-semibold truncate w-full transition-colors"
                                :class="
                                    theme === t.value
                                        ? 'text-primary'
                                        : 'text-base-content'
                                "
                                >{{ t.label }}</span
                            >
                        </button>
                    </div>

                    <div class="mt-4 flex justify-end">
                        <button
                            class="btn btn-ghost btn-sm rounded-xl bg-base-200/50 hover:bg-base-300 transition-colors"
                            @click="close"
                        >
                            关闭
                        </button>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<script setup>
import { ref } from "vue";
import { useAppState } from "../composables/useAppState";

const { theme } = useAppState();

const show = ref(false);

const selectTheme = (value) => {
    theme.value = value;
};

const open = () => {
    show.value = true;
};

const close = () => {
    show.value = false;
};

defineExpose({ open });

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
</script>

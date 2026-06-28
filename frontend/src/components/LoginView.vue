<template>
    <div
        class="min-h-screen flex items-center justify-center relative overflow-hidden bg-base-100"
    >
        <div
            class="absolute top-[-20%] left-[-10%] w-[600px] h-[600px] bg-primary/12 rounded-full blur-[120px] animate-pulse"
        />
        <div
            class="absolute bottom-[-20%] right-[-10%] w-[600px] h-[600px] bg-secondary/10 rounded-full blur-[120px] animate-pulse"
            style="animation-delay: 1.5s"
        />
        <div
            class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-accent/10 rounded-full blur-[150px]"
        />

        <div class="relative z-10 w-full max-w-md mx-4">
            <div
                class="card bg-base-100/70 backdrop-blur-2xl border border-base-300/40 shadow-2xl"
            >
                <div class="card-body items-center text-center gap-5 p-10">
                    <div class="mb-1">
                        <div
                            class="w-20 h-20 mx-auto rounded-2xl bg-gradient-to-br from-primary to-secondary flex items-center justify-center shadow-xl shadow-primary/25 hover:scale-105 transition-transform duration-500"
                        >
                            <svg
                                class="w-10 h-10 text-primary-content"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="1.8"
                                    d="M8.625 9.75a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H8.25m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H12m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0h-.375m-13.5 3.01c0 1.6 1.123 2.994 2.707 3.227 1.087.16 2.185.283 3.293.369V21l4.184-4.183a1.14 1.14 0 01.778-.332 48.294 48.294 0 005.83-.498c1.585-.233 2.708-1.626 2.708-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0012 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018z"
                                />
                            </svg>
                        </div>
                    </div>

                    <h1
                        class="text-3xl font-black bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent"
                    >
                        VokloWave
                    </h1>
                    <p class="text-base-content/50 -mt-2 text-sm font-medium">
                        高效协作，即时连接
                    </p>

                    <div class="tabs tabs-box bg-base-200/50 w-full">
                        <a
                            v-for="tab in tabs"
                            :key="tab.key"
                            class="tab flex-1 text-xs font-semibold"
                            :class="mode === tab.key ? 'tab-active' : ''"
                            @click="switchMode(tab.key)"
                            >{{ tab.label }}</a
                        >
                    </div>

                    <template v-if="mode === 'login'">
                        <div class="form-control w-full">
                            <label class="label pb-1.5">
                                <span
                                    class="label-text text-base-content/60 text-xs font-bold uppercase tracking-wider select-none"
                                    >邮箱</span
                                >
                            </label>
                            <input
                                v-model="loginEmail"
                                class="input h-12 w-full bg-base-200/50 border-transparent focus:bg-base-100 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 transition-all rounded-xl text-sm"
                                placeholder="请输入邮箱"
                                type="email"
                                @keyup.enter="doLogin"
                            />
                        </div>
                        <div class="form-control w-full">
                            <label class="label pb-1.5">
                                <span
                                    class="label-text text-base-content/60 text-xs font-bold uppercase tracking-wider select-none"
                                    >密码</span
                                >
                            </label>
                            <input
                                v-model="loginPassword"
                                class="input h-12 w-full bg-base-200/50 border-transparent focus:bg-base-100 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 transition-all rounded-xl text-sm"
                                placeholder="请输入密码"
                                type="password"
                                @keyup.enter="doLogin"
                            />
                        </div>
                        <button
                            class="btn btn-primary h-12 w-full text-base rounded-xl shadow-sm shadow-primary/20 hover:shadow-md hover:shadow-primary/30 transition-all"
                            :class="
                                loginEmail.trim() && loginPassword
                                    ? 'hover:scale-[1.02]'
                                    : ''
                            "
                            :disabled="
                                !loginEmail.trim() ||
                                !loginPassword ||
                                authLoading
                            "
                            @click="doLogin"
                        >
                            <span
                                v-if="authLoading"
                                class="loading loading-spinner loading-sm"
                            ></span>
                            <span v-else>登录</span>
                        </button>
                        <button
                            class="btn btn-link btn-sm text-base-content/40 p-0 h-auto min-h-0 font-medium hover:text-base-content/70 no-underline"
                            @click="switchToForgot"
                        >
                            忘记密码？
                        </button>
                    </template>

                    <template v-if="mode === 'register'">
                        <div class="form-control w-full">
                            <label class="label pb-1.5">
                                <span
                                    class="label-text text-base-content/60 text-xs font-bold uppercase tracking-wider select-none"
                                    >用户名</span
                                >
                            </label>
                            <input
                                v-model="regUsername"
                                class="input h-12 w-full bg-base-200/50 border-transparent focus:bg-base-100 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 transition-all rounded-xl text-sm"
                                placeholder="请输入用户名"
                                maxlength="20"
                                @keyup.enter="doRegister"
                            />
                        </div>
                        <div class="form-control w-full">
                            <label class="label pb-1.5">
                                <span
                                    class="label-text text-base-content/60 text-xs font-bold uppercase tracking-wider select-none"
                                    >邮箱</span
                                >
                            </label>
                            <div
                                class="flex items-stretch w-full bg-base-200/50 rounded-xl focus-within:bg-base-100 focus-within:ring-2 focus-within:ring-primary/20 transition-all border border-transparent focus-within:border-primary/50 overflow-visible"
                            >
                                <input
                                    v-model="regEmailLocal"
                                    class="input h-12 bg-transparent border-none outline-none focus:outline-none rounded-r-none flex-[4] min-w-0"
                                    placeholder="邮箱前缀"
                                    @keyup.enter="doRegister"
                                />
                                <div
                                    class="flex items-center justify-center w-8 bg-transparent text-base-content/40 text-sm font-bold pointer-events-none select-none shrink-0"
                                >
                                    @
                                </div>
                                <!-- 自定义下拉：邮箱域名 -->
                                <div class="relative flex-[5] min-w-0">
                                    <button
                                        class="flex items-center justify-between w-full h-12 px-3 text-sm bg-transparent border-none outline-none hover:bg-base-200/50 transition-colors rounded-r-xl"
                                        :class="
                                            !regEmailDomain
                                                ? 'text-base-content/50'
                                                : 'text-base-content'
                                        "
                                        type="button"
                                        @click="
                                            showEmailDropdown =
                                                !showEmailDropdown
                                        "
                                        @blur="onEmailDropdownBlur"
                                    >
                                        <span class="truncate">{{
                                            regEmailDomain || "选择邮箱"
                                        }}</span>
                                        <svg
                                            class="w-4 h-4 opacity-50 shrink-0 ml-1 transition-transform duration-200"
                                            :class="
                                                showEmailDropdown
                                                    ? 'rotate-180'
                                                    : ''
                                            "
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
                                    <!-- 下拉列表 -->
                                    <div
                                        v-if="showEmailDropdown"
                                        class="absolute left-0 top-full mt-1 w-full z-[100] rounded-2xl border border-base-content/10 shadow-2xl bg-base-100/40 backdrop-blur-2xl overflow-hidden"
                                    >
                                        <div
                                            class="max-h-48 overflow-y-auto overflow-x-hidden py-1.5"
                                        >
                                            <button
                                                v-for="d in emailDomains"
                                                :key="d"
                                                class="w-full text-left px-4 py-2.5 text-[13px] font-medium transition-colors hover:bg-base-content/10"
                                                :class="
                                                    regEmailDomain === d
                                                        ? 'bg-primary/20 text-primary font-bold'
                                                        : 'text-base-content/80'
                                                "
                                                type="button"
                                                @mousedown.prevent="
                                                    selectEmailDomain(d)
                                                "
                                            >
                                                {{ d }}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="form-control w-full">
                            <label class="label pb-1.5">
                                <span
                                    class="label-text text-base-content/60 text-xs font-bold uppercase tracking-wider select-none"
                                    >密码</span
                                >
                            </label>
                            <input
                                v-model="regPassword"
                                class="input h-12 w-full bg-base-200/50 border-transparent focus:bg-base-100 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 transition-all rounded-xl text-sm"
                                placeholder="请输入密码"
                                type="password"
                                @keyup.enter="doRegister"
                            />
                        </div>
                        <button
                            class="btn btn-secondary h-12 w-full text-base rounded-xl shadow-sm shadow-secondary/20 hover:shadow-md hover:shadow-secondary/30 transition-all"
                            :class="
                                regUsername.trim() && regEmail && regPassword
                                    ? 'hover:scale-[1.02]'
                                    : ''
                            "
                            :disabled="
                                !regUsername.trim() ||
                                !regEmail ||
                                !regPassword ||
                                authLoading
                            "
                            @click="doRegister"
                        >
                            <span
                                v-if="authLoading"
                                class="loading loading-spinner loading-sm"
                            ></span>
                            <span v-else>注册并登录</span>
                        </button>
                    </template>

                    <template v-if="mode === 'verify'">
                        <div
                            class="flex flex-col items-center w-full gap-4 py-2"
                        >
                            <div
                                class="w-16 h-16 bg-success/10 rounded-full flex items-center justify-center mb-2 text-success"
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
                                        stroke-width="2"
                                        d="M21.75 6.75v10.5a2.25 2.25 0 01-2.25 2.25h-15a2.25 2.25 0 01-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25m19.5 0v.243a2.25 2.25 0 01-1.07 1.916l-7.5 4.615a2.25 2.25 0 01-2.36 0L3.32 8.91a2.25 2.25 0 01-1.07-1.916V6.75"
                                    />
                                </svg>
                            </div>
                            <p class="text-sm text-base-content/70">
                                验证码已发送至
                            </p>
                            <p class="text-sm font-bold text-base-content">
                                {{ verifyEmailAddr }}
                            </p>

                            <div class="join w-full">
                                <input
                                    v-model="verifyCode"
                                    class="input h-14 w-full bg-base-200/50 border-transparent focus:bg-base-100 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 transition-all rounded-xl text-center text-2xl tracking-[0.5em] font-mono"
                                    placeholder="000000"
                                    maxlength="6"
                                    @keyup.enter="doVerify"
                                />
                            </div>
                            <p
                                v-if="verifyMsg"
                                class="text-xs font-medium"
                                :class="
                                    verifyOk ? 'text-success' : 'text-error'
                                "
                            >
                                {{ verifyMsg }}
                            </p>

                            <button
                                class="btn btn-success h-12 w-full text-base rounded-xl shadow-sm shadow-success/20 hover:shadow-md hover:shadow-success/30 transition-all"
                                :disabled="
                                    verifyCode.length !== 6 || verifyLoading
                                "
                                @click="doVerify"
                            >
                                <span
                                    v-if="verifyLoading"
                                    class="loading loading-spinner loading-sm"
                                ></span>
                                <span v-else>激活账号</span>
                            </button>

                            <div
                                class="flex items-center gap-1 text-xs text-base-content/50"
                            >
                                <span>没有收到邮件？</span>
                                <button
                                    class="btn btn-link btn-xs text-primary p-0 h-auto min-h-0 underline underline-offset-2"
                                    :disabled="
                                        resendLoading || resendCooldown > 0
                                    "
                                    @click="doResend"
                                >
                                    <span
                                        v-if="resendLoading"
                                        class="loading loading-spinner loading-xs"
                                    ></span>
                                    <span v-else-if="resendCooldown > 0"
                                        >重新发送 ({{ resendCooldown }}s)</span
                                    >
                                    <span v-else>重新发送</span>
                                </button>
                            </div>

                            <button
                                class="btn btn-ghost btn-sm text-base-content/40"
                                @click="
                                    switchMode('login');
                                    clearVerify();
                                "
                            >
                                返回登录
                            </button>
                        </div>
                    </template>

                    <template v-if="mode === 'forgot'">
                        <template v-if="forgotStep === 1">
                            <div
                                class="flex flex-col items-center w-full gap-4 py-2"
                            >
                                <div
                                    class="w-16 h-16 bg-warning/10 rounded-full flex items-center justify-center mb-2 text-warning"
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
                                            stroke-width="2"
                                            d="M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z"
                                        />
                                    </svg>
                                </div>
                                <p class="text-sm text-base-content/70">
                                    请输入您的注册邮箱
                                </p>
                                <div class="form-control w-full">
                                    <input
                                        v-model="forgotEmail"
                                        class="input h-12 w-full bg-base-200/50 border-transparent focus:bg-base-100 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 transition-all rounded-xl"
                                        placeholder="请输入邮箱"
                                        type="email"
                                        @keyup.enter="doForgotPassword"
                                    />
                                </div>
                                <p
                                    v-if="forgotMsg"
                                    class="text-xs font-medium"
                                    :class="
                                        forgotOk ? 'text-success' : 'text-error'
                                    "
                                >
                                    {{ forgotMsg }}
                                </p>
                                <button
                                    class="btn btn-primary h-12 w-full text-base rounded-xl shadow-sm shadow-primary/20 hover:shadow-md hover:shadow-primary/30 transition-all"
                                    :disabled="
                                        !forgotEmail.trim() || forgotLoading
                                    "
                                    @click="doForgotPassword"
                                >
                                    <span
                                        v-if="forgotLoading"
                                        class="loading loading-spinner loading-sm"
                                    ></span>
                                    <span v-else>发送重置验证码</span>
                                </button>
                                <button
                                    class="btn btn-ghost btn-sm text-base-content/40"
                                    @click="
                                        switchMode('login');
                                        clearForgot();
                                    "
                                >
                                    返回登录
                                </button>
                            </div>
                        </template>

                        <template v-if="forgotStep === 2">
                            <div
                                class="flex flex-col items-center w-full gap-4 py-2"
                            >
                                <div
                                    class="w-16 h-16 bg-success/10 rounded-full flex items-center justify-center mb-2 text-success"
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
                                            stroke-width="2"
                                            d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                        />
                                    </svg>
                                </div>
                                <p class="text-sm text-base-content/70">
                                    验证码已发送至
                                </p>
                                <p class="text-sm font-bold text-base-content">
                                    {{ forgotEmail }}
                                </p>

                                <div class="form-control w-full">
                                    <input
                                        v-model="forgotCode"
                                        class="input h-14 w-full bg-base-200/50 border-transparent focus:bg-base-100 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 transition-all rounded-xl text-center text-2xl tracking-[0.5em] font-mono"
                                        placeholder="000000"
                                        maxlength="6"
                                        @keyup.enter="focusForgotPassword"
                                    />
                                </div>

                                <div class="form-control w-full">
                                    <input
                                        ref="forgotPasswordInput"
                                        v-model="forgotNewPassword"
                                        class="input h-12 w-full bg-base-200/50 border-transparent focus:bg-base-100 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 transition-all rounded-xl"
                                        placeholder="请输入新密码"
                                        type="password"
                                        @keyup.enter="doResetPassword"
                                    />
                                </div>

                                <p
                                    v-if="forgotMsg"
                                    class="text-xs font-medium"
                                    :class="
                                        forgotOk ? 'text-success' : 'text-error'
                                    "
                                >
                                    {{ forgotMsg }}
                                </p>

                                <button
                                    class="btn btn-success h-12 w-full text-base rounded-xl shadow-sm shadow-success/20 hover:shadow-md hover:shadow-success/30 transition-all"
                                    :disabled="
                                        forgotCode.length !== 6 ||
                                        !forgotNewPassword ||
                                        forgotLoading
                                    "
                                    @click="doResetPassword"
                                >
                                    <span
                                        v-if="forgotLoading"
                                        class="loading loading-spinner loading-sm"
                                    ></span>
                                    <span v-else>重置密码</span>
                                </button>

                                <button
                                    class="btn btn-ghost btn-sm text-base-content/40"
                                    @click="
                                        switchMode('login');
                                        clearForgot();
                                    "
                                >
                                    返回登录
                                </button>
                            </div>
                        </template>
                    </template>

                    <template v-if="mode === 'quick'">
                        <div
                            class="flex flex-col items-center justify-center w-full py-4"
                        >
                            <div
                                class="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mb-4 text-primary"
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
                                        stroke-width="2"
                                        d="M13 10V3L4 14h7v7l9-11h-7z"
                                    />
                                </svg>
                            </div>
                            <p class="text-sm text-base-content/60 mb-2 px-4">
                                无需注册，一键加入 #general
                                公开频道，仅可查看和发送消息。
                            </p>
                            <p
                                class="text-xs text-warning/80 mb-6 px-4 flex items-center justify-center gap-1"
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
                                        d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z"
                                    />
                                </svg>
                                访客账号和消息将在 24 小时后自动清除
                            </p>
                            <button
                                class="btn btn-primary h-12 w-full text-base rounded-field hover:scale-[1.02]"
                                :disabled="authLoading"
                                @click="handleQuickJoin"
                            >
                                <span
                                    v-if="authLoading"
                                    class="loading loading-spinner loading-sm"
                                ></span>
                                <span v-else>🚀 一键加入聊天</span>
                            </button>
                        </div>
                    </template>

                    <p
                        v-if="authError"
                        class="text-error text-xs font-medium -mb-2"
                    >
                        {{ authError }}
                    </p>

                    <div class="divider divider-neutral/20 my-0"></div>

                    <div class="dropdown dropdown-top w-full">
                        <div
                            tabindex="0"
                            role="button"
                            class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-base-content/60 bg-base-200/50 border border-base-300 hover:bg-base-200 transition-colors cursor-pointer w-full"
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
                                    d="M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z"
                                />
                            </svg>
                            <span class="truncate">{{ themeLabel }}</span>
                            <svg
                                class="w-3 h-3 ml-auto shrink-0"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M4.5 15.75l7.5-7.5 7.5 7.5"
                                />
                            </svg>
                        </div>
                        <ul
                            tabindex="0"
                            class="dropdown-content z-50 menu p-1.5 shadow-xl bg-base-200 rounded-xl border border-base-300/50 w-full mb-1 max-h-60 overflow-y-auto"
                        >
                            <li v-for="t in themes" :key="t.value">
                                <a
                                    class="text-xs py-2 rounded-lg"
                                    :class="
                                        theme === t.value
                                            ? 'bg-primary/15 text-primary font-semibold'
                                            : ''
                                    "
                                    @click="selectTheme(t.value)"
                                    >{{ t.label }}</a
                                >
                            </li>
                        </ul>
                    </div>

                    <p class="text-[11px] text-base-content/30">
                        Enter 快速提交 · 欢乐交流，文明发言
                    </p>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
import { ref, computed } from "vue";
import { useAppState } from "../composables/useAppState";

// 注意这里解构出了 quickExperience 和验证相关方法
const {
    login,
    register,
    verifyEmail,
    resendVerification,
    theme,
    authError,
    pendingEmail,
    quickExperience,
    forgotPassword,
    resetPassword,
} = useAppState();

const mode = ref("login");
const authLoading = ref(false);

// ── 登录表单 ──
const loginEmail = ref("");
const loginPassword = ref("");

const doLogin = async () => {
    if (!loginEmail.value.trim() || !loginPassword.value) return;
    authLoading.value = true;
    const result = await login(loginEmail.value.trim(), loginPassword.value);
    if (result.needVerify) {
        mode.value = "verify";
    }
    authLoading.value = false;
};

// ── 注册表单 ──
const regUsername = ref("");
const regEmailLocal = ref("");
const regEmailDomain = ref("");
const regPassword = ref("");

const emailDomains = [
    "qq.com",
    "gmail.com",
    "outlook.com",
    "163.com",
    "126.com",
    "yeah.net",
    "proton.me",
    "protonmail.com",
    "icloud.com",
    "hotmail.com",
    "live.com",
    "foxmail.com",
    "sina.com",
    "sohu.com",
];

const showEmailDropdown = ref(false);

const selectEmailDomain = (d) => {
    regEmailDomain.value = d;
    showEmailDropdown.value = false;
};

const onEmailDropdownBlur = () => {
    // 延迟关闭，让 mousedown 先触发
    setTimeout(() => {
        showEmailDropdown.value = false;
    }, 150);
};

const regEmail = computed(() => {
    if (!regEmailLocal.value.trim() || !regEmailDomain.value) return "";
    return regEmailLocal.value.trim() + "@" + regEmailDomain.value;
});

const doRegister = async () => {
    if (!regUsername.value.trim() || !regEmail.value || !regPassword.value)
        return;
    authLoading.value = true;
    const result = await register(
        regUsername.value.trim(),
        regEmail.value,
        regPassword.value,
    );
    if (result.ok) {
        pendingEmail.value = regEmail.value;
        mode.value = "verify";
        clearVerify();
        startResendCooldown();
    }
    authLoading.value = false;
};

// ── 验证码 ──
const verifyCode = ref("");
const verifyLoading = ref(false);
const verifyMsg = ref("");
const verifyOk = ref(false);
const verifyEmailAddr = computed(() => pendingEmail.value);

const clearVerify = () => {
    verifyCode.value = "";
    verifyMsg.value = "";
    verifyOk.value = false;
    resendCooldown.value = 0;
    clearInterval(resendCooldownTimer);
    resendCooldownTimer = null;
};

const doVerify = async () => {
    if (verifyCode.value.length !== 6) return;
    verifyLoading.value = true;
    verifyMsg.value = "";
    const result = await verifyEmail(pendingEmail.value, verifyCode.value);
    if (result.ok) {
        verifyOk.value = true;
        verifyMsg.value = result.message;
        // 2 秒后返回登录页
        setTimeout(() => {
            switchMode("login");
            loginEmail.value = pendingEmail.value;
            pendingEmail.value = "";
            clearVerify();
        }, 2000);
    } else {
        verifyOk.value = false;
        verifyMsg.value = result.error;
    }
    verifyLoading.value = false;
};

// ── 重新发送验证码 ──
const resendLoading = ref(false);
const resendCooldown = ref(0);
let resendCooldownTimer = null;

const startResendCooldown = () => {
    resendCooldown.value = 60;
    clearInterval(resendCooldownTimer);
    resendCooldownTimer = setInterval(() => {
        resendCooldown.value--;
        if (resendCooldown.value <= 0) {
            clearInterval(resendCooldownTimer);
            resendCooldownTimer = null;
        }
    }, 1000);
};

const doResend = async () => {
    if (!pendingEmail.value || resendLoading.value || resendCooldown.value > 0)
        return;
    resendLoading.value = true;
    verifyMsg.value = "";
    const result = await resendVerification(pendingEmail.value);
    if (result.ok) {
        verifyOk.value = true;
        verifyMsg.value = result.message;
        startResendCooldown();
    } else {
        verifyOk.value = false;
        verifyMsg.value = result.error;
    }
    resendLoading.value = false;
};

// ── 忘记密码 ──
const forgotEmail = ref("");
const forgotStep = ref(1);
const forgotCode = ref("");
const forgotNewPassword = ref("");
const forgotLoading = ref(false);
const forgotMsg = ref("");
const forgotOk = ref(false);
const forgotPasswordInput = ref(null);

const clearForgot = () => {
    forgotEmail.value = "";
    forgotStep.value = 1;
    forgotCode.value = "";
    forgotNewPassword.value = "";
    forgotMsg.value = "";
    forgotOk.value = false;
};

const switchToForgot = () => {
    clearForgot();
    forgotEmail.value = loginEmail.value;
    mode.value = "forgot";
};

const doForgotPassword = async () => {
    if (!forgotEmail.value.trim()) return;
    forgotLoading.value = true;
    forgotMsg.value = "";
    const result = await forgotPassword(forgotEmail.value.trim());
    if (result.ok) {
        forgotOk.value = true;
        forgotMsg.value = result.message;
        forgotStep.value = 2;
    } else {
        forgotOk.value = false;
        forgotMsg.value = result.error;
    }
    forgotLoading.value = false;
};

const focusForgotPassword = () => {
    forgotPasswordInput.value?.focus();
};

const doResetPassword = async () => {
    if (forgotCode.value.length !== 6 || !forgotNewPassword.value) return;
    forgotLoading.value = true;
    forgotMsg.value = "";
    const result = await resetPassword(
        forgotEmail.value,
        forgotCode.value,
        forgotNewPassword.value,
    );
    if (result.ok) {
        forgotOk.value = true;
        forgotMsg.value = result.message;
        // 2 秒后返回登录页，预填邮箱
        setTimeout(() => {
            switchMode("login");
            loginEmail.value = forgotEmail.value;
            clearForgot();
        }, 2000);
    } else {
        forgotOk.value = false;
        forgotMsg.value = result.error;
    }
    forgotLoading.value = false;
};

// ── 快速体验（调用后端 API 版）──
const handleQuickJoin = async () => {
    authLoading.value = true;
    await quickExperience();
    authLoading.value = false;
};

const switchMode = (key) => {
    mode.value = key;
    authError.value = "";
    if (key !== "verify") {
        clearVerify();
        pendingEmail.value = "";
    }
    if (key !== "forgot") {
        clearForgot();
    }
};

const selectTheme = (value) => {
    theme.value = value;
    document.activeElement?.blur();
};

const tabs = [
    { key: "login", label: "登录" },
    { key: "register", label: "注册" },
    { key: "quick", label: "快速体验" },
];

const themes = [
    { value: "dark", label: "🌙 深色" },
    { value: "light", label: "☀️ 浅色" },
    { value: "cyberpunk", label: "🤖 赛博朋克" },
    { value: "cupcake", label: "🧁 纸杯蛋糕" },
    { value: "synthwave", label: "🌴 合成波" },
    { value: "nord", label: "❄️ 北欧风" },
    { value: "sunset", label: "🌅 日落" },
    { value: "winter", label: "⛄ 冬季" },
    { value: "coffee", label: "☕ 咖啡" },
    { value: "lemonade", label: "🍋 柠檬水" },
    { value: "luxury", label: "👑 奢华金" },
    { value: "business", label: "💼 商务" },
    { value: "autumn", label: "🍂 秋季" },
    { value: "dim", label: "💜 暗紫" },
];

const themeLabel = computed(() => {
    const t = themes.find((t) => t.value === theme.value);
    return t ? t.label : "🌙 深色";
});
</script>

<script setup lang="ts">
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Password from 'primevue/password'
import { useToast } from 'primevue/usetoast'
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { login } from '@/services/auth'

const router = useRouter()
const username = ref('')
const password = ref('')
const loginLoading = ref(false)
const loginError = ref('')
const toast = useToast()

watch(loginError, (value) => {
    if (!value) {
        return
    }

    toast.add({ severity: 'error', summary: '登录失败', detail: value, life: 3000 })
})

async function submitLogin() {
    const trimmedName = username.value.trim()

    loginError.value = ''

    if (!trimmedName || !password.value) {
        loginError.value = '请输入用户名和密码'
        return
    }

    loginLoading.value = true

    try {
        await login(trimmedName, password.value)
        password.value = ''
        await router.replace({ name: 'metrics-panel' })
    } catch (err) {
        loginError.value = err instanceof Error ? err.message : '登录失败'
    } finally {
        loginLoading.value = false
    }
}
</script>

<template>
    <main class="auth-layout">
        <section class="login-card">
            <h1>风纪面板登录</h1>
            <p class="login-tip">使用管理员账号登录</p>

            <form class="login-form" @submit.prevent="submitLogin">
                <label class="field-label" for="username">用户名</label>
                <InputText id="username" v-model="username" autocomplete="username" class="field-input"
                    :disabled="loginLoading" placeholder="请输入用户名" />

                <label class="field-label" for="password">密码</label>
                <Password input-id="password" v-model="password" autocomplete="current-password" class="field-password"
                    inputClass="field-input" :feedback="false" :toggleMask="true" :disabled="loginLoading"
                    placeholder="请输入密码" />

                <Button type="submit" class="login-btn" :disabled="loginLoading"
                    :label="loginLoading ? '登录中...' : '登录'" />
            </form>
        </section>
    </main>
</template>

<style scoped>
.auth-layout {
    min-height: 100vh;
    display: grid;
    place-items: center;
    background: var(--surface-ground);
    padding: 1rem;
}

.login-card {
    width: min(100%, 420px);
    background: var(--surface-card);
    border: 1px solid var(--surface-border);
    border-radius: 1rem;
    box-shadow: 0 24px 60px rgba(15, 23, 42, 0.08);
    padding: 1.2rem 1.1rem;
}

.login-card h1 {
    margin: 0 0 0.5rem;
    font-size: 1.2rem;
    color: var(--text-color);
}

.login-tip {
    margin: 0 0 0.9rem;
    color: var(--text-color-secondary);
    font-size: 0.93rem;
}

.login-form {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
}

.field-label {
    color: var(--text-color-secondary);
    font-size: 0.9rem;
}

.field-input {
    width: 100%;
}

.field-password {
    width: 100%;
}

.field-password :deep(.p-inputtext) {
    width: 100%;
}

.field-input:deep(.p-inputtext) {
    border: 1px solid var(--surface-border);
    border-radius: 0.6rem;
    background: var(--surface-0);
    color: var(--text-color);
    padding: 0.6rem 0.7rem;
}

.login-btn {
    margin-top: 0.4rem;
    border-radius: 0.6rem;
}

.login-btn:disabled,
.field-input:deep(.p-inputtext:disabled) {
    cursor: not-allowed;
    opacity: 0.75;
}
</style>

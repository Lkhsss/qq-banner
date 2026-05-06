<script setup lang="ts">
import Button from 'primevue/button'
import IconField from 'primevue/iconfield'
import InputIcon from 'primevue/inputicon'
import InputText from 'primevue/inputtext'
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
    <div class="login-bg">
        <div class="login-card">
            <div class="login-header">
                <svg xmlns="http://www.w3.org/2000/svg" class="login-logo" width="33" height="32" viewBox="0 0 33 32"
                    fill="none">
                    <path fill-rule="evenodd" clip-rule="evenodd"
                        d="M7.09219 2.87829C5.94766 3.67858 4.9127 4.62478 4.01426 5.68992C7.6857 5.34906 12.3501 5.90564 17.7655 8.61335C23.5484 11.5047 28.205 11.6025 31.4458 10.9773C31.1517 10.087 30.7815 9.23135 30.343 8.41791C26.6332 8.80919 21.8772 8.29127 16.3345 5.51998C12.8148 3.76014 9.71221 3.03521 7.09219 2.87829ZM28.1759 5.33332C25.2462 2.06 20.9887 0 16.25 0C14.8584 0 13.5081 0.177686 12.2209 0.511584C13.9643 0.987269 15.8163 1.68319 17.7655 2.65781C21.8236 4.68682 25.3271 5.34013 28.1759 5.33332ZM32.1387 14.1025C28.2235 14.8756 22.817 14.7168 16.3345 11.4755C10.274 8.44527 5.45035 8.48343 2.19712 9.20639C2.0292 9.24367 1.86523 9.28287 1.70522 9.32367C1.2793 10.25 0.939308 11.2241 0.695362 12.2356C0.955909 12.166 1.22514 12.0998 1.50293 12.0381C5.44966 11.161 11.0261 11.1991 17.7655 14.5689C23.8261 17.5991 28.6497 17.561 31.9029 16.838C32.0144 16.8133 32.1242 16.7877 32.2322 16.7613C32.2441 16.509 32.25 16.2552 32.25 16C32.25 15.358 32.2122 14.7248 32.1387 14.1025ZM31.7098 20.1378C27.8326 20.8157 22.5836 20.5555 16.3345 17.431C10.274 14.4008 5.45035 14.439 2.19712 15.1619C1.475 15.3223 0.825392 15.5178 0.252344 15.7241C0.250782 15.8158 0.25 15.9078 0.25 16C0.25 24.8366 7.41344 32 16.25 32C23.6557 32 29.8862 26.9687 31.7098 20.1378Z"
                        class="fill-surface-0" />
                </svg>
                <div class="login-title">
                    <div class="login-title-main">Welcome Back</div>
                    <!-- <div class="login-title-sub">
                        <span>Don't have an account? </span>
                        <a class="login-link" role="button">Sign up</a>
                    </div> -->
                </div>
            </div>
            <form class="login-form" @submit.prevent="submitLogin">
                <div class="login-fields">
                    <IconField>
                        <InputIcon class="pi pi-user login-icon" />
                        <InputText v-model="username" type="text" autocomplete="username" class="login-input"
                            placeholder="Username" :disabled="loginLoading" />
                    </IconField>
                    <IconField>
                        <InputIcon class="pi pi-lock login-icon" />
                        <InputText v-model="password" type="password" autocomplete="current-password"
                            class="login-input" placeholder="Password" :disabled="loginLoading" />
                    </IconField>
                </div>
                <Button type="submit" :label="loginLoading ? 'Signing In...' : 'Sign In'" class="login-submit"
                    :disabled="loginLoading" />
            </form>
        </div>
    </div>
</template>

<style scoped>
.login-bg {
    min-height: 100vh;
    width: 100%;
    padding: 5rem 1.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background-image: url('/signin-glass.jpg');
    background-size: cover;
    background-position: center;
    background-repeat: no-repeat;
}

.login-card {
    width: 100%;
    max-width: 24.5rem;
    padding: 3rem 2rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3rem;
    border-radius: 1rem;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(24px);
    box-shadow: 0 24px 60px rgba(15, 23, 42, 0.2);
}

.login-header {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
}

.login-logo {
    width: 3.5rem;
    height: 3.5rem;
}

.login-title {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    text-align: center;
}

.login-title-main {
    font-size: 1.75rem;
    font-weight: 600;
    color: #fff;
    line-height: 1.2;
}

.login-title-sub {
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.95rem;
}

.login-form {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2rem;
}

.login-fields {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
}

:deep(.login-input) {
    width: 100%;
    border-radius: 9999px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
    padding: 0.65rem 0.9rem 0.65rem 2.2rem;
    outline: none;
    box-shadow: 0 8px 18px rgba(15, 23, 42, 0.15);
}

:deep(.login-input::placeholder) {
    color: rgba(255, 255, 255, 0.7);
}

:deep(.login-icon) {
    color: rgba(255, 255, 255, 0.7);
}

.login-submit {
    width: 100%;
    border-radius: 9999px;
    background: #0f172a;
    border: 1px solid #0f172a;
    color: #fff;
}

.login-submit:hover:enabled {
    background: rgba(15, 23, 42, 0.8);
}

.login-link {
    color: rgba(255, 255, 255, 0.8);
    cursor: pointer;
    text-decoration: underline;
}

.login-link:hover {
    color: rgba(255, 255, 255, 0.95);
}

@media (min-width: 768px) {
    .login-bg {
        padding: 5rem 5rem;
    }

    .login-card {
        padding: 3rem 3rem;
    }
}

@media (min-width: 1024px) {
    .login-bg {
        padding: 5rem 10rem;
    }

    .login-card {
        padding: 3rem 4rem;
    }
}
</style>

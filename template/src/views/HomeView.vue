<script setup lang="ts">
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Menu from 'primevue/menu'
import Password from 'primevue/password'
import { useToast } from 'primevue/usetoast'
import { computed, onMounted, ref, watch } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'
import { authChecking, checkAuthByProbe, isAuthenticated, login } from '@/services/auth'

const router = useRouter()
const route = useRoute()
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

const items = computed(() => [
  {
    label: '封禁管理',
    icon: 'pi pi-ban',
    command: () => router.push({ name: 'banned-list' }),
    styleClass: route.name === 'banned-list' ? 'menu-item-active' : '',
  },
  {
    label: '管理员管理',
    icon: 'pi pi-users',
    command: () => router.push({ name: 'manager-list' }),
    styleClass: route.name === 'manager-list' ? 'menu-item-active' : '',
  },
])

function toggleDarkMode() {
  document.documentElement.classList.toggle('my-app-dark')
}

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
  } catch (err) {
    loginError.value = err instanceof Error ? err.message : '登录失败'
  } finally {
    loginLoading.value = false
  }
}

onMounted(() => {
  void checkAuthByProbe()
})
</script>

<template>
  <main v-if="authChecking" class="auth-loading">正在检查登录状态...</main>

  <main v-else-if="!isAuthenticated" class="auth-layout">
    <section class="login-card">
      <h1>风纪面板登录</h1>
      <p class="login-tip">使用管理员账号登录后，将自动写入 cookie 并完成后续接口鉴权。</p>

      <form class="login-form" @submit.prevent="submitLogin">
        <label class="field-label" for="username">用户名</label>
        <InputText id="username" v-model="username" autocomplete="username" class="field-input" :disabled="loginLoading"
          placeholder="请输入用户名" />

        <label class="field-label" for="password">密码</label>
        <Password input-id="password" v-model="password" autocomplete="current-password" class="field-password"
          inputClass="field-input" :feedback="false" :toggleMask="true" :disabled="loginLoading" placeholder="请输入密码" />

        <Button type="submit" class="login-btn" :disabled="loginLoading" :label="loginLoading ? '登录中...' : '登录'" />
      </form>

    </section>
  </main>

  <main v-else class="home-layout">
    <aside class="home-sidebar">
      <div class="sidebar-header">
        <h1>风纪面板</h1>
        <Button type="button" class="theme-toggle" severity="secondary" variant="outlined" label="切换暗色"
          @click="toggleDarkMode" />
      </div>
      <Menu :model="items" class="side-menu" />
    </aside>

    <section class="home-content">
      <RouterView />
    </section>
  </main>
</template>

<style scoped>
.home-layout {
  min-height: 100vh;
  display: grid;
  grid-template-columns: 260px 1fr;
  background: linear-gradient(160deg, #f8fafc 0%, #eef2ff 45%, #e2e8f0 100%);
}

.home-sidebar {
  border-right: 1px solid #dbe3f1;
  background: rgba(255, 255, 255, 0.85);
  backdrop-filter: blur(8px);
  padding: 1rem;
}

.sidebar-header {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  margin-bottom: 1rem;
}

.sidebar-header h1 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 700;
}

.theme-toggle {
  border: 1px solid #cbd5e1;
  border-radius: 0.5rem;
  background: #fff;
  padding: 0.5rem 0.75rem;
  cursor: pointer;
}

.side-menu {
  border: none;
  width: 100%;
}

.home-content {
  padding: 1.5rem;
}

:deep(.menu-item-active > .p-menu-item-content) {
  background: #dbeafe;
  border-radius: 0.5rem;
}

.auth-loading,
.auth-layout {
  min-height: 100vh;
  display: grid;
  place-items: center;
  background: linear-gradient(145deg, #f8fafc 0%, #e0f2fe 45%, #f1f5f9 100%);
  padding: 1rem;
}

.login-card {
  width: min(100%, 420px);
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid #dbe3f1;
  border-radius: 1rem;
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.1);
  padding: 1.2rem 1.1rem;
}

.login-card h1 {
  margin: 0 0 0.5rem;
  font-size: 1.2rem;
}

.login-tip {
  margin: 0 0 0.9rem;
  color: #475569;
  font-size: 0.93rem;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
}

.field-label {
  color: #334155;
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
  border: 1px solid #cbd5e1;
  border-radius: 0.6rem;
  background: #fff;
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

.login-error {
  margin: 0.8rem 0 0;
  color: #b91c1c;
}

@media (max-width: 900px) {
  .home-layout {
    grid-template-columns: 1fr;
  }

  .home-sidebar {
    border-right: none;
    border-bottom: 1px solid #dbe3f1;
  }
}
</style>

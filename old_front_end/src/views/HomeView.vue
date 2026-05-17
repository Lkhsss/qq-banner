<script setup lang="ts">
import Button from 'primevue/button'
import Menu from 'primevue/menu'
import { computed } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'
import { Permission, currentPermission, hasPermission, logout } from '@/services/auth'

const router = useRouter()
const route = useRoute()

const items = computed(() => {
  const menu = [] as Array<{
    label: string
    icon: string
    command: () => void
    styleClass: string
  }>

  if (hasPermission(Permission.Admin)) {
    menu.push({
      label: '数据面板',
      icon: 'pi pi-chart-bar',
      command: () => router.push({ name: 'metrics-panel' }),
      styleClass: route.name === 'metrics-panel' ? 'menu-item-active' : '',
    })

    menu.push({
      label: '封禁管理',
      icon: 'pi pi-ban',
      command: () => router.push({ name: 'banned-list' }),
      styleClass: route.name === 'banned-list' ? 'menu-item-active' : '',
    })
  }

  if (hasPermission(Permission.SuperAdmin)) {
    menu.push({
      label: '管理员管理',
      icon: 'pi pi-users',
      command: () => router.push({ name: 'manager-list' }),
      styleClass: route.name === 'manager-list' ? 'menu-item-active' : '',
    })
  }

  return menu
})

const permissionLabel = computed(() => {
  if (currentPermission.value === Permission.SuperAdmin) {
    return '当前权限：SuperAdmin'
  }

  if (currentPermission.value === Permission.Admin) {
    return '当前权限：Admin'
  }

  if (currentPermission.value === Permission.User) {
    return '当前权限：User'
  }

  return '当前权限：未知'
})

function toggleDarkMode() {
  document.documentElement.classList.toggle('my-app-dark')
}

async function handleLogout() {
  logout()
  await router.replace({ name: 'login' })
}
</script>

<template>
  <main class="home-layout">
    <aside class="home-sidebar">
      <div class="sidebar-header">
        <h1>风纪面板</h1>
        <p class="permission-text">{{ permissionLabel }}</p>
        <Button type="button" class="theme-toggle" severity="secondary" variant="outlined" label="切换暗色"
          @click="toggleDarkMode" />
        <Button type="button" class="logout-btn" severity="danger" variant="outlined" label="退出登录"
          @click="handleLogout" />
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
  background: var(--surface-ground);
}

.home-sidebar {
  border-right: 1px solid var(--surface-border);
  background: var(--surface-card);
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
  color: var(--text-color);
}


.permission-text {
  margin: 0;
  color: var(--text-color-secondary);
  font-size: 0.85rem;
}


.theme-toggle {
  border: 1px solid var(--surface-border);
  border-radius: 0.5rem;
  background: var(--surface-0);
  padding: 0.5rem 0.75rem;
  cursor: pointer;
}

.logout-btn {
  border-radius: 0.5rem;
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

<script setup lang="ts">
import Button from 'primevue/button'
import Password from 'primevue/password'
import Toolbar from 'primevue/toolbar'
import { useToast } from 'primevue/usetoast'
import { onMounted, ref, watch } from 'vue'
import { apiFetch } from '@/services/auth'

type ManagerItem = {
    name: string
    password: string
}

const loading = ref(false)
const error = ref('')
const managerList = ref<ManagerItem[]>([])
const toast = useToast()

watch(error, (value) => {
    if (!value) {
        return
    }

    toast.add({ severity: 'error', summary: '请求失败', detail: value, life: 3000 })
})

async function loadManagerList() {
    loading.value = true
    error.value = ''

    try {
        const response = await apiFetch('/api/manager', { method: 'GET' })

        if (!response.ok) {
            throw new Error(`请求失败: ${response.status}`)
        }

        const data = (await response.json()) as ManagerItem[]
        managerList.value = Array.isArray(data) ? data : []
    } catch (err) {
        error.value = err instanceof Error ? err.message : '获取管理员列表失败'
        managerList.value = []
    } finally {
        loading.value = false
    }
}

onMounted(() => {
    loadManagerList()
})
</script>

<template>
    <section class="panel">
        <h2>管理员管理</h2>

        <Toolbar class="toolbar">
            <template #start>
                <Button type="button" class="refresh-btn" severity="secondary" variant="outlined" :disabled="loading"
                    @click="loadManagerList" :label="loading ? '加载中...' : '刷新列表'" />
            </template>
        </Toolbar>

        <ul v-if="managerList.length > 0" class="list">
            <li v-for="(item, index) in managerList" :key="`${item.name}-${index}`" class="item">
                <div class="meta">
                    <span class="label">用户名</span>
                    <strong>{{ item.name }}</strong>
                </div>
                <div class="password-wrap">
                    <span class="label">密码</span>
                    <Password v-model="item.password" toggleMask :feedback="false" inputClass="password-input" />
                </div>
            </li>
        </ul>

        <p v-else class="empty">暂无管理员数据</p>
    </section>
</template>

<style scoped>
.panel {
    background: #fff;
    border: 1px solid #dbe3f1;
    border-radius: 0.9rem;
    padding: 1rem 1.25rem;
}

h2 {
    margin: 0 0 0.5rem;
}

p {
    margin: 0 0 0.9rem;
    color: #475569;
}

.toolbar {
    margin-bottom: 0.9rem;
}

.toolbar :deep(.p-toolbar-start) {
    width: 100%;
}

.refresh-btn {
    border-radius: 0.5rem;
}

.list {
    margin: 0;
    padding: 0;
    list-style: none;
}

.item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
    border: 1px solid #e2e8f0;
    border-radius: 0.6rem;
    padding: 0.7rem 0.75rem;
}

.meta,
.password-wrap {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    flex: 1 1 50%;
    min-width: 0;
}

.label {
    font-size: 0.8rem;
    color: #64748b;
}

.error {
    color: #b91c1c;
}

.empty {
    color: #64748b;
}

:deep(.password-input) {
    width: 100%;
}

@media (max-width: 700px) {
    .item {
        flex-direction: column;
        align-items: stretch;
    }

    .password-wrap {
        min-width: 0;
    }
}
</style>

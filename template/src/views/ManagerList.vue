<script setup lang="ts">
import Button from 'primevue/button'
import ConfirmDialog from 'primevue/confirmdialog'
import ContextMenu from 'primevue/contextmenu'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import type { MenuItem } from 'primevue/menuitem'
import Paginator, { type PageState } from 'primevue/paginator'
import Password from 'primevue/password'
import Toolbar from 'primevue/toolbar'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { computed, onMounted, ref, watch } from 'vue'
import { apiFetch } from '@/services/auth'

type ManagerItem = {
    name: string
    password: string
}

const loading = ref(false)
const error = ref('')
const managerList = ref<ManagerItem[]>([])
const addDialogVisible = ref(false)
const addName = ref('')
const addLoading = ref(false)
const deletingName = ref('')
const first = ref(0)
const rows = ref(10)
const selectedManager = ref<ManagerItem | null>(null)
const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)
const confirm = useConfirm()
const toast = useToast()

const pagedManagerList = computed(() => {
    return managerList.value.slice(first.value, first.value + rows.value)
})

const contextMenuItems = computed(() => {
    const items: MenuItem[] = [
        {
            label: '删除管理员',
            icon: 'pi pi-trash',
            disabled: loading.value || !selectedManager.value || deletingName.value.length > 0,
            command: () => {
                if (!selectedManager.value) {
                    return
                }

                openDeleteConfirm(selectedManager.value.name)
            },
        },
    ]

    return items
})

watch(error, (value) => {
    if (!value) {
        return
    }

    toast.add({ severity: 'error', summary: '请求失败', detail: value, life: 3000 })
})

watch([managerList, rows], () => {
    if (first.value >= managerList.value.length) {
        first.value = Math.max(0, Math.floor((managerList.value.length - 1) / rows.value) * rows.value)
    }
})

function handlePage(event: PageState) {
    first.value = event.first
    rows.value = event.rows
}

function showItemContextMenu(event: MouseEvent, item: ManagerItem) {
    selectedManager.value = item
    contextMenuRef.value?.show(event)
}

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

function openAddDialog() {
    addName.value = ''
    addDialogVisible.value = true
}

function closeAddDialog() {
    addDialogVisible.value = false
}

async function createManager() {
    const name = addName.value.trim()

    if (!name) {
        toast.add({ severity: 'warn', summary: '提示', detail: '请输入管理员用户名', life: 2500 })
        return
    }

    addLoading.value = true

    try {
        const response = await apiFetch(`/api/manager/${encodeURIComponent(name)}`, {
            method: 'POST',
        })

        if (!response.ok) {
            throw new Error(`添加管理员失败: ${response.status}`)
        }

        const created = (await response.json()) as Partial<ManagerItem>
        await loadManagerList()
        addDialogVisible.value = false

        if (created.password) {
            toast.add({
                severity: 'success',
                summary: '添加成功',
                detail: `账号 ${name} 已创建，初始密码：${created.password}`,
                life: 5000,
            })
        } else {
            toast.add({ severity: 'success', summary: '添加成功', detail: `账号 ${name} 已创建`, life: 3000 })
        }
    } catch (err) {
        toast.add({
            severity: 'error',
            summary: '添加失败',
            detail: err instanceof Error ? err.message : '添加管理员失败',
            life: 3000,
        })
    } finally {
        addLoading.value = false
    }
}

function openDeleteConfirm(name: string) {
    confirm.require({
        header: '确认删除管理员',
        message: `确定删除管理员账号「${name}」吗？`,
        icon: 'pi pi-exclamation-triangle',
        rejectLabel: '取消',
        acceptLabel: '确认删除',
        acceptClass: 'p-button-danger',
        accept: () => {
            void deleteManager(name)
        },
    })
}

async function deleteManager(name: string) {
    deletingName.value = name

    try {
        const response = await apiFetch(`/api/manager/${encodeURIComponent(name)}`, {
            method: 'DELETE',
        })

        if (!response.ok) {
            throw new Error(`删除管理员失败: ${response.status}`)
        }

        await loadManagerList()
        toast.add({ severity: 'success', summary: '删除成功', detail: `账号 ${name} 已删除`, life: 3000 })
    } catch (err) {
        toast.add({
            severity: 'error',
            summary: '删除失败',
            detail: err instanceof Error ? err.message : '删除管理员失败',
            life: 3000,
        })
    } finally {
        deletingName.value = ''
    }
}

onMounted(() => {
    loadManagerList()
})
</script>

<template>
    <section class="panel">
        <ConfirmDialog />
        <ContextMenu ref="contextMenuRef" :model="contextMenuItems" />
        <h2>管理员管理</h2>

        <Toolbar class="toolbar">
            <template #start>
                <div class="toolbar-actions">
                    <Button type="button" class="refresh-btn" severity="secondary" variant="outlined"
                        :disabled="loading" @click="loadManagerList" :label="loading ? '加载中...' : '刷新列表'" />
                    <Button type="button" class="add-btn" :disabled="loading" @click="openAddDialog" label="添加管理员" />
                </div>
            </template>
        </Toolbar>

        <Dialog v-model:visible="addDialogVisible" modal header="添加管理员" :style="{ width: '26rem' }">
            <div class="dialog-body">
                <label class="label" for="add-manager-name">用户名</label>
                <InputText id="add-manager-name" v-model="addName" class="name-input" :disabled="addLoading"
                    placeholder="请输入管理员用户名" @keyup.enter="createManager" />
            </div>

            <template #footer>
                <Button type="button" severity="secondary" variant="text" :disabled="addLoading" label="取消"
                    @click="closeAddDialog" />
                <Button type="button" :loading="addLoading" :disabled="addLoading" label="确认添加"
                    @click="createManager" />
            </template>
        </Dialog>

        <ul v-if="managerList.length > 0" class="list">
            <li v-for="(item, index) in pagedManagerList" :key="`${item.name}-${first + index}`" class="item"
                @contextmenu.prevent="showItemContextMenu($event, item)">
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

        <Paginator v-if="managerList.length > 0" class="paginator" :first="first" :rows="rows"
            :totalRecords="managerList.length" :rowsPerPageOptions="[5, 10, 20, 50]" @page="handlePage" />

        <p v-else class="empty">暂无管理员数据</p>
    </section>
</template>

<style scoped>
.panel {
    background: var(--surface-card);
    border: 1px solid var(--surface-border);
    border-radius: 0.9rem;
    padding: 1rem 1.25rem;
}

h2 {
    margin: 0 0 0.5rem;
}

p {
    margin: 0 0 0.9rem;
    color: var(--text-color-secondary);
}

.toolbar {
    margin-bottom: 0.9rem;
}

.toolbar :deep(.p-toolbar-start) {
    width: 100%;
}

.toolbar-actions {
    width: 100%;
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
}

.refresh-btn {
    border-radius: 0.5rem;
}

.add-btn {
    border-radius: 0.5rem;
}

.dialog-body {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
}

.name-input {
    width: 100%;
}

.list {
    margin: 0;
    padding: 0;
    list-style: none;
}

.paginator {
    margin-top: 0.7rem;
    border: 1px solid var(--surface-border);
    border-radius: 0.6rem;
}

.item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
    border: 1px solid var(--surface-border);
    border-radius: 0.6rem;
    padding: 0.7rem 0.75rem;
    cursor: context-menu;
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
    color: var(--text-color-secondary);
}


.error {
    color: var(--red-600, #b91c1c);
}


.empty {
    color: var(--text-color-secondary);
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

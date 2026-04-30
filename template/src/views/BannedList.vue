<script setup lang="ts">
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import ConfirmDialog from 'primevue/confirmdialog'
import ContextMenu from 'primevue/contextmenu'
import FloatLabel from 'primevue/floatlabel'
import InputNumber from 'primevue/inputnumber'
import type { MenuItem } from 'primevue/menuitem'
import Paginator, { type PageState } from 'primevue/paginator'
import Toolbar from 'primevue/toolbar'
import ToggleSwitch from 'primevue/toggleswitch'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { computed, onMounted, ref, watch } from 'vue'
import { apiFetch } from '@/services/auth'

type BannedItem = {
    id: number
    time: number
    duration: number
}

type BannedViewItem = {
    id: number
    time: number
    duration: number
    nickname: string
    avatar: string
}

type BanResult = {
    status: string
    id: number
    time: number
    duration: number
}

type QqUserInfo = {
    qq?: string
    nickname?: string
    avatar_url?: string
    avatar?: string
    headurl?: string
    data?: QqUserInfo
    result?: QqUserInfo
}

type QqProfile = {
    nickname: string
    avatar: string
}

type CachedQqProfile = QqProfile & {
    expiresAt: number
}

const PROFILE_CACHE_TTL_MS = 10 * 60 * 1000
const PROFILE_ERROR_CACHE_TTL_MS = 60 * 1000
const qqProfileCache = new Map<number, CachedQqProfile>()
const qqProfileInFlight = new Map<number, Promise<QqProfile>>()

const loading = ref(false)
const error = ref('')
const actionError = ref('')
const actionSuccess = ref('')
const banning = ref(false)
const unbanning = ref(false)
const banQq = ref<number | null>(null)
const banDuration = ref<number | null>(0)
const multiSelectEnabled = ref(false)
const selectedIds = ref<number[]>([])
const bannedList = ref<BannedViewItem[]>([])
const selectedItem = ref<BannedViewItem | null>(null)
const first = ref(0)
const rows = ref(10)
const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)
const confirm = useConfirm()
const toast = useToast()

const pagedBannedList = computed(() => {
    return bannedList.value.slice(first.value, first.value + rows.value)
})

const selectedItems = computed(() => {
    const selectedSet = new Set(selectedIds.value)
    return bannedList.value.filter((item) => selectedSet.has(item.id))
})

const hasSelectedData = computed(() => selectedItems.value.length > 0)

const contextMenuItems = ref<MenuItem[]>([
    {
        label: '解除封禁',
        icon: 'pi pi-user-minus',
        command: () => {
            if (!selectedItem.value) {
                return
            }
            openUnbanConfirm(selectedItem.value)
        },
    },
])

watch(multiSelectEnabled, (enabled) => {
    if (!enabled) {
        selectedIds.value = []
    }
})

watch(actionError, (value) => {
    if (!value) {
        return
    }

    toast.add({ severity: 'error', summary: '操作失败', detail: value, life: 3000 })
})

watch(actionSuccess, (value) => {
    if (!value) {
        return
    }

    toast.add({ severity: 'success', summary: '操作成功', detail: value, life: 2500 })
})

watch(error, (value) => {
    if (!value) {
        return
    }

    toast.add({ severity: 'error', summary: '请求失败', detail: value, life: 3000 })
})

watch([bannedList, rows], () => {
    if (first.value >= bannedList.value.length) {
        first.value = Math.max(0, Math.floor((bannedList.value.length - 1) / rows.value) * rows.value)
    }
})

function handlePage(event: PageState) {
    first.value = event.first
    rows.value = event.rows
}

function formatUnixTime(unixSeconds: number) {
    return new Date(unixSeconds * 1000).toLocaleString('zh-CN', {
        hour12: false,
    })
}

function formatDuration(seconds: number) {
    if (seconds === 0) {
        return '永久封禁'
    }

    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const remainSeconds = seconds % 60
    const parts: string[] = []

    if (days > 0) {
        parts.push(`${days}天`)
    }
    if (hours > 0) {
        parts.push(`${hours}小时`)
    }
    if (minutes > 0) {
        parts.push(`${minutes}分钟`)
    }
    if (remainSeconds > 0 || parts.length === 0) {
        parts.push(`${remainSeconds}秒`)
    }

    return parts.join('')
}

function formatExpireText(item: BannedViewItem) {
    if (item.duration === 0) {
        return '到期时间: 永不'
    }

    return `到期时间: ${formatUnixTime(item.time + item.duration)}`
}

function getAvatarUrl(qq: number) {
    return `http://q.qlogo.cn/headimg_dl?dst_uin=${qq}&spec=640&img_type=jpg`
}

function resolveNickname(value: unknown, qq: number) {
    if (typeof value !== 'string') {
        return `QQ ${qq}`
    }

    return value.trim().length > 0 ? value : `QQ ${qq}`
}

function resolveAvatar(value: unknown, qq: number) {
    if (typeof value !== 'string') {
        return getAvatarUrl(qq)
    }

    return value.trim().length > 0 ? value : getAvatarUrl(qq)
}

function asRecord(value: unknown) {
    if (typeof value === 'object' && value !== null) {
        return value as Record<string, unknown>
    }

    return null
}

function parseQqProfilePayload(payload: unknown, qq: number): QqProfile | null {
    const root = asRecord(payload)
    if (!root) {
        return null
    }

    const nested = asRecord(root.data) ?? asRecord(root.result)
    const source = nested ?? root

    const nickname = resolveNickname(source.nickname, qq)
    const avatar = resolveAvatar(source.avatar_url ?? source.avatar ?? source.headurl, qq)

    if (nickname === `QQ ${qq}` && avatar === getAvatarUrl(qq)) {
        return null
    }

    return {
        nickname,
        avatar,
    }
}

function getDefaultProfile(qq: number): QqProfile {
    return {
        nickname: `QQ ${qq}`,
        avatar: getAvatarUrl(qq),
    }
}

function getCachedProfile(qq: number) {
    const cached = qqProfileCache.get(qq)
    if (!cached) {
        return null
    }

    if (cached.expiresAt <= Date.now()) {
        qqProfileCache.delete(qq)
        return null
    }

    return {
        nickname: cached.nickname,
        avatar: cached.avatar,
    }
}

function setCachedProfile(qq: number, profile: QqProfile, ttlMs: number) {
    qqProfileCache.set(qq, {
        ...profile,
        expiresAt: Date.now() + ttlMs,
    })
}

async function fetchQqUserInfoFromApi(qq: number) {
    try {
        const response = await apiFetch(`/api/qq/userinfo/${qq}`, {
            method: 'GET',
            headers: {
                Accept: 'application/json, text/plain, */*',
            },
        })

        if (!response.ok) {
            throw new Error(`获取QQ资料失败: ${response.status}`)
        }

        const payload = (await response.json()) as unknown
        const profile = parseQqProfilePayload(payload, qq)
        if (!profile) {
            throw new Error('QQ资料返回结构不匹配')
        }

        return profile
    } catch (err) {
        console.warn('获取QQ昵称失败，使用默认资料', qq, err)
        return null
    }
}

async function fetchQqUserInfo(qq: number) {
    const cached = getCachedProfile(qq)
    if (cached) {
        return cached
    }

    const inFlight = qqProfileInFlight.get(qq)
    if (inFlight) {
        return inFlight
    }

    // 同一QQ并发请求时复用同一个 Promise，避免短时间重复打接口。
    const task = (async () => {
        const fromApi = await fetchQqUserInfoFromApi(qq)
        if (fromApi) {
            setCachedProfile(qq, fromApi, PROFILE_CACHE_TTL_MS)
            return fromApi
        }

        const fallback = getDefaultProfile(qq)
        setCachedProfile(qq, fallback, PROFILE_ERROR_CACHE_TTL_MS)
        return fallback
    })()

    qqProfileInFlight.set(qq, task)

    try {
        return await task
    } finally {
        qqProfileInFlight.delete(qq)
    }
}

function isBanResult(value: unknown): value is BanResult {
    if (typeof value !== 'object' || value === null) {
        return false
    }

    const data = value as Record<string, unknown>
    return typeof data.status === 'string' && typeof data.id === 'number' && typeof data.time === 'number' && typeof data.duration === 'number'
}

function upsertBannedItem(item: BannedViewItem) {
    const next = bannedList.value.filter((existing) => existing.id !== item.id)
    next.unshift(item)
    bannedList.value = next
}

function removeBannedItem(id: number) {
    bannedList.value = bannedList.value.filter((item) => item.id !== id)
    selectedIds.value = selectedIds.value.filter((selectedId) => selectedId !== id)
}

function showItemContextMenu(event: MouseEvent, item: BannedViewItem) {
    selectedItem.value = item
    contextMenuRef.value?.show(event)
}

function openUnbanConfirm(item: BannedViewItem) {
    confirm.require({
        header: '确认解除封禁',
        message: `确认解除 ${item.nickname}（QQ: ${item.id}）的封禁吗？`,
        icon: 'pi pi-exclamation-triangle',
        rejectLabel: '取消',
        acceptLabel: '确认解除',
        acceptClass: 'p-button-danger',
        accept: () => {
            void unbanById(item.id, item.nickname)
        },
    })
}

function openBatchUnbanConfirm() {
    if (!hasSelectedData.value) {
        return
    }

    confirm.require({
        header: '确认批量解封',
        message: `确认批量解除 ${selectedItems.value.length} 个账号的封禁吗？`,
        icon: 'pi pi-exclamation-triangle',
        rejectLabel: '取消',
        acceptLabel: '确认解除',
        acceptClass: 'p-button-danger',
        accept: () => {
            void unbanSelectedItems()
        },
    })
}

function openBanConfirmDialog() {
    actionError.value = ''
    actionSuccess.value = ''

    confirm.require({
        group: 'ban-user',
        header: '添加封禁',
        message: '设置封禁对象与封禁时长（秒）',
    })
}

async function unbanById(id: number, nickname: string) {
    actionError.value = ''
    actionSuccess.value = ''
    unbanning.value = true

    try {
        const response = await apiFetch(`/api/${id}`, { method: 'DELETE' })
        if (!response.ok) {
            throw new Error(`解除封禁失败: ${response.status}`)
        }

        removeBannedItem(id)
        actionSuccess.value = `已解除封禁：${nickname}（${id}）`
    } catch (err) {
        actionError.value = err instanceof Error ? err.message : '解除封禁失败'
    } finally {
        unbanning.value = false
    }
}

async function unbanSelectedItems() {
    if (!hasSelectedData.value) {
        return
    }

    actionError.value = ''
    actionSuccess.value = ''
    unbanning.value = true

    const targets = [...selectedItems.value]

    try {
        const results = await Promise.allSettled(
            targets.map(async (item) => {
                const response = await apiFetch(`/api/${item.id}`, { method: 'DELETE' })
                if (!response.ok) {
                    throw new Error(`${item.nickname}（${item.id}）: ${response.status}`)
                }

                return item
            }),
        )

        const successItems: BannedViewItem[] = []
        const failMessages: string[] = []

        results.forEach((result) => {
            if (result.status === 'fulfilled') {
                successItems.push(result.value)
                return
            }

            const reason = result.reason
            failMessages.push(reason instanceof Error ? reason.message : '未知错误')
        })

        successItems.forEach((item) => removeBannedItem(item.id))

        if (successItems.length > 0) {
            actionSuccess.value = `批量解封成功：${successItems.length} 个账号`
        }

        if (failMessages.length > 0) {
            const preview = failMessages.slice(0, 3).join('；')
            const remains = failMessages.length > 3 ? `，另有 ${failMessages.length - 3} 个失败` : ''
            actionError.value = `部分解封失败：${preview}${remains}`
        }
    } finally {
        unbanning.value = false
    }
}

async function loadBannedList() {
    loading.value = true
    error.value = ''

    try {
        const response = await apiFetch('/api/list', { method: 'GET' })

        if (!response.ok) {
            throw new Error(`请求失败: ${response.status}`)
        }

        const data = (await response.json()) as BannedItem[]
        const list = Array.isArray(data) ? data : []
        const displayList = await Promise.all(
            list.map(async (item) => {
                const profile = await fetchQqUserInfo(item.id)
                return {
                    id: item.id,
                    time: item.time,
                    duration: item.duration,
                    nickname: profile.nickname,
                    avatar: profile.avatar,
                }
            }),
        )

        bannedList.value = displayList
    } catch (err) {
        error.value = err instanceof Error ? err.message : '获取封禁列表失败'
        bannedList.value = []
    } finally {
        loading.value = false
    }
}

async function banByQq() {
    const qq = banQq.value

    actionError.value = ''
    actionSuccess.value = ''

    if (qq === null || !Number.isInteger(qq) || qq <= 0) {
        actionError.value = '请输入正确的QQ号'
        return false
    }

    if (banDuration.value === null || !Number.isInteger(banDuration.value) || banDuration.value < 0) {
        actionError.value = '封禁时长必须为大于等于 0 的整数（秒）'
        return false
    }

    const qqText = String(qq)
    const duration = banDuration.value

    banning.value = true

    try {
        const response = await apiFetch(`/api/${qqText}?duration=${duration}`, { method: 'POST' })

        if (!response.ok) {
            throw new Error(`封禁失败: ${response.status}`)
        }

        const payload = (await response.json()) as unknown
        if (!isBanResult(payload)) {
            throw new Error('封禁失败: 返回数据格式不正确')
        }

        if (payload.status !== 'Banned') {
            throw new Error(`封禁失败: status=${payload.status}`)
        }

        const profile = await fetchQqUserInfo(payload.id)
        upsertBannedItem({
            id: payload.id,
            time: payload.time,
            duration: payload.duration,
            nickname: profile.nickname,
            avatar: profile.avatar,
        })

        actionSuccess.value = `封禁成功：${payload.id}（${formatDuration(payload.duration)}）`
        banQq.value = null
        return true
    } catch (err) {
        actionError.value = err instanceof Error ? err.message : '封禁失败'
        return false
    } finally {
        banning.value = false
    }
}

async function submitBanAndClose(acceptCallback: () => void) {
    const ok = await banByQq()
    if (!ok) {
        return
    }

    acceptCallback()
}

onMounted(() => {
    loadBannedList()
})
</script>

<template>
    <section class="panel">
        <ConfirmDialog />
        <ConfirmDialog group="ban-user">
            <template #container="{ message, acceptCallback, rejectCallback }">
                <form class="ban-dialog" @submit.prevent="submitBanAndClose(acceptCallback)">
                    <div class="ban-dialog-head">
                        <h3>{{ message.header }}</h3>
                        <p>{{ message.message }}</p>
                    </div>

                    <FloatLabel variant="on">
                        <InputNumber v-model="banQq" input-id="ban-qq-dialog-input" class="ban-input"
                            input-class="ban-input-inner" :use-grouping="false" :min="0" :disabled="banning || unbanning" />
                        <label for="ban-qq-dialog-input">QQ号</label>
                    </FloatLabel>

                    <FloatLabel variant="on">
                        <InputNumber v-model="banDuration" input-id="ban-duration-dialog-input" class="ban-input"
                            input-class="ban-input-inner" :use-grouping="false" :min="0" :disabled="banning || unbanning" />
                        <label for="ban-duration-dialog-input">封禁时长(秒，0=永久)</label>
                    </FloatLabel>

                    <div class="ban-dialog-actions">
                        <Button type="button" label="取消" severity="secondary" variant="outlined"
                            :disabled="banning" @click="rejectCallback" />
                        <Button type="submit" severity="danger" :disabled="banning || unbanning"
                            :label="banning ? '封禁中...' : '确认封禁'" />
                    </div>
                </form>
            </template>
        </ConfirmDialog>
        <ContextMenu ref="contextMenuRef" :model="contextMenuItems" />

        <h2>封禁管理</h2>

        <Toolbar class="action-toolbar">
            <template #start>
                <Button type="button" class="refresh-btn" severity="secondary" variant="outlined" :disabled="loading"
                    @click="loadBannedList" :label="loading ? '加载中...' : '刷新列表'" />

                <label class="multi-toggle">
                    <span>多选模式</span>
                    <ToggleSwitch v-model="multiSelectEnabled" input-id="multi-select-switch" />
                </label>

                <Button v-if="hasSelectedData" type="button" class="batch-unban-btn" severity="danger"
                    :disabled="unbanning || banning" @click="openBatchUnbanConfirm"
                    :label="unbanning ? '解封中...' : `批量解封（${selectedItems.length}）`" />
            </template>

            <template #end>
                <Button type="button" class="ban-btn" severity="danger" :disabled="banning || unbanning || loading"
                    @click="openBanConfirmDialog" label="添加封禁" />
            </template>
        </Toolbar>

        <ul v-if="bannedList.length > 0" class="list">
            <li v-for="item in pagedBannedList" :key="item.id" class="item"
                :class="{ 'item-multi': multiSelectEnabled }" @contextmenu.prevent="showItemContextMenu($event, item)">
                <Checkbox v-if="multiSelectEnabled" v-model="selectedIds" :value="item.id" class="item-checkbox"
                    aria-label="选择封禁项" />
                <img class="avatar" :src="item.avatar" :alt="`${item.nickname} 头像`" />
                <div class="info-block">
                    <strong class="nickname">{{ item.nickname }}</strong>
                    <span class="qq">QQ: {{ item.id }}</span>
                    <span class="qq">封禁时长: {{ formatDuration(item.duration) }}</span>
                </div>
                <span class="time">封禁时间: {{ formatUnixTime(item.time) }}</span>
                <span class="time">{{ formatExpireText(item) }}</span>
            </li>
        </ul>

        <Paginator v-if="bannedList.length > 0" class="paginator" :first="first" :rows="rows"
            :totalRecords="bannedList.length" :rowsPerPageOptions="[5, 10, 20, 50]" @page="handlePage" />

        <p v-else class="empty">暂无封禁数据</p>
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


.action-toolbar {
    margin-bottom: 0.9rem;
    border-radius: 0.75rem;
    border: 1px solid var(--surface-border);
    background: var(--surface-0);
    padding: 0.55rem;
}

.action-toolbar :deep(.p-toolbar-start) {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
}

.action-toolbar :deep(.p-toolbar-end) {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
    justify-content: flex-end;
}

.multi-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-color-secondary);
    font-size: 0.9rem;
}

.refresh-btn,
.ban-btn,
.batch-unban-btn {
    border-radius: 0.5rem;
}

.ban-dialog {
    background: var(--surface-card);
    border: 1px solid var(--surface-border);
    border-radius: 0.85rem;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: min(520px, 92vw);
}

.ban-dialog-head h3 {
    margin: 0;
    color: var(--text-color);
}

.ban-dialog-head p {
    margin: 0.35rem 0 0;
    color: var(--text-color-secondary);
}

.ban-dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
}

.ban-input {
    min-width: 220px;
}

:deep(.ban-input .p-inputtext) {
    border: 1px solid var(--surface-border);
    border-radius: 0.5rem;
    background: var(--surface-0);
    color: var(--text-color);
    padding: 0.45rem 0.7rem;
    width: 100%;
}

:deep(.ban-input .p-inputtext:disabled) {
    cursor: not-allowed;
    opacity: 0.7;
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
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
    border: 1px solid var(--surface-border);
    border-radius: 0.6rem;
    padding: 0.55rem 0.75rem;
    cursor: context-menu;
}

.item-checkbox {
    flex-shrink: 0;
}

.avatar {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    border: 1px solid var(--surface-border);
    object-fit: cover;
}

.info-block {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
    flex: 1;
}


.nickname {
    color: var(--text-color);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}


.qq {
    color: var(--text-color-secondary);
}


.time {
    color: var(--text-color-secondary);
    flex-shrink: 0;
}


.error {
    color: var(--red-600, #b91c1c);
}


.success {
    color: var(--green-600, #15803d);
}


.empty {
    color: var(--text-color-secondary);
}

@media (max-width: 760px) {

    .action-toolbar :deep(.p-toolbar-start),
    .action-toolbar :deep(.p-toolbar-end) {
        align-items: stretch;
        width: 100%;
        justify-content: flex-start;
    }

    .ban-input {
        width: 100%;
        min-width: 0;
    }

    .item {
        align-items: flex-start;
        flex-wrap: wrap;
    }

    .time {
        width: 100%;
        padding-left: calc(44px + 1rem);
    }

    .item.item-multi .time {
        padding-left: calc(44px + 1.9rem);
    }
}
</style>

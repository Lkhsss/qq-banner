<script setup lang="ts">
import Button from 'primevue/button'
import { computed, onMounted, ref } from 'vue'
import { apiFetch } from '@/services/auth'

type MetricKey = 'success' | 'fail' | 'banned' | 'request'

type MetricCard = {
    key: MetricKey
    title: string
    hint: string
    value: number
}

const loading = ref(false)
const error = ref('')

const metrics = ref<Record<MetricKey, number>>({
    success: 0,
    fail: 0,
    banned: 0,
    request: 0,
})

const cards = computed<MetricCard[]>(() => {
    return [
        {
            key: 'success',
            title: '成功调用 API 次数',
            hint: '/api/metrics/success',
            value: metrics.value.success,
        },
        {
            key: 'fail',
            title: '调用失败次数',
            hint: '/api/metrics/fail',
            value: metrics.value.fail,
        },
        {
            key: 'banned',
            title: '被封禁人数',
            hint: '/api/metrics/banned',
            value: metrics.value.banned,
        },
        {
            key: 'request',
            title: '总请求数（含未命中路径）',
            hint: '/api/metrics/request',
            value: metrics.value.request,
        },
    ]
})

function parseMetricValue(payload: unknown, endpoint: string) {
    if (typeof payload === 'number' && Number.isFinite(payload)) {
        return payload
    }

    if (typeof payload === 'string') {
        const value = Number(payload.trim())

        if (Number.isFinite(value)) {
            return value
        }
    }

    throw new Error(`接口 ${endpoint} 返回值不是数字`)
}

async function fetchMetric(endpoint: string) {
    const response = await apiFetch(endpoint, {
        method: 'GET',
        headers: {
            Accept: 'text/plain, application/json, */*',
        },
    })

    if (!response.ok) {
        throw new Error(`请求失败: ${response.status}`)
    }

    const payload = await response.json()
    return parseMetricValue(payload, endpoint)
}

async function loadMetrics() {
    loading.value = true
    error.value = ''

    try {
        const [success, fail, banned, request] = await Promise.all([
            fetchMetric('/api/metrics/success'),
            fetchMetric('/api/metrics/fail'),
            fetchMetric('/api/metrics/banned'),
            fetchMetric('/api/metrics/request'),
        ])

        metrics.value = {
            success,
            fail,
            banned,
            request,
        }
    } catch (err) {
        error.value = err instanceof Error ? err.message : '获取指标失败'
    } finally {
        loading.value = false
    }
}

onMounted(() => {
    void loadMetrics()
})
</script>

<template>
    <section class="panel">
        <div class="header-row">
            <div>
                <h2>数据展示面板</h2>
                <p>后端请求统计实时快照</p>
            </div>
            <Button type="button" severity="secondary" variant="outlined" :disabled="loading"
                :label="loading ? '刷新中...' : '刷新数据'" @click="loadMetrics" />
        </div>

        <p v-if="error" class="error">{{ error }}</p>

        <div class="metrics-grid">
            <article v-for="card in cards" :key="card.key" class="metric-card">
                <span class="metric-title">{{ card.title }}</span>
                <strong class="metric-value">{{ card.value.toLocaleString('zh-CN') }}</strong>
                <small class="metric-hint">{{ card.hint }}</small>
            </article>
        </div>
    </section>
</template>

<style scoped>
.panel {
    background: var(--surface-card);
    border: 1px solid var(--surface-border);
    border-radius: 0.9rem;
    padding: 1rem 1.25rem;
}

.header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.9rem;
}

h2 {
    margin: 0 0 0.4rem;
}

p {
    margin: 0;
    color: var(--text-color-secondary);
}

.error {
    color: var(--red-600, #b91c1c);
    margin: 0 0 0.9rem;
}

.metrics-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.75rem;
}

.metric-card {
    border: 1px solid var(--surface-border);
    border-radius: 0.75rem;
    padding: 0.95rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    background: var(--surface-0);
}

.metric-title {
    color: var(--text-color-secondary);
    font-size: 0.86rem;
}

.metric-value {
    font-size: 1.5rem;
    line-height: 1.2;
}

.metric-hint {
    color: var(--text-color-secondary);
}

@media (max-width: 760px) {
    .header-row {
        flex-direction: column;
        align-items: stretch;
    }

    .metrics-grid {
        grid-template-columns: 1fr;
    }
}
</style>

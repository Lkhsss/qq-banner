import { ref } from 'vue'
import axios, { type AxiosRequestConfig, type Method } from 'axios'

export const isAuthenticated = ref(false)
export const authChecking = ref(false)

const apiClient = axios.create({
  withCredentials: true,
  headers: {
    Accept: 'application/json, text/plain, */*',
  },
  validateStatus: () => true,
})

apiClient.interceptors.response.use((response) => {
  if (response.status === 401) {
    isAuthenticated.value = false
  }

  return response
})

type ApiResponse<T = unknown> = {
  ok: boolean
  status: number
  json: () => Promise<T>
}

function toUrl(input: RequestInfo | URL) {
  if (typeof input === 'string') {
    return input
  }

  if (input instanceof URL) {
    return input.toString()
  }

  return input.url
}

export async function login(name: string, password: string) {
  const formData = new URLSearchParams()
  formData.set('name', name)
  formData.set('password', password)

  const response = await apiClient.post('/api/auth', formData.toString(), {
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8',
    },
  })

  if (response.status < 200 || response.status >= 300) {
    throw new Error(`登录失败: ${response.status}`)
  }

  isAuthenticated.value = true
}

export async function checkAuthByProbe(path = '/api/list') {
  authChecking.value = true

  try {
    const response = await apiClient.get(path)
    const ok = response.status >= 200 && response.status < 300

    isAuthenticated.value = ok
    return ok
  } catch {
    isAuthenticated.value = false
    return false
  } finally {
    authChecking.value = false
  }
}

export async function apiFetch(input: RequestInfo | URL, init?: RequestInit) {
  const config: AxiosRequestConfig = {
    url: toUrl(input),
    method: (init?.method ?? 'GET') as Method,
    headers: init?.headers as AxiosRequestConfig['headers'],
    data: init?.body,
  }

  const response = await apiClient.request(config)

  if (response.status === 401) {
    throw new Error('未登录或登录已过期，请重新登录')
  }

  return {
    ok: response.status >= 200 && response.status < 300,
    status: response.status,
    json: async () => response.data,
  } as ApiResponse
}

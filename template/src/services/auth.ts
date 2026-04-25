import { ref } from 'vue'
import axios, { type AxiosRequestConfig, type Method } from 'axios'

export const isAuthenticated = ref(false)
export const authChecking = ref(false)
export const forceLoggedOut = ref(false)

export enum Permission {
  SuperAdmin = 0,
  Admin = 1,
  User = 2,
}

export const currentPermission = ref<Permission | null>(null)

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

function parsePermissionValue(value: string): Permission | null {
  const normalized = value.trim()

  if (normalized === '0') {
    return Permission.SuperAdmin
  }

  if (normalized === '1') {
    return Permission.Admin
  }

  if (normalized === '2') {
    return Permission.User
  }

  return null
}

function readCookie(name: string) {
  const all = document.cookie ? document.cookie.split('; ') : []

  for (const row of all) {
    const [key, ...rest] = row.split('=')
    if (key !== name) {
      continue
    }

    return decodeURIComponent(rest.join('='))
  }

  return null
}

export function getPermissionFromCookie() {
  const raw = readCookie('permisson')
  if (!raw) {
    return null
  }

  return parsePermissionValue(raw)
}

function syncPermissionFromCookie() {
  currentPermission.value = getPermissionFromCookie()
}

export function hasPermission(required: Permission) {
  if (currentPermission.value === null) {
    return false
  }

  return currentPermission.value <= required
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

  forceLoggedOut.value = false
  isAuthenticated.value = true
  syncPermissionFromCookie()
}

export async function checkAuthByProbe(path = '/api/auth') {
  if (forceLoggedOut.value) {
    isAuthenticated.value = false
    currentPermission.value = null
    return false
  }

  authChecking.value = true

  try {
    const response = await apiClient.get(path)
    const ok = response.status >= 200 && response.status < 300 && response.data?.ok === true

    isAuthenticated.value = ok
    currentPermission.value = ok ? getPermissionFromCookie() : null
    return ok
  } catch {
    isAuthenticated.value = false
    currentPermission.value = null
    return false
  } finally {
    authChecking.value = false
  }
}

export function logout() {
  const expiredAt = 'Thu, 01 Jan 1970 00:00:00 GMT'
  const paths = ['/', '/api']

  for (const path of paths) {
    document.cookie = `token=; Max-Age=0; path=${path}`
    document.cookie = `token=; Expires=${expiredAt}; path=${path}`
    document.cookie = `permisson=; Max-Age=0; path=${path}`
    document.cookie = `permisson=; Expires=${expiredAt}; path=${path}`
  }

  forceLoggedOut.value = true
  isAuthenticated.value = false
  currentPermission.value = null
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

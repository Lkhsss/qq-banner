import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from './views/HomeView.vue'
import BannedList from './views/BannedList.vue'
import ManagerList from './views/ManagerList.vue'
import NotFoundView from './views/NotFoundView.vue'
import LoginView from './views/LoginView.vue'
import PermissionDeniedView from './views/PermissionDeniedView.vue'
import MetricsView from './views/MetricsView.vue'
import {
  Permission,
  checkAuthByProbe,
  getPermissionFromCookie,
  hasPermission,
} from './services/auth'

function getDefaultRouteNameByPermission() {
  const permission = getPermissionFromCookie()

  if (permission === Permission.SuperAdmin || permission === Permission.Admin) {
    return 'metrics-panel'
  }

  return 'permission-denied'
}

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: LoginView,
    },
    {
      path: '/',
      name: 'home',
      component: HomeView,
      meta: { requiresAuth: true },
      children: [
        {
          path: '',
          redirect: { name: 'metrics-panel' },
        },
        {
          path: 'banned',
          name: 'banned-list',
          component: BannedList,
          meta: { requiresPermission: Permission.Admin },
        },
        {
          path: 'metrics',
          name: 'metrics-panel',
          component: MetricsView,
          meta: { requiresPermission: Permission.Admin },
        },
        {
          path: 'manager',
          name: 'manager-list',
          component: ManagerList,
          meta: { requiresPermission: Permission.SuperAdmin },
        },
        {
          path: 'forbidden',
          name: 'permission-denied',
          component: PermissionDeniedView,
        },
      ],
    },
    {
      path: '/:pathMatch(.*)*',
      name: 'not-found',
      component: NotFoundView,
    },
  ],
})

router.beforeEach(async (to) => {
  if (to.path === '/') {
    const ok = await checkAuthByProbe('/api/auth')
    return ok ? { name: getDefaultRouteNameByPermission() } : { name: 'login' }
  }

  if (to.name === 'login') {
    const ok = await checkAuthByProbe('/api/auth')
    return ok ? { name: getDefaultRouteNameByPermission() } : true
  }

  if (to.matched.some((record) => record.meta.requiresAuth)) {
    const ok = await checkAuthByProbe('/api/auth')

    if (!ok) {
      return { name: 'login' }
    }
  }

  const requiredPermission = to.matched
    .map((record) => record.meta.requiresPermission)
    .find((value): value is Permission => typeof value === 'number')

  if (requiredPermission !== undefined && !hasPermission(requiredPermission)) {
    return { name: 'permission-denied' }
  }

  return true
})

export default router

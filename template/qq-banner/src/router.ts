import { createRouter, createWebHistory } from 'vue-router'
import HomeView from './views/HomeView.vue'
import BannedList from './views/BannedList.vue'
import ManagerList from './views/ManagerList.vue'
import NotFoundView from './views/NotFoundView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
      children: [
        {
          path: '',
          redirect: { name: 'banned-list' },
        },
        {
          path: 'banned',
          name: 'banned-list',
          component: BannedList,
        },
        {
          path: 'manager',
          name: 'manager-list',
          component: ManagerList,
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

export default router

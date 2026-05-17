<script setup lang="ts">
import Button from 'primevue/button'
import { useRouter } from 'vue-router'

const router = useRouter()
</script>

<template>
  <main class="not-found">
    <h1>404</h1>
    <p>页面不存在</p>
    <Button label="返回首页" severity="secondary" variant="outlined" @click="router.push('/')" />
  </main>
</template>

<style scoped>
.not-found {
  min-height: 100vh;
  display: grid;
  place-content: center;
  gap: 0.75rem;
  text-align: center;
  background: var(--surface-ground);
  color: var(--text-color);
}

h1 {
  margin: 0;
  font-size: 4rem;
  line-height: 1;
  color: var(--text-color);
}

p {
  margin: 0;
  color: var(--text-color-secondary);
}
</style>

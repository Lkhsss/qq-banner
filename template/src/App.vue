<script setup lang="ts">
import Toast from 'primevue/toast'
import { RouterView } from 'vue-router'
</script>

<template>
  <Toast position="top-right" />
  <RouterView />
</template>

<style>
html,
body,
#app {
  min-height: 100%;
  background: var(--surface-ground);
  color: var(--text-color);
  transition: background 0.3s, color 0.3s;
}

html.my-app-dark,
html.my-app-dark body,
html.my-app-dark #app {
  color: var(--text-color);
}
</style>

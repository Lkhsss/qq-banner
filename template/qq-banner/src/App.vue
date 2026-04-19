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
}

html.my-app-dark,
html.my-app-dark body,
html.my-app-dark #app {
  background: #020617;
  color: #e2e8f0;
}
</style>

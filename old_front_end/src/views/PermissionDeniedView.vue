<template>
    <section class="panel">
        <h2>权限不足</h2>
        <p>当前账号没有可访问的功能，请联系超级管理员分配权限。</p>
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
    margin: 0;
    color: var(--text-color-secondary);
}
</style>

<template>
 <button class="recap-entry" hover-class="recap-entry--pressed" @tap="emit('open')"><view class="recap-entry__copy"><text class="recap-entry__title">我的赛季总结</text><text class="recap-entry__caption">把为蓉城走过的里程，做成一张海报</text></view><text class="recap-entry__arrow">›</text></button>
</template>
<script setup lang="ts">
const emit=defineEmits<{(event:'open'):void}>()
</script>
<style scoped>
.recap-entry{width:100%;margin:0;padding:var(--fi-space-24);display:flex;align-items:center;justify-content:space-between;gap:var(--fi-space-16);box-sizing:border-box;text-align:left;border:var(--fi-border-card);border-radius:var(--fi-radius-lg);background:var(--fi-color-surface);box-shadow:var(--fi-shadow-card);line-height:var(--fi-leading-normal);}
.recap-entry::after{border:0;}.recap-entry--pressed{opacity:var(--fi-opacity-pressed);}.recap-entry__copy{display:flex;flex-direction:column;gap:var(--fi-space-8);}
.recap-entry__title{color:var(--fi-color-text-primary);font-size:var(--fi-font-30);font-weight:var(--fi-weight-bold);}.recap-entry__caption{color:var(--fi-color-text-muted);font-size:var(--fi-font-24);}.recap-entry__arrow{color:var(--fi-color-primary);font-size:var(--fi-font-36);}
</style>

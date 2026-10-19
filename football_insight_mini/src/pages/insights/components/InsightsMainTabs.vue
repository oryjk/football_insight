<template>
  <view class="main-tabs" role="group" aria-label="洞察内容">
    <button v-for="item in items" :key="item.key" class="main-tabs__item" :class="{ active: modelValue === item.key }"
      :aria-pressed="modelValue === item.key" hover-class="main-tabs__pressed" @tap="emit('update:modelValue', item.key)">
      <text>{{ item.label }}</text>
      <view v-if="modelValue === item.key" class="main-tabs__indicator" />
    </button>
  </view>
</template>
<script setup lang="ts">
defineProps<{ modelValue: 'data' | 'tactics' }>()
const emit = defineEmits<{ (e: 'update:modelValue', value: 'data' | 'tactics'): void }>()
const items = [{ key: 'data', label: '数据洞察' }, { key: 'tactics', label: '战术' }] as const
</script>
<style scoped>
.main-tabs { display:flex; border-bottom:var(--fi-border-width) solid var(--fi-color-divider); }
.main-tabs__item { position:relative; flex:1; margin:0; padding:var(--fi-space-18) 0 var(--fi-space-22); background:transparent; color:var(--fi-color-text-secondary); font-size:var(--fi-font-30); line-height:var(--fi-leading-normal); font-weight:var(--fi-weight-bold); border-radius:0; }
.main-tabs__item::after { border:0; }
.main-tabs__item.active { color:var(--fi-color-primary); }
.main-tabs__indicator { position:absolute; bottom:0; left:40%; right:40%; height:var(--fi-space-6); border-radius:var(--fi-radius-round); background:var(--fi-color-primary); }
.main-tabs__pressed { opacity:var(--fi-opacity-pressed); }
</style>

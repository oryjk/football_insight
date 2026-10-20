<template>
 <view class="dock">
  <view class="dock__stats">
   <text class="dock__total">已选 {{stats.total}} 场</text>
   <text class="dock__detail">主 {{stats.home}} · 客 {{stats.away}}</text>
  </view>
  <button class="dock__action" :class="{'dock__action--disabled':generating}" :disabled="generating" hover-class="dock__action--pressed" @tap="handleAction">{{label}}</button>
 </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
const props = defineProps<{stats:{total:number;home:number;away:number};generating:boolean;hasPoster:boolean}>()
const emit = defineEmits<{(event:'generate'):void;(event:'open'):void}>()
const label = computed(() => props.generating ? '正在生成…' : props.hasPoster ? '查看海报' : '生成海报')
function handleAction() {
 if (props.generating) return
 if (props.hasPoster) emit('open'); else emit('generate')
}
</script>
<style scoped>
.dock {position:fixed;left:var(--fi-space-24);right:var(--fi-space-24);bottom:calc(var(--fi-space-24) + env(safe-area-inset-bottom));z-index:8;display:flex;align-items:center;justify-content:space-between;gap:var(--fi-space-16);padding:var(--fi-space-16) var(--fi-space-20);background:var(--fi-color-surface);border:var(--fi-border-card);border-radius:var(--fi-radius-lg);box-shadow:var(--fi-shadow-floating);}
.dock__stats {display:flex;flex-direction:column;gap:var(--fi-space-4);min-width:0;}
.dock__total {font-size:var(--fi-font-26);font-weight:var(--fi-weight-bold);color:var(--fi-color-text-primary);line-height:var(--fi-leading-snug);}
.dock__detail {font-size:var(--fi-font-22);color:var(--fi-color-text-muted);line-height:var(--fi-leading-normal);}
.dock__action {flex-shrink:0;margin:0;padding:var(--fi-space-16) var(--fi-space-28);background:var(--fi-color-primary);color:var(--fi-color-surface);border:0;border-radius:var(--fi-radius-round);font-size:var(--fi-font-28);font-weight:var(--fi-weight-bold);line-height:var(--fi-leading-normal);}
.dock__action::after {border:0;}
.dock__action--pressed {opacity:var(--fi-opacity-pressed);}
.dock__action--disabled {background:var(--fi-color-divider);color:var(--fi-color-text-muted);}
</style>

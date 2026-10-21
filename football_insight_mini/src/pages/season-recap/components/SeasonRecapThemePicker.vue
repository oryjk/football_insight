<template>
 <view class="theme-picker">
  <text class="theme-picker__title">海报样式</text>
  <view class="theme-picker__options">
   <button v-for="option in options" :key="option.key" class="theme-picker__option" :class="{'theme-picker__option--selected':modelValue===option.key}" :disabled="disabled" :aria-pressed="modelValue===option.key" hover-class="theme-picker__option--pressed" @tap="emit('update:modelValue',option.key)">
    <view class="theme-picker__swatch" :class="`theme-picker__swatch--${option.key}`" />
    <text>{{option.label}}</text>
   </button>
  </view>
 </view>
</template>
<script setup lang="ts">
import type { RecapPosterTheme } from '../poster'
defineProps<{modelValue:RecapPosterTheme;disabled:boolean}>()
const emit=defineEmits<{(event:'update:modelValue',value:RecapPosterTheme):void}>()
const options: Array<{key:RecapPosterTheme;label:string}>=[{key:'dark',label:'深色 · 热血夜场'},{key:'light',label:'浅色 · 赛季珍藏'}]
</script>
<style scoped>
.theme-picker {display:flex;flex-direction:column;gap:var(--fi-space-16);}
.theme-picker__title {font-size:var(--fi-font-26);font-weight:var(--fi-weight-bold);color:var(--fi-color-text-primary);}
.theme-picker__options {display:flex;gap:var(--fi-space-12);}
.theme-picker__option {flex:1;display:flex;align-items:center;justify-content:center;gap:var(--fi-space-12);min-width:0;margin:0;padding:var(--fi-space-16) var(--fi-space-12);border:var(--fi-border-card);border-radius:var(--fi-radius-md);background:var(--fi-color-page);color:var(--fi-color-text-secondary);font-size:var(--fi-font-24);line-height:var(--fi-leading-normal);}
.theme-picker__option::after {border:0;}
.theme-picker__option--selected {border-color:var(--fi-color-primary);color:var(--fi-color-primary);font-weight:var(--fi-weight-bold);}
.theme-picker__option--pressed {opacity:var(--fi-opacity-pressed);}
/* decorative：海报主题色样，和生成位图的配色一致。 */
.theme-picker__swatch {width:var(--fi-space-24);height:var(--fi-space-32);border-radius:var(--fi-radius-xs);flex-shrink:0;}
.theme-picker__swatch--dark {background:linear-gradient(135deg,#711623,#160a10);}
.theme-picker__swatch--light {background:linear-gradient(135deg,#fff7eb,#edcca0);}
</style>

<template>
 <view :class="overlay ? 'progress-mask' : 'progress-inline'" @touchmove.stop="">
  <view class="progress-card">
   <text class="progress-card__title">{{ title }}</text>
   <view class="progress-track">
    <view class="progress-fill" :style="{width:`${percent}%`}">
     <image class="progress-crest" :src="CHENGDU_RONGCHENG_CREST_IMAGE_URL" mode="aspectFit" />
    </view>
   </view>
   <view class="progress-meta">
    <text class="progress-meta__stage">{{stage}}</text>
    <text class="progress-meta__percent">{{percent}}%</text>
   </view>
  </view>
 </view>
</template>
<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { CHENGDU_RONGCHENG_CREST_IMAGE_URL } from '../config/assets'

const props = withDefaults(defineProps<{ title?: string; overlay?: boolean; stages?: [string, string, string, string] }>(), { title: '正在加载', overlay: false, stages: () => ['连接数据中…', '读取最新数据…', '整理榜单…', '马上就好…'] })
// 没有真实进度回调：按缓动曲线逼近 94%，完成后组件随调用方的 v-if 卸载。overlay 为全屏遮罩，否则是行内卡片。
const percent = ref(4)
let timer: ReturnType<typeof setInterval> | undefined
onMounted(() => { timer = setInterval(() => { percent.value = Math.min(94, percent.value + Math.max(1, Math.round((94 - percent.value) * 0.06))) }, 250) })
onBeforeUnmount(() => { if (timer) clearInterval(timer) })
const stage = computed(() => percent.value < 25 ? props.stages[0] : percent.value < 50 ? props.stages[1] : percent.value < 80 ? props.stages[2] : props.stages[3])
</script>
<style scoped>
.progress-mask {position:fixed;left:0;right:0;top:0;bottom:0;z-index:20;display:flex;align-items:center;justify-content:center;padding:0 var(--fi-space-48);background:rgba(0,0,0,0.45);}
.progress-inline .progress-card { box-shadow: none; border: var(--fi-border-card); }
.progress-card {width:100%;box-sizing:border-box;padding:var(--fi-space-32) var(--fi-space-28);background:var(--fi-color-surface);border-radius:var(--fi-radius-lg);box-shadow:var(--fi-shadow-floating);}
.progress-card__title {display:block;margin-bottom:var(--fi-space-28);font-size:var(--fi-font-30);font-weight:var(--fi-weight-bold);color:var(--fi-color-text-primary);text-align:center;}
.progress-track {position:relative;height:var(--fi-space-16);margin:0 var(--fi-space-24);background:var(--fi-color-divider);border-radius:var(--fi-radius-round);}
.progress-fill {position:relative;height:100%;min-width:var(--fi-space-16);background:linear-gradient(90deg,#ffda79,#ff963d,#e90016);border-radius:var(--fi-radius-round);transition:width 240ms linear;}
.progress-crest {position:absolute;right:calc(var(--fi-space-24) * -1);top:50%;width:var(--fi-space-48);height:var(--fi-space-48);margin-top:calc(var(--fi-space-24) * -1);}
.progress-meta {display:flex;justify-content:space-between;margin-top:var(--fi-space-28);}
.progress-meta__stage {font-size:var(--fi-font-24);color:var(--fi-color-text-secondary);}
.progress-meta__percent {font-size:var(--fi-font-24);font-weight:var(--fi-weight-bold);color:var(--fi-color-primary);}
</style>

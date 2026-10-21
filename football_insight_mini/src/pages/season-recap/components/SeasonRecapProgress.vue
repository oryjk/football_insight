<template>
 <view class="progress-mask" @touchmove.stop.prevent>
  <view class="progress-card">
   <text class="progress-card__title">正在生成赛季海报</text>
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
import { CHENGDU_RONGCHENG_CREST_IMAGE_URL } from '../../../config/assets'
// 生成没有真实进度回调：按缓动曲线逼近 94%，完成后组件随 generating 结束而卸载。
const percent = ref(4)
let timer: ReturnType<typeof setInterval> | undefined
onMounted(() => { timer = setInterval(() => { percent.value = Math.min(94, percent.value + Math.max(1, Math.round((94 - percent.value) * 0.06))) }, 250) })
onBeforeUnmount(() => { if (timer) clearInterval(timer) })
const stage = computed(() => percent.value < 25 ? '保存你的勾选…' : percent.value < 50 ? '准备二维码与队徽…' : percent.value < 80 ? '绘制海报…' : '导出高清图片…')
</script>
<style scoped>
.progress-mask {position:fixed;left:0;right:0;top:0;bottom:0;z-index:20;display:flex;align-items:center;justify-content:center;padding:0 var(--fi-space-48);background:rgba(0,0,0,0.45);}
.progress-card {width:100%;box-sizing:border-box;padding:var(--fi-space-32) var(--fi-space-28);background:var(--fi-color-surface);border-radius:var(--fi-radius-lg);box-shadow:var(--fi-shadow-floating);}
.progress-card__title {display:block;margin-bottom:var(--fi-space-28);font-size:var(--fi-font-30);font-weight:var(--fi-weight-bold);color:var(--fi-color-text-primary);text-align:center;}
.progress-track {position:relative;height:var(--fi-space-16);margin:0 var(--fi-space-24);background:var(--fi-color-divider);border-radius:var(--fi-radius-round);}
.progress-fill {position:relative;height:100%;min-width:var(--fi-space-16);background:linear-gradient(90deg,#ffda79,#ff963d,#e90016);border-radius:var(--fi-radius-round);transition:width 240ms linear;}
.progress-crest {position:absolute;right:calc(var(--fi-space-24) * -1);top:50%;width:var(--fi-space-48);height:var(--fi-space-48);margin-top:calc(var(--fi-space-24) * -1);}
.progress-meta {display:flex;justify-content:space-between;margin-top:var(--fi-space-28);}
.progress-meta__stage {font-size:var(--fi-font-24);color:var(--fi-color-text-secondary);}
.progress-meta__percent {font-size:var(--fi-font-24);font-weight:var(--fi-weight-bold);color:var(--fi-color-primary);}
</style>

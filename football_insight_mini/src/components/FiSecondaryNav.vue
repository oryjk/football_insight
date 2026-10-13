<template>
  <view class="fi-secondary-nav">
    <view class="fi-secondary-nav__status" />
    <view class="fi-secondary-nav__bar">
      <view class="fi-secondary-nav__row" :style="rowStyle">
        <view class="fi-secondary-nav__actions">
          <button class="fi-secondary-nav__action" aria-label="返回" @tap="emit('back')">
            <image :src="backIcon" class="fi-secondary-nav__icon" mode="aspectFit" />
          </button>
          <template v-if="shareable">
            <view class="fi-secondary-nav__divider" />
            <!-- #ifdef MP-WEIXIN -->
            <button class="fi-secondary-nav__action" aria-label="分享" open-type="share" :disabled="shareDisabled">
              <image :src="shareIcon" class="fi-secondary-nav__icon" mode="aspectFit" />
            </button>
            <!-- #endif -->
            <!-- #ifndef MP-WEIXIN -->
            <button class="fi-secondary-nav__action" aria-label="分享" :disabled="shareDisabled" @tap="emit('share')">
              <image :src="shareIcon" class="fi-secondary-nav__icon" mode="aspectFit" />
            </button>
            <!-- #endif -->
          </template>
        </view>
        <text class="fi-secondary-nav__title">足球洞察</text>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import backIcon from '../static/navigation/back.png'
import shareIcon from '../static/navigation/share.png'

withDefaults(defineProps<{ shareable?: boolean; shareDisabled?: boolean }>(), {
  shareable: true,
  shareDisabled: false,
})
const emit = defineEmits<{ (e: 'back'): void; (e: 'share'): void }>()
const metrics = ref<{ top: number; height: number } | null>(null)
const rowStyle = computed(() => metrics.value
  ? { top: `${metrics.value.top}px`, height: `${metrics.value.height}px` } : {})
onMounted(() => {
  // #ifdef MP-WEIXIN
  try {
    const capsule = uni.getMenuButtonBoundingClientRect?.()
    if (capsule?.width && capsule.height) metrics.value = { top: capsule.top, height: capsule.height }
  } catch { /* CSS fallback keeps the navigation usable. */ }
  // #endif
})
</script>

<style scoped>
.fi-secondary-nav { position: fixed; top: 0; left: 0; right: 0; z-index: 80; background: var(--fi-color-brand-nav-bg); border-bottom: var(--fi-border-width) solid var(--fi-color-brand-nav-border); }
.fi-secondary-nav__status { height: var(--status-bar-height, 0px); }
.fi-secondary-nav__bar { min-height: var(--fi-brand-nav-bar-height); }
.fi-secondary-nav__row { position: fixed; top: calc(var(--status-bar-height, 0px) + var(--fi-space-12)); left: var(--fi-space-24); right: var(--fi-space-24); height: var(--fi-space-brand-nav-row-size); }
.fi-secondary-nav__actions { position: absolute; left: 0; top: 0; height: 100%; display: flex; align-items: center; border: var(--fi-border-card); border-radius: var(--fi-radius-round); background: var(--fi-color-bg); overflow: hidden; }
.fi-secondary-nav__action { width: var(--fi-space-secondary-nav-action); height: 100%; display: flex; align-items: center; justify-content: center; padding: 0; margin: 0; border: 0; background: transparent; }
.fi-secondary-nav__action::after { border: 0; }
.fi-secondary-nav__action[disabled] { opacity: 0.4; }
.fi-secondary-nav__icon { width: var(--fi-space-36); height: var(--fi-space-36); }
.fi-secondary-nav__divider { width: var(--fi-border-width); height: 60%; background: var(--fi-color-divider); }
.fi-secondary-nav__title { position: absolute; left: var(--fi-space-brand-nav-capsule); right: var(--fi-space-brand-nav-capsule); height: 100%; display: flex; align-items: center; justify-content: center; color: var(--fi-color-text-primary); font-size: var(--fi-font-32); font-weight: var(--fi-weight-bold); line-height: var(--fi-leading-none); white-space: nowrap; pointer-events: none; }
</style>

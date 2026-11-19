<template>
  <view class="fi-layout-block" :class="{ 'fi-layout-block--bg': hasBg, 'fi-layout-block--fill': fill }">
    <view v-if="hasBg" class="fi-layout-block__bg" :style="bgStyle" aria-hidden="true"></view>
    <view v-if="hasBg" class="fi-layout-block__wash" aria-hidden="true"></view>
    <view class="fi-layout-block__inner">
      <slot />
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    backgroundImageUrl?: string | null
    fill?: boolean
  }>(),
  {
    backgroundImageUrl: null,
    fill: false,
  },
)

const trimmedUrl = computed(() => props.backgroundImageUrl?.trim() ?? '')
const hasBg = computed(() => Boolean(trimmedUrl.value))
const bgStyle = computed(() =>
  trimmedUrl.value ? { backgroundImage: `url(${trimmedUrl.value})` } : {},
)
</script>

<style scoped lang="css">
.fi-layout-block {
  width: 100%;
  min-width: 0;
}

.fi-layout-block--fill {
  flex: 1;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.fi-layout-block--bg {
  position: relative;
  border-radius: var(--fi-radius-xl);
  overflow: hidden;
  --fi-component-panel-bg: var(--fi-color-layout-card-scrim);
  --fi-color-surface-frosted: var(--fi-color-layout-card-scrim);
  --fi-color-surface-panel: var(--fi-color-layout-card-scrim);
  --fi-color-page-soft: var(--fi-color-layout-card-scrim);
}

.fi-layout-block__bg {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-repeat: no-repeat;
  background-size: cover;
  background-position: center;
  pointer-events: none;
  z-index: 0;
}

.fi-layout-block__wash {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--fi-color-layout-card-wash);
  pointer-events: none;
  z-index: 0;
}

.fi-layout-block__inner {
  position: relative;
  z-index: 1;
  width: 100%;
  min-width: 0;
}

.fi-layout-block--fill .fi-layout-block__inner {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
</style>

<template>
  <view v-if="resolvedImageUrl" class="fi-page-backdrop" :class="{ 'fi-page-backdrop--full': fullPage }" aria-hidden="true">
    <view class="fi-page-backdrop__image" :style="imageStyle"></view>
    <view class="fi-page-backdrop__fade" :class="{ 'fi-page-backdrop__fade--custom': isCustomImage }"></view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    imageUrl?: string | null
    fallbackImageUrl?: string | null
    fullPage?: boolean
  }>(),
  {
    imageUrl: null,
    fallbackImageUrl: null,
    fullPage: false,
  },
)

const isCustomImage = computed(() => Boolean(props.imageUrl && props.imageUrl.trim()))
const resolvedImageUrl = computed(() => {
  const custom = props.imageUrl?.trim()
  if (custom) return custom
  const fallback = props.fallbackImageUrl?.trim()
  return fallback || ''
})

const imageStyle = computed(() =>
  resolvedImageUrl.value ? { backgroundImage: `url(${resolvedImageUrl.value})` } : {},
)
</script>

<style scoped lang="css">
.fi-page-backdrop {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: var(--fi-space-600);
  pointer-events: none;
  z-index: 0;
  overflow: hidden;
}

.fi-page-backdrop--full {
  position: fixed;
  bottom: 0;
  height: auto;
}

.fi-page-backdrop__image {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-repeat: no-repeat;
  background-size: cover;
  background-position: center top;
}

.fi-page-backdrop__fade {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--fi-color-page-backdrop-fade);
}

.fi-page-backdrop__fade--custom {
  background: var(--fi-color-layout-page-fade);
}
</style>

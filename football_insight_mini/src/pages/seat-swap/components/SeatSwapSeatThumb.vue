<template>
  <view
    class="seat-thumb"
    hover-class="seat-thumb--pressed"
    role="button"
    aria-label="查看球场座位位置"
    @tap.stop="emit('open')"
  >
    <image class="seat-thumb__base" src="/static/seat-swap/stadium-thumb.svg" mode="scaleToFill" />
    <view
      v-for="item in highlights"
      :key="item.key"
      class="seat-thumb__mark"
      :class="`seat-thumb__mark--${item.kind}`"
      :style="item.style"
    />
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { resolveSeatSwapThumbHighlights } from '../helpers'

const props = defineProps<{
  currentKey: string
  desiredKeys: string[]
}>()

const emit = defineEmits<{ (e: 'open'): void }>()

const highlights = computed(() => resolveSeatSwapThumbHighlights(props.currentKey, props.desiredKeys))
</script>

<style scoped>
/* 宽高比与大图球场区域一致（750×560）。只画轮廓与高亮，不显示任何文字。 */
.seat-thumb {
  position: relative;
  flex-shrink: 0;
  width: var(--fi-component-seat-swap-thumb-width);
  height: var(--fi-component-seat-swap-thumb-height);
  overflow: hidden;
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-page);
  transition: transform var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}

.seat-thumb--pressed {
  transform: scale(0.96);
}

.seat-thumb__base {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

/* 缩略图里分区很小：高亮块放大并加白边，保证一眼能看到。 */
.seat-thumb__mark {
  position: absolute;
  border-radius: var(--fi-radius-round);
  box-shadow: 0 0 0 var(--fi-space-2) var(--fi-primitive-white);
  transform: scale(1.6);
}

.seat-thumb__mark--current {
  z-index: 2;
  background: var(--fi-color-seat-current);
}

.seat-thumb__mark--desired {
  z-index: 1;
  background: var(--fi-color-seat-desired);
}
</style>

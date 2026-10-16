<template>
  <view v-if="rendered" class="fi-collapse" :class="`fi-collapse--${phase}`">
    <view class="fi-collapse__inner">
      <slot />
    </view>
  </view>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'

// Keep in sync with the keyframe duration below.
const DURATION_MS = 280

const props = defineProps<{ open: boolean }>()

type Phase = 'open' | 'opening' | 'closing'
// Content is only mounted while open or animating, so collapsed cards stay cheap.
const rendered = ref(props.open)
const phase = ref<Phase>('open')
let timer: ReturnType<typeof setTimeout> | null = null

function clearTimer() {
  if (timer !== null) clearTimeout(timer)
  timer = null
}

// Animations are CSS keyframes, so no height measuring or frame timing is needed.
watch(() => props.open, (open) => {
  clearTimer()
  if (open) {
    rendered.value = true
    phase.value = 'opening'
    timer = setTimeout(() => {
      phase.value = 'open'
    }, DURATION_MS)
  } else {
    phase.value = 'closing'
    timer = setTimeout(() => {
      rendered.value = false
    }, DURATION_MS)
  }
})

onBeforeUnmount(clearTimer)
</script>

<style scoped lang="css">
.fi-collapse {
  display: grid;
  grid-template-rows: 1fr;
}

/* flow-root keeps the first/last child's margins inside the animated height. */
.fi-collapse__inner {
  min-height: 0;
  display: flow-root;
}

/* Clip only while moving, so shadows are not cut off in the resting state. */
.fi-collapse--opening,
.fi-collapse--closing,
.fi-collapse--opening .fi-collapse__inner,
.fi-collapse--closing .fi-collapse__inner {
  overflow: hidden;
}

.fi-collapse--opening {
  animation: fi-collapse-open 280ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.fi-collapse--closing {
  animation: fi-collapse-close 280ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

@keyframes fi-collapse-open {
  from {
    grid-template-rows: 0fr;
    opacity: 0;
  }
  to {
    grid-template-rows: 1fr;
    opacity: 1;
  }
}

@keyframes fi-collapse-close {
  from {
    grid-template-rows: 1fr;
    opacity: 1;
  }
  to {
    grid-template-rows: 0fr;
    opacity: 0;
  }
}
</style>

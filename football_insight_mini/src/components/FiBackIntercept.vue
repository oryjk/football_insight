<template>
  <!-- #ifdef MP-WEIXIN -->
  <page-container
    v-if="isTopmost"
    :show="true"
    :overlay="false"
    :duration="0"
    custom-style="background: transparent; height: 0; overflow: hidden;"
    @beforeleave="handleBeforeLeave"
  />
  <!-- #endif -->
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, watch } from 'vue'
import {
  allocateBackInterceptId,
  isTopBackIntercept,
  pushBackIntercept,
  removeBackIntercept,
} from '../utils/backIntercept'

const props = withDefaults(defineProps<{ active?: boolean }>(), {
  active: true,
})

const emit = defineEmits<{
  (e: 'leave'): void
}>()

const interceptId = allocateBackInterceptId()
const isTopmost = computed(() => props.active && isTopBackIntercept(interceptId))

watch(
  () => props.active,
  (active) => {
    if (active) {
      pushBackIntercept(interceptId)
    } else {
      removeBackIntercept(interceptId)
    }
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  removeBackIntercept(interceptId)
})

function handleBeforeLeave(): void {
  if (!isTopmost.value) return
  emit('leave')
}
</script>

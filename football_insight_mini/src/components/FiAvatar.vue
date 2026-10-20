<template>
  <!-- 球员头像：先小图，加载失败改用原图，再失败显示名字首字。尺寸与圆角由使用方的外层 view 决定。 -->
  <image v-if="current" class="fi-avatar" :src="current" mode="aspectFill" :lazy-load="lazy" @error="next" />
  <view v-else class="fi-avatar fi-avatar--fallback"><text>{{ initial }}</text></view>
</template>
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { avatarCandidates, type AvatarThumbSize } from '../utils/avatarThumb'
const props = withDefaults(defineProps<{ src: string | null; size: AvatarThumbSize; name: string; lazy?: boolean }>(), { lazy: false })
const index = ref(0)
const candidates = computed(() => avatarCandidates(props.src, props.size))
const current = computed(() => candidates.value[index.value] ?? '')
const initial = computed(() => Array.from(props.name.trim())[0] ?? '')
watch(() => props.src, () => { index.value = 0 })
function next(): void { index.value += 1 }
</script>
<style scoped>
.fi-avatar { display: block; width: 100%; height: 100%; border-radius: inherit; }
.fi-avatar--fallback { display: flex; align-items: center; justify-content: center; color: var(--fi-color-text-secondary); font-size: inherit; font-weight: var(--fi-weight-bold); }
</style>

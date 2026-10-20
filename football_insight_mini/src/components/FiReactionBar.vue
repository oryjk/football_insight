<template>
  <!-- 浅底描边胶囊：左赞（带赞数）右踩（可选踩数），互斥；sm 用于列表行，md 用于详情页按钮行。战术板与转会投票共用。 -->
  <view class="reactions" :class="[`reactions--${size}`, { 'reactions--pending': pending }]" :aria-busy="pending" @tap.stop>
    <button class="reaction" :class="{ 'reaction--liked': liked }" hover-class="reaction--pressed"
      :aria-label="liked ? '取消点赞' : '点赞'" :aria-pressed="liked" :aria-disabled="pending" @tap.stop="tap('like')">
      <!-- Lucide thumbs-up/down raster（scripts/generate-reaction-icons.mjs）；ISC license in static/home/lucide-LICENSE.txt. -->
      <image class="reaction__icon" :src="liked ? ICONS.likeActive : ICONS.likeIdle" mode="aspectFit" />
      <text class="reaction__count">{{ count }}</text>
    </button>
    <view class="reactions__divider" />
    <button class="reaction reaction--dislike" :class="{ 'reaction--disliked': disliked, 'reaction--dislike-count': dislikeCount !== undefined }" hover-class="reaction--pressed"
      :aria-label="disliked ? '取消点踩' : '点踩'" :aria-pressed="disliked" :aria-disabled="pending" @tap.stop="tap('dislike')">
      <image class="reaction__icon" :src="disliked ? ICONS.dislikeActive : ICONS.dislikeIdle" mode="aspectFit" />
      <text v-if="dislikeCount !== undefined" class="reaction__count">{{ dislikeCount }}</text>
    </button>
  </view>
</template>
<script setup lang="ts">
const props = withDefaults(defineProps<{ count: number; liked: boolean; disliked?: boolean; pending: boolean; size?: 'sm' | 'md'; dislikeCount?: number }>(), {
  disliked: false,
  size: 'sm',
  dislikeCount: undefined,
})
const emit = defineEmits<{ (e: 'like' | 'dislike'): void }>()
const ICONS = {
  likeIdle: '/static/lineup/reaction-like-idle.png',
  likeActive: '/static/lineup/reaction-like-active.png',
  dislikeIdle: '/static/lineup/reaction-dislike-idle.png',
  dislikeActive: '/static/lineup/reaction-dislike-active.png',
}
// 不用原生 disabled：微信会给 button[disabled] 强制套灰底灰字，选中态会在请求中闪成灰色。
function tap(kind: 'like' | 'dislike'): void {
  if (!props.pending) emit(kind)
}
</script>
<style scoped>
.reactions { display: flex; flex-shrink: 0; align-items: center; box-sizing: border-box; overflow: hidden; border: var(--fi-border-width) solid var(--fi-color-reaction-border); border-radius: var(--fi-radius-round); background: var(--fi-color-reaction-bg); transition: opacity var(--fi-motion-duration-fast) var(--fi-motion-ease-out); }
.reactions--sm { height: var(--fi-component-reaction-height-sm); }
.reactions--md { height: var(--fi-component-reaction-height-md); }
.reactions--pending { opacity: var(--fi-opacity-pressed); }
.reaction { display: flex; flex-shrink: 0; align-items: center; justify-content: center; gap: var(--fi-space-8); box-sizing: border-box; height: 100%; margin: 0; padding: 0 var(--fi-space-16); border: 0; border-radius: 0; background: transparent; color: var(--fi-color-reaction-text); line-height: var(--fi-leading-none); transition: background-color var(--fi-motion-duration-fast) var(--fi-motion-ease-out), color var(--fi-motion-duration-fast) var(--fi-motion-ease-out); }
.reaction::after { border: 0; }
.reactions--md .reaction { gap: var(--fi-space-10); padding: 0 var(--fi-space-24); }
.reaction--dislike { padding: 0 var(--fi-space-14); }
.reactions--md .reaction--dislike { padding: 0 var(--fi-space-22); }
.reaction--dislike-count { padding: 0 var(--fi-space-16); }
.reactions--md .reaction--dislike-count { padding: 0 var(--fi-space-24); }
.reaction--liked { background: var(--fi-color-reaction-like-bg); color: var(--fi-color-reaction-like); }
.reaction--disliked { background: var(--fi-color-reaction-dislike-bg); color: var(--fi-color-reaction-dislike); }
.reaction--pressed { opacity: var(--fi-opacity-pressed); }
.reaction__icon { width: var(--fi-space-28); height: var(--fi-space-28); }
.reactions--md .reaction__icon { width: var(--fi-space-32); height: var(--fi-space-32); }
.reaction__count { font-size: var(--fi-font-22); font-weight: var(--fi-weight-bold); font-variant-numeric: tabular-nums; }
.reactions--md .reaction__count { font-size: var(--fi-font-26); }
.reactions__divider { flex-shrink: 0; width: var(--fi-border-width); height: 50%; background: var(--fi-color-reaction-border); }
</style>

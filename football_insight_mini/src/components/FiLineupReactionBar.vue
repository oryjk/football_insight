<template>
  <view class="reactions" :class="{ 'reactions--pending': pending }" @tap.stop>
    <button class="reaction reaction--like" :class="{ 'reaction--selected': liked }" :disabled="pending"
      :aria-label="liked ? '取消点赞' : '点赞'" :aria-pressed="liked" @tap.stop="emit('like')">
      <!-- Lucide thumbs-up raster; ISC license in static/home/lucide-LICENSE.txt. -->
      <image class="reaction__icon" src="/static/lineup/like.png" mode="aspectFit" />
      <text class="reaction__count">{{ count }}</text>
    </button>
    <view class="reactions__divider" />
    <button class="reaction reaction--dislike" :class="{ 'reaction--selected': disliked }" :disabled="pending"
      :aria-label="disliked ? '取消点踩' : '点踩'" :aria-pressed="disliked" @tap.stop="emit('dislike')">
      <image class="reaction__icon reaction__icon--down" src="/static/lineup/like.png" mode="aspectFit" />
    </button>
  </view>
</template>
<script setup lang="ts">
withDefaults(defineProps<{ count: number; liked: boolean; disliked?: boolean; pending: boolean }>(), { disliked: false })
const emit = defineEmits<{ (e: 'like' | 'dislike'): void }>()
</script>
<style scoped>
.reactions { display:flex; flex-shrink:0; align-items:center; overflow:hidden; border-radius:var(--fi-radius-round); background:linear-gradient(var(--fi-color-reaction-gloss), transparent),var(--fi-color-reaction-bg); color:var(--fi-color-reaction-text); }
.reaction { display:flex; align-items:center; justify-content:center; flex-shrink:0; gap:var(--fi-space-10); height:var(--fi-space-64); margin:0; padding:0 var(--fi-space-20); border:0; border-radius:0; background:transparent; color:var(--fi-color-reaction-text); line-height:var(--fi-leading-none); }
.reaction::after { border:0; }
.reaction--selected { background:var(--fi-color-reaction-selected); }
.reaction--dislike { width:var(--fi-space-64); padding:0; }
.reaction__icon { width:var(--fi-space-32); height:var(--fi-space-32); filter:brightness(0) invert(1); opacity:0.8; }
.reaction--selected .reaction__icon { opacity:1; }
.reaction__icon--down { transform:rotate(180deg); }
.reaction__count { font-size:var(--fi-font-24); font-weight:var(--fi-weight-bold); }
.reactions__divider { flex-shrink:0; width:var(--fi-border-width); height:var(--fi-space-32); background:var(--fi-color-reaction-divider); }
.reactions--pending { opacity:0.55; }
</style>

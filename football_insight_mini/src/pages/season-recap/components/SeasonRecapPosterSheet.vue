<template>
 <FiBottomSheet :visible="visible" title="我的赛季海报" @update:visible="emit('update:visible',$event)">
  <view class="poster-body">
   <image v-if="posterPath" :src="posterPath" class="poster-body__image" mode="widthFix" @tap="emit('preview')" />
  </view>
  <template #footer>
   <!-- #ifdef MP-WEIXIN -->
   <button class="poster-action" hover-class="poster-action--pressed" @tap="emit('save')">保存到相册</button>
   <button class="poster-action poster-action--secondary" open-type="share" hover-class="poster-action--pressed">分享给好友</button>
   <!-- #endif -->
   <!-- #ifndef MP-WEIXIN -->
   <button class="poster-action" hover-class="poster-action--pressed" @tap="emit('preview')">查看大图，长按保存</button>
   <button class="poster-action poster-action--secondary" hover-class="poster-action--pressed" @tap="emit('copy')">复制分享链接</button>
   <!-- #endif -->
  </template>
 </FiBottomSheet>
</template>
<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
defineProps<{visible:boolean;posterPath:string}>()
const emit = defineEmits<{(event:'update:visible',value:boolean):void;(event:'save'):void;(event:'preview'):void;(event:'copy'):void}>()
</script>
<style scoped>
.poster-body {display:flex;flex-direction:column;align-items:stretch;}
.poster-body__image {width:100%;display:block;border-radius:var(--fi-radius-md);}
.poster-action {flex:1;margin:0;padding:var(--fi-space-20) var(--fi-space-24);background:var(--fi-color-primary);color:var(--fi-color-surface);border:0;border-radius:var(--fi-radius-round);font-size:var(--fi-font-28);font-weight:var(--fi-weight-bold);line-height:var(--fi-leading-normal);}
.poster-action::after {border:0;}
.poster-action--secondary {background:var(--fi-color-page);color:var(--fi-color-text-primary);}
.poster-action--pressed {opacity:var(--fi-opacity-pressed);}
</style>

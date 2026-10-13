<template>
  <view class="basic-account">
    <view class="basic-account__identity">
      <view class="basic-account__avatar">
        <image v-if="user.avatar_url" :src="user.avatar_url" mode="aspectFill" class="basic-account__avatar-image" />
        <text v-else>{{ name.slice(0, 1) }}</text>
      </view>
      <view class="basic-account__copy">
        <text class="basic-account__name">{{ name }}</text>
        <text class="basic-account__status">已登录</text>
      </view>
    </view>
    <button class="basic-account__logout" @tap="emit('logout')">退出登录</button>
  </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import type { CurrentUser } from '../../../types/auth'
const props = defineProps<{ user: CurrentUser }>()
const emit = defineEmits<{ (e: 'logout'): void }>()
const name = computed(() => props.user.display_name?.trim() || '球迷')
</script>
<style scoped>
.basic-account { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-16); padding: var(--fi-space-28); background: var(--fi-color-bg); border: var(--fi-border-card); border-radius: var(--fi-radius-xl); box-shadow: var(--fi-shadow-soft); }
.basic-account__identity { display: flex; align-items: center; gap: var(--fi-space-16); min-width: 0; }
.basic-account__avatar { flex-shrink: 0; width: var(--fi-space-64); height: var(--fi-space-64); border-radius: var(--fi-radius-round); overflow: hidden; display: flex; align-items: center; justify-content: center; background: var(--fi-color-page); color: var(--fi-color-text-primary); font-size: var(--fi-font-30); font-weight: var(--fi-weight-bold); }
.basic-account__avatar-image { width: 100%; height: 100%; }
.basic-account__copy { min-width: 0; }
.basic-account__name { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--fi-color-text-primary); font-size: var(--fi-font-30); font-weight: var(--fi-weight-bold); }
.basic-account__status { display: block; margin-top: var(--fi-space-6); color: var(--fi-color-success-accent); font-size: var(--fi-font-22); }
.basic-account__logout { flex-shrink: 0; margin: 0; padding: var(--fi-space-12) var(--fi-space-16); background: var(--fi-color-page); color: var(--fi-color-text-secondary); border-radius: var(--fi-radius-round); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.basic-account__logout::after { border: 0; }
</style>

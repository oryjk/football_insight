<template>
  <view v-if="users.length" class="panel h5-test-login-panel">
    <view class="h5-test-login-panel__header">
      <text class="h5-test-login-panel__title">测试账号登录</text>
      <text class="h5-test-login-panel__caption">仅 H5 测试环境可用，点击直接切换账号</text>
    </view>

    <view class="h5-test-login-panel__search">
      <input
        v-model="keyword"
        class="h5-test-login-panel__search-input"
        type="text"
        placeholder="按昵称模糊搜索"
        placeholder-class="h5-test-login-panel__search-placeholder"
        confirm-type="search"
      />
    </view>

    <view v-if="filteredUsers.length" class="h5-test-login-panel__list">
      <view
        v-for="user in filteredUsers"
        :key="user.id"
        class="h5-test-login-panel__item"
        :class="{ 'h5-test-login-panel__item--active': user.id === currentUserId }"
        hover-class="h5-test-login-panel__item--pressed"
        hover-stay-time="100"
        @click="handleSelect(user)"
      >
        <view class="h5-test-login-panel__item-body">
          <text class="h5-test-login-panel__item-name">{{ resolveDisplayName(user) }}</text>
          <text class="h5-test-login-panel__item-identifier">{{ user.account_identifier }}</text>
        </view>
        <text v-if="user.id === currentUserId" class="h5-test-login-panel__item-badge">当前</text>
        <text v-else class="h5-test-login-panel__item-arrow">›</text>
      </view>
    </view>

    <text v-else class="h5-test-login-panel__empty">没有昵称匹配「{{ keyword.trim() }}」的测试账号</text>
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { H5TestLoginUser } from '../../../types/auth'

const props = defineProps<{
  users: H5TestLoginUser[]
  currentUserId?: string
}>()

const emit = defineEmits<{
  (e: 'select', user: H5TestLoginUser): void
}>()

const keyword = ref('')

const filteredUsers = computed(() => {
  const query = keyword.value.trim().toLowerCase()
  if (!query) {
    return props.users
  }

  // 按昵称模糊匹配，昵称缺失时回退到账号名匹配。
  return props.users.filter((user) =>
    (user.display_name?.trim() || user.account_identifier).toLowerCase().includes(query),
  )
})

function resolveDisplayName(user: H5TestLoginUser): string {
  const displayName = user.display_name?.trim()
  return displayName || user.account_identifier
}

function handleSelect(user: H5TestLoginUser): void {
  if (user.id === props.currentUserId) {
    return
  }

  emit('select', user)
}

</script>

<style scoped lang="css">
.panel {
  position: relative;
  z-index: 1;
  background: rgba(255,255,255,0.72);
  border-radius: var(--fi-radius-xl);
  padding: 20rpx;
  border: 2rpx solid rgba(255,255,255,0.55);
  box-shadow: 0 20rpx 48rpx rgba(26,28,36,0.06);
  backdrop-filter: blur(18rpx);
  -webkit-backdrop-filter: blur(18rpx);
}

.h5-test-login-panel {
  padding: 22rpx;
  border-radius: var(--fi-radius-lg);
  border: 1rpx dashed rgba(214, 158, 46, 0.72);
  background: rgba(255, 250, 235, 0.9);
}

.h5-test-login-panel__header {
  display: grid;
  gap: 8rpx;
}

.h5-test-login-panel__title {
  color: #1b1c20;
  font-size: var(--fi-font-30);
  font-weight: 800;
  line-height: 1;
}

.h5-test-login-panel__caption {
  color: #96917a;
  font-size: var(--fi-font-22);
  line-height: 1.4;
}

.h5-test-login-panel__search {
  margin-top: 16rpx;
}

.h5-test-login-panel__search-input {
  height: 64rpx;
  padding: 0 24rpx;
  box-sizing: border-box;
  border-radius: var(--fi-radius-round);
  border: 1rpx solid rgba(232, 226, 208, 0.98);
  background: rgba(255, 255, 255, 0.9);
  color: #17181c;
  font-size: 25rpx;
  line-height: 64rpx;
}

.h5-test-login-panel__search-placeholder {
  color: #969ca8;
  font-size: 25rpx;
}

.h5-test-login-panel__list {
  margin-top: 18rpx;
  display: grid;
  gap: 10rpx;
}

.h5-test-login-panel__empty {
  display: block;
  margin-top: 18rpx;
  color: #96917a;
  font-size: var(--fi-font-24);
  line-height: 1.5;
  text-align: center;
}

.h5-test-login-panel__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14rpx;
  padding: 18rpx;
  box-sizing: border-box;
  border-radius: 22rpx;
  border: 1rpx solid rgba(232, 226, 208, 0.98);
  background: rgba(255, 255, 255, 0.82);
}

.h5-test-login-panel__item--pressed {
  transform: scale(0.99);
  background: rgba(255, 244, 214, 0.95);
}

.h5-test-login-panel__item--active {
  border-color: rgba(214, 158, 46, 0.72);
}

.h5-test-login-panel__item-body {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 8rpx;
}

.h5-test-login-panel__item-name {
  color: #17181c;
  font-size: 25rpx;
  font-weight: 800;
  line-height: 1.15;
  word-break: break-word;
}

.h5-test-login-panel__item-identifier {
  color: #969ca8;
  font-size: var(--fi-font-22);
  line-height: 1;
  word-break: break-all;
}

.h5-test-login-panel__item-arrow {
  flex-shrink: 0;
  color: #b7a877;
  font-size: 36rpx;
  line-height: 1;
  font-weight: 700;
}

.h5-test-login-panel__item-badge {
  flex-shrink: 0;
  padding: 8rpx 18rpx;
  border-radius: var(--fi-radius-round);
  background: rgba(214, 158, 46, 0.16);
  color: #a0751c;
  font-size: var(--fi-font-22);
  font-weight: 700;
  line-height: 1;
}
</style>

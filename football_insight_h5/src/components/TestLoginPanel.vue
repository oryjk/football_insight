<template>
  <div v-if="users.length" class="test-login">
    <div class="test-login__header">
      <span class="test-login__title">测试账号登录</span>
      <span class="test-login__caption">仅测试环境可用，点击直接切换账号</span>
    </div>

    <div class="test-login__search">
      <input
        v-model="keyword"
        class="test-login__search-input"
        type="text"
        placeholder="按昵称模糊搜索"
      />
    </div>

    <div v-if="filteredUsers.length" class="test-login__list">
      <div
        v-for="user in filteredUsers"
        :key="user.id"
        class="test-login__item"
        @click="emit('select', user)"
      >
        <div class="test-login__item-body">
          <span class="test-login__item-name">{{ resolveDisplayName(user) }}</span>
          <span class="test-login__item-identifier">{{ user.account_identifier }}</span>
        </div>
        <span class="test-login__item-arrow">›</span>
      </div>
    </div>

    <span v-else class="test-login__empty">没有昵称匹配「{{ keyword.trim() }}」的测试账号</span>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { H5TestLoginUser } from '../types/auth'

const props = defineProps<{
  users: H5TestLoginUser[]
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
  return props.users.filter((user) =>
    (user.display_name?.trim() || user.account_identifier).toLowerCase().includes(query),
  )
})

function resolveDisplayName(user: H5TestLoginUser): string {
  const displayName = user.display_name?.trim()
  return displayName || user.account_identifier
}
</script>

<style scoped>
.test-login {
  padding: 12px;
  border-radius: 10px;
  border: 1px dashed rgba(214, 158, 46, 0.72);
  background: rgba(255, 250, 235, 0.92);
}

.test-login__header {
  display: grid;
  gap: 4px;
}

.test-login__title {
  color: #1b1c20;
  font-size: 15px;
  font-weight: 800;
}

.test-login__caption {
  color: #96917a;
  font-size: 11px;
}

.test-login__search {
  margin-top: 8px;
}

.test-login__search-input {
  width: 100%;
  height: 32px;
  padding: 0 12px;
  box-sizing: border-box;
  border-radius: 999px;
  border: 1px solid rgba(232, 226, 208, 0.98);
  background: rgba(255, 255, 255, 0.9);
  color: #17181c;
  font-size: 13px;
  outline: none;
}

.test-login__search-input::placeholder {
  color: #969ca8;
}

.test-login__list {
  margin-top: 9px;
  display: grid;
  gap: 6px;
}

.test-login__empty {
  display: block;
  margin-top: 9px;
  color: #96917a;
  font-size: 12px;
  text-align: center;
}

.test-login__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 9px;
  border-radius: 9px;
  border: 1px solid rgba(232, 226, 208, 0.98);
  background: rgba(255, 255, 255, 0.82);
  cursor: pointer;
}

.test-login__item:hover {
  background: rgba(255, 244, 214, 0.95);
}

.test-login__item-body {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 3px;
}

.test-login__item-name {
  color: #17181c;
  font-size: 13px;
  font-weight: 800;
}

.test-login__item-identifier {
  color: #969ca8;
  font-size: 11px;
  word-break: break-all;
}

.test-login__item-arrow {
  flex-shrink: 0;
  color: #b7a877;
  font-size: 18px;
  font-weight: 700;
}
</style>

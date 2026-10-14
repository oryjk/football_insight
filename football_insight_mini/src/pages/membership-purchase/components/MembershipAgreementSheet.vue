<template>
  <FiBottomSheet
    :visible="visible"
    title="会员服务声明"
    height="76vh"
    @update:visible="emit('update:visible', $event)"
  >
    <view class="agreement-list">
      <view v-for="(point, index) in points" :key="`agreement-${index}`" class="agreement-list__item">
        <text class="agreement-list__index">{{ index + 1 }}.</text>
        <text class="agreement-list__text">{{ point }}</text>
      </view>
    </view>

    <template #footer>
      <button class="agreement-accept" @tap="emit('accept')">我已阅读并同意</button>
    </template>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'

defineProps<{
  visible: boolean
  points: string[]
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'accept'): void
}>()
</script>

<style scoped lang="css">
.agreement-list {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-18);
}

.agreement-list__item {
  display: flex;
  gap: var(--fi-space-10);
}

.agreement-list__index {
  flex-shrink: 0;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-relaxed);
}

.agreement-list__text {
  flex: 1;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-relaxed);
}

.agreement-accept {
  width: 100%;
  height: 88rpx;
  line-height: 88rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-bold);
}
</style>

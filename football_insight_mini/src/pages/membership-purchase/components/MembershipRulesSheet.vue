<template>
  <FiBottomSheet
    :visible="visible"
    title="会员等级与升级规则"
    height="76vh"
    @update:visible="emit('update:visible', $event)"
  >
    <text class="rules-summary">{{ summary }}</text>

    <view class="rules-table">
      <view class="rules-table__head">
        <text class="rules-table__col rules-table__col--tier">等级</text>
        <text class="rules-table__col rules-table__col--condition">升级条件</text>
        <text class="rules-table__col rules-table__col--refresh">刷新</text>
      </view>
      <view
        v-for="guide in guides"
        :key="guide.code"
        class="rules-table__row"
        :class="{ 'rules-table__row--selected': guide.code === selectedTier }"
      >
        <view class="rules-table__col rules-table__col--tier">
          <text class="rules-table__code">{{ guide.code }}</text>
          <text class="rules-table__name">{{ guide.name }}</text>
        </view>
        <text class="rules-table__col rules-table__col--condition rules-table__text">{{ stripConditionPrefix(guide.condition) }}</text>
        <text class="rules-table__col rules-table__col--refresh rules-table__text">{{ guide.refreshLabel }}</text>
      </view>
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import type { MembershipTierGuide } from '../../../utils/membershipRules'

defineProps<{
  visible: boolean
  summary: string
  guides: MembershipTierGuide[]
  selectedTier: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

// 表头已标明「升级条件」，行内去掉重复前缀。
function stripConditionPrefix(condition: string): string {
  return condition.replace(/^升级条件：/, '')
}
</script>

<style scoped lang="css">
.rules-summary {
  display: block;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-relaxed);
}

.rules-table {
  margin-top: var(--fi-space-20);
  border-radius: var(--fi-radius-md);
  border: var(--fi-border-divider);
  overflow: hidden;
}

.rules-table__head,
.rules-table__row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  padding: var(--fi-space-18) var(--fi-space-20);
}

.rules-table__head {
  background: var(--fi-color-page);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.rules-table__row {
  border-top: var(--fi-border-divider);
}

.rules-table__row--selected {
  background: rgba(var(--fi-primitive-red-rgb), 0.04);
}

.rules-table__col--tier {
  width: 180rpx;
  flex-shrink: 0;
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-8);
}

.rules-table__col--condition {
  flex: 1;
  min-width: 0;
}

.rules-table__col--refresh {
  width: 140rpx;
  flex-shrink: 0;
  text-align: right;
}

.rules-table__code {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.rules-table__row--selected .rules-table__code {
  color: var(--fi-color-primary);
}

.rules-table__name {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.rules-table__text {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-snug);
}
</style>

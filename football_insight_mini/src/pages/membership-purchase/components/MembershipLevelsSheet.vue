<template>
  <FiBottomSheet
    :visible="visible"
    title="会员等级说明"
    height="76vh"
    @update:visible="emit('update:visible', $event)"
  >
    <text class="levels-summary">等级越高，余票监控刷新越快，最近回流速览能看的时间范围越长。</text>

    <view class="levels-table">
      <view class="levels-table__head">
        <text class="levels-table__col levels-table__col--tier">等级</text>
        <text class="levels-table__col levels-table__col--ways">获得方式</text>
        <text class="levels-table__col levels-table__col--refresh">刷新</text>
      </view>
      <view
        v-for="row in rows"
        :key="row.code"
        class="levels-table__row"
        :class="{ 'levels-table__row--selected': row.code === selectedTier }"
      >
        <view class="levels-table__col levels-table__col--tier">
          <text class="levels-table__code">{{ row.code }}</text>
          <text class="levels-table__name">{{ row.name }}</text>
        </view>
        <text class="levels-table__col levels-table__col--ways levels-table__text">{{ row.ways }}</text>
        <text class="levels-table__col levels-table__col--refresh levels-table__text">{{ row.refreshLabel }}</text>
      </view>
    </view>

    <view class="levels-invite-link" @tap="emit('open-invite')">
      <text class="levels-invite-link__text">「邀请 N 人」怎么算？查看邀请规则</text>
      <text class="levels-invite-link__arrow">›</text>
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import type { MembershipLevelRow } from '../../../utils/membershipRules'

defineProps<{
  visible: boolean
  rows: MembershipLevelRow[]
  selectedTier: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'open-invite'): void
}>()
</script>

<style scoped lang="css">
.levels-summary {
  display: block;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-relaxed);
}

.levels-table {
  margin-top: var(--fi-space-20);
  border-radius: var(--fi-radius-md);
  border: var(--fi-border-divider);
  overflow: hidden;
}

.levels-table__head,
.levels-table__row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  padding: var(--fi-space-18) var(--fi-space-20);
}

.levels-table__head {
  background: var(--fi-color-page);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.levels-table__row {
  border-top: var(--fi-border-divider);
}

.levels-table__row--selected {
  background: rgba(var(--fi-primitive-red-rgb), 0.04);
}

.levels-table__col--tier {
  width: 200rpx;
  flex-shrink: 0;
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-8);
}

.levels-table__col--ways {
  flex: 1;
  min-width: 0;
}

.levels-table__col--refresh {
  width: 140rpx;
  flex-shrink: 0;
  text-align: right;
}

.levels-table__code {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.levels-table__row--selected .levels-table__code {
  color: var(--fi-color-primary);
}

.levels-table__name {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.levels-table__text {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-snug);
}

.levels-invite-link {
  margin-top: var(--fi-space-20);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fi-space-18) var(--fi-space-20);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
}

.levels-invite-link__text {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-24);
}

.levels-invite-link__arrow {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-30);
  line-height: var(--fi-leading-none);
}
</style>

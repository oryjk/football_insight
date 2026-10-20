<template>
  <FiBottomSheet
    :visible="visible"
    eyebrow="选择球员"
    :title="slotLabel"
    height="80vh"
    @update:visible="emit('update:visible', $event)"
  >
    <view v-if="occupied" class="remove" hover-class="remove--pressed" @tap="emit('remove')">
      <text>移出这个位置</text>
    </view>

    <!-- 现役 / 前队员用胶囊切换（共享 FiSegmented），不再把前队员折叠在列表最下面 -->
    <view v-if="tabs.length" class="tabs">
      <FiSegmented :items="tabs" :model-value="activeGroup?.key ?? 'active'" aria-label="球员分组" @update:model-value="tab = $event" />
    </view>

    <view
      v-for="item in activeGroup?.players ?? []"
      :key="item.player_id"
      class="row"
      :class="{ 'row--on-pitch': item.onPitch }"
      hover-class="row--pressed"
      @tap="emit('pick', item.player_id)"
    >
      <image v-if="item.avatar_url" class="row__avatar" :src="item.avatar_url" mode="aspectFill" lazy-load />
      <view v-else class="row__avatar row__avatar--fallback">
        <text>{{ item.shirt_number ?? item.name.slice(0, 1) }}</text>
      </view>
      <view class="row__body">
        <text class="row__name">{{ item.name }}</text>
        <text class="row__meta">
          {{ item.shirt_number !== null ? `${item.shirt_number} 号` : '无号码' }}
          <template v-if="item.position"> · {{ POSITION_NAMES[item.position] }}</template>
          <template v-if="item.former"> · {{ item.last_season }} 赛季在队</template>
        </text>
      </view>
      <text v-if="item.onPitch" class="row__tag">已上场</text>
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import FiSegmented from '../../../components/FiSegmented.vue'
import type { LineupPosition } from '../../../types/lineup'
import { buildSheetTabs, resolveSheetTab, type SheetGroup } from '../lineupViewModel'

const props = defineProps<{
  visible: boolean
  slotLabel: string
  occupied: boolean
  groups: SheetGroup[]
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'pick', playerId: number): void
  (e: 'remove'): void
}>()

const POSITION_NAMES: Record<LineupPosition, string> = { GK: '门将', DF: '后卫', MF: '中场', FW: '前锋' }
// 当前选中的分组；每次重新打开弹层都回到"现役"，避免停在上次的前队员页造成困惑。
const tab = ref('active')
const tabs = computed(() => buildSheetTabs(props.groups))
const activeGroup = computed(() => resolveSheetTab(props.groups, tab.value))
watch(() => props.visible, (open) => {
  if (open) tab.value = 'active'
})
</script>

<style scoped>
.remove {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: var(--fi-space-16);
  padding: var(--fi-space-18);
  border: var(--fi-border-width) solid rgba(var(--fi-primitive-red-rgb), 0.3);
  border-radius: var(--fi-radius-md);
  background: rgba(var(--fi-primitive-red-rgb), 0.06);
  color: var(--fi-color-primary-deep);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.remove--pressed {
  opacity: var(--fi-primitive-opacity-pressed);
}

.tabs {
  margin-bottom: var(--fi-space-12);
}

.row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-18);
  padding: var(--fi-space-14) var(--fi-space-4);
  border-top: var(--fi-border-width) solid var(--fi-color-divider-soft);
}

.row--pressed {
  opacity: var(--fi-primitive-opacity-pressed);
}

.row--on-pitch {
  opacity: 0.5;
}

.row__avatar {
  flex-shrink: 0;
  width: var(--fi-space-64);
  height: var(--fi-space-64);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
}

.row__avatar--fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.row__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-4);
}

.row__name {
  overflow: hidden;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row__meta {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.row__tag {
  flex-shrink: 0;
  padding: var(--fi-space-4) var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
}
</style>

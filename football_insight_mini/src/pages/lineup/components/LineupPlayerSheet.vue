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

    <view v-for="group in groups" :key="group.key" class="group">
      <view
        v-if="group.key === 'former'"
        class="group__head group__head--toggle"
        hover-class="group__head--pressed"
        @tap="formerOpen = !formerOpen"
      >
        <text class="group__title">{{ group.title }}</text>
        <text class="group__count">{{ group.players.length }} 人</text>
        <image
          class="group__chevron"
          :class="{ 'group__chevron--open': formerOpen }"
          src="/static/seat-swap/chevron-down.svg"
          mode="aspectFit"
        />
      </view>
      <view v-else class="group__head">
        <text class="group__title">{{ group.title }}</text>
        <text class="group__count">{{ group.players.length }} 人</text>
      </view>

      <template v-if="group.key === 'active' || formerOpen">
        <view
          v-for="item in group.players"
          :key="item.player_id"
          class="row"
          :class="{ 'row--on-pitch': item.onPitch }"
          hover-class="row--pressed"
          @tap="emit('pick', item.player_id)"
        >
          <image v-if="item.avatar_url" class="row__avatar" :src="item.avatar_url" mode="aspectFill" />
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
      </template>
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import type { LineupPosition } from '../../../types/lineup'
import type { SheetGroup } from '../lineupViewModel'

defineProps<{
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
// 前队员默认折叠，点标题展开。
const formerOpen = ref(false)
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

.group__head {
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-12);
  padding: var(--fi-space-18) var(--fi-space-4) var(--fi-space-10);
}

.group__head--toggle {
  align-items: center;
}

.group__head--pressed {
  opacity: var(--fi-primitive-opacity-pressed);
}

.group__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.group__count {
  flex: 1;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.group__chevron {
  width: var(--fi-space-32);
  height: var(--fi-space-32);
  transition: transform var(--fi-motion-duration-base) var(--fi-motion-ease-out);
}

.group__chevron--open {
  transform: rotate(180deg);
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

@media (prefers-reduced-motion: reduce) {
  .group__chevron {
    transition: none;
  }
}
</style>

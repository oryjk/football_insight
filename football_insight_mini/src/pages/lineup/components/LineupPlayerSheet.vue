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

    <!-- 搜索框常驻：输入后列表换成搜索结果（本队在前、其他球队在后），清空或点「取消」回到现役 / 前队员。 -->
    <view class="search">
      <view class="search__icon" aria-hidden="true" />
      <input class="search__input" :value="query" placeholder="搜索全部中超球员（含其他球队）" confirm-type="search" maxlength="20" @input="onInput" />
      <text v-if="searching" class="search__cancel" role="button" @tap="clearQuery">取消</text>
    </view>

    <template v-if="searching">
      <text class="search__summary">{{ searchSummary }}</text>
      <template v-for="group in searchGroups" :key="group.key">
        <text class="group-title">{{ group.title }}</text>
        <view
          v-for="item in group.players"
          :key="item.player_id"
          class="row"
          :class="{ 'row--on-pitch': item.onPitch }"
          hover-class="row--pressed"
          @tap="emit('pick', item.player_id)"
        >
          <view v-if="item.avatar_url" class="row__avatar"><FiAvatar :src="item.avatar_url" :size="96" :name="item.name" lazy /></view>
          <view v-else class="row__avatar row__avatar--fallback"><text>{{ item.shirt_number ?? item.name.slice(0, 1) }}</text></view>
          <view class="row__body">
            <text class="row__name">{{ item.name }}</text>
            <text class="row__meta">
              <template v-if="item.club">{{ item.club }} · </template>{{ item.shirt_number !== null ? `${item.shirt_number} 号` : '无号码' }}
              <template v-if="item.position"> · {{ POSITION_NAMES[item.position] }}</template>
              <template v-if="item.former"> · {{ item.last_season }} 赛季在队</template>
            </text>
          </view>
          <text v-if="item.onPitch" class="row__tag">已上场</text>
          <text v-else-if="item.club" class="row__tag row__tag--club">其他球队</text>
        </view>
      </template>
    </template>

    <!-- 现役 / 前队员用胶囊切换（共享 FiSegmented），不再把前队员折叠在列表最下面 -->
    <view v-if="!searching && tabs.length" class="tabs">
      <FiSegmented :items="tabs" :model-value="activeGroup?.key ?? 'active'" aria-label="球员分组" @update:model-value="tab = $event" />
    </view>

    <view
      v-for="item in searching ? [] : activeGroup?.players ?? []"
      :key="item.player_id"
      class="row"
      :class="{ 'row--on-pitch': item.onPitch }"
      hover-class="row--pressed"
      @tap="emit('pick', item.player_id)"
    >
      <view v-if="item.avatar_url" class="row__avatar"><FiAvatar :src="item.avatar_url" :size="96" :name="item.name" lazy /></view>
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
import FiAvatar from '../../../components/FiAvatar.vue'
import FiSegmented from '../../../components/FiSegmented.vue'
import type { LineupPosition } from '../../../types/lineup'
import { buildSheetTabs, resolveSheetTab, type SheetGroup } from '../lineupViewModel'

const props = defineProps<{
  visible: boolean
  slotLabel: string
  occupied: boolean
  groups: SheetGroup[]
  /** 搜索结果分组（本队 / 其他球队），由页面按 `search` 事件查询后传入。 */
  searchGroups: SheetGroup[]
  searchLoading: boolean
  searchError: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'pick', playerId: number): void
  (e: 'remove'): void
  (e: 'search', query: string): void
}>()

const query = ref('')
const searching = computed(() => query.value.trim().length > 0)
const searchSummary = computed(() => {
  if (props.searchError) return props.searchError
  const total = props.searchGroups.reduce((sum, group) => sum + group.players.length, 0)
  if (props.searchLoading && !total) return '正在搜索…'
  return total ? `找到 ${total} 名球员 · 本队在前，同位置优先` : `没有找到「${query.value.trim()}」，换个写法试试`
})
function onInput(event: Event) {
  // 小程序 input 事件的值在 detail.value。
  query.value = (event as unknown as { detail: { value: string } }).detail.value
  emit('search', query.value)
}
function clearQuery() {
  query.value = ''
  emit('search', '')
}

const POSITION_NAMES: Record<LineupPosition, string> = { GK: '门将', DF: '后卫', MF: '中场', FW: '前锋' }
// 当前选中的分组；每次重新打开弹层都回到"现役"，避免停在上次的前队员页造成困惑。
const tab = ref('active')
const tabs = computed(() => buildSheetTabs(props.groups))
const activeGroup = computed(() => resolveSheetTab(props.groups, tab.value))
watch(() => props.visible, (open) => {
  if (!open) return
  tab.value = 'active'
  if (query.value) clearQuery()
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

.search {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  height: 88rpx;
  margin-bottom: var(--fi-space-16);
  padding: 0 var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
}

.search__icon {
  position: relative;
  flex-shrink: 0;
  width: 24rpx;
  height: 24rpx;
  border: 4rpx solid var(--fi-color-text-muted);
  border-radius: var(--fi-radius-round);
}

.search__icon::after {
  content: '';
  position: absolute;
  right: -10rpx;
  bottom: -8rpx;
  width: 4rpx;
  height: 12rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-muted);
  transform: rotate(-45deg);
}

.search__input {
  flex: 1;
  min-width: 0;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
}

.search__cancel {
  flex-shrink: 0;
  padding: var(--fi-space-12) 0 var(--fi-space-12) var(--fi-space-12);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.search__summary {
  display: block;
  margin-bottom: var(--fi-space-8);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.group-title {
  display: block;
  padding: var(--fi-space-16) var(--fi-space-4) var(--fi-space-8);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
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
  overflow: hidden;
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
.row__tag--club {
  background: var(--fi-color-outsider-soft);
  color: var(--fi-color-outsider);
}
</style>

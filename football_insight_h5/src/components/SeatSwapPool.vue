<template>
  <div class="pool">
    <div class="pool__toolbar">
      <div class="pool__search">
        <input
          v-model="keyword"
          class="pool__search-input"
          type="text"
          placeholder="搜索昵称或座位，如 127 / 8排"
        />
      </div>
      <span class="pool__total">共 {{ totalCount }} 条</span>
    </div>

    <div v-if="filterKey" class="pool__filter">
      <span class="pool__filter-text">
        想换到 {{ filterName }} 的球迷 · {{ filteredCandidates.length }} 条
      </span>
      <button type="button" class="pool__filter-clear" @click="emit('clear-filter')">✕ 清除筛选</button>
    </div>

    <template v-if="visibleGroups.length">
      <div v-for="group in visibleGroups" :key="group.region_key" class="pool__group">
        <div class="pool__group-row" @click="toggleGroup(group.region_key)">
          <div class="pool__group-main">
            <span class="pool__group-name">{{ group.region_name }}</span>
            <span class="pool__group-count">{{ group.requests.length }} 条发布</span>
            <span v-if="myDesiredKeys.includes(group.region_key)" class="pool__group-hit">命中我的目标</span>
          </div>
          <span class="pool__group-caret">{{ collapsedKeys.includes(group.region_key) ? '展开' : '收起' }}</span>
        </div>
        <template v-if="!collapsedKeys.includes(group.region_key)">
          <SeatSwapCandidateCard
            v-for="candidate in group.requests"
            :key="candidate.request_id"
            :candidate="candidate"
            :action="resolveAction(candidate)"
            @confirm="(id: string) => emit('confirm', id)"
            @cancel-confirmation="(id: string) => emit('cancel-confirmation', id)"
            @matched-cancel="(id: string) => emit('matched-cancel', id)"
          />
        </template>
      </div>
    </template>

    <div v-else class="pool__empty">
      <span>{{ emptyText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import SeatSwapCandidateCard from './SeatSwapCandidateCard.vue'
import type { SeatSwapCandidate } from '../types/seatSwap'
import {
  resolveSeatSwapCandidateAction,
  type SeatSwapRegionGroup,
} from '../utils/seatSwap'

const props = defineProps<{
  groups: SeatSwapRegionGroup<SeatSwapCandidate>[]
  filterKey: string
  filterName: string
  filteredCandidates: SeatSwapCandidate[]
  totalCount: number
  myDesiredKeys: string[]
  isLoggedIn: boolean
  myRequestId: string | null
}>()

const emit = defineEmits<{
  (e: 'clear-filter'): void
  (e: 'confirm', requestId: string): void
  (e: 'cancel-confirmation', requestId: string): void
  (e: 'matched-cancel', requestId: string): void
}>()

const keyword = ref('')
const collapsedKeys = ref<string[]>([])

const filteredGroups = computed(() => {
  // 地图筛选态下展示“想换到该分区”的扁平列表；否则按当前座位分区分组。
  if (props.filterKey) {
    return props.filteredCandidates.length
      ? [{ region_key: '__filtered__', region_name: props.filterName, requests: props.filteredCandidates }]
      : []
  }
  return props.groups
})

const visibleGroups = computed(() => {
  const query = keyword.value.trim().toLowerCase()
  if (!query) {
    return filteredGroups.value
  }

  return filteredGroups.value
    .map((group) => ({
      ...group,
      requests: group.requests.filter((candidate) => {
        const seat = `${candidate.current_region_name} ${candidate.current_row}排 ${candidate.current_seat_no}号`
        const desired = candidate.desired_seats.map((seatItem) => seatItem.region_name).join(' ')
        return `${candidate.display_name} ${seat} ${desired}`.toLowerCase().includes(query)
      }),
    }))
    .filter((group) => group.requests.length > 0)
})

const emptyText = computed(() => {
  if (keyword.value.trim()) {
    return `没有匹配「${keyword.value.trim()}」的换座意向`
  }
  if (props.filterKey) {
    return `暂时没有想换到 ${props.filterName} 的球迷`
  }
  return '换座池暂时是空的，来发布第一条换座意向吧'
})

function toggleGroup(key: string): void {
  if (collapsedKeys.value.includes(key)) {
    collapsedKeys.value = collapsedKeys.value.filter((item) => item !== key)
  } else {
    collapsedKeys.value = [...collapsedKeys.value, key]
  }
}

function resolveAction(candidate: SeatSwapCandidate) {
  return resolveSeatSwapCandidateAction({
    candidateStatus: candidate.status,
    candidateRequestId: candidate.request_id,
    myRequestId: props.myRequestId,
    isLoggedIn: props.isLoggedIn,
  })
}
</script>

<style scoped>
.pool__toolbar {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 10px;
}

.pool__search {
  flex: 1;
  min-width: 0;
}

.pool__search-input {
  width: 100%;
  height: 38px;
  padding: 0 14px;
  border-radius: 999px;
  border: 1px solid rgba(232, 233, 238, 0.95);
  background: rgba(255, 255, 255, 0.96);
  color: #17181c;
  font-size: 13px;
  box-shadow: 0 4px 9px rgba(26, 28, 36, 0.04);
  box-sizing: border-box;
  outline: none;
}

.pool__search-input::placeholder {
  color: #988f84;
}

.pool__total {
  flex-shrink: 0;
  color: #988f84;
  font-size: 12px;
  white-space: nowrap;
}

.pool__filter {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 4px;
  margin-bottom: 4px;
  border-bottom: 1px solid rgba(220, 211, 192, 0.5);
}

.pool__filter-text {
  color: #17181c;
  font-size: 14px;
  font-weight: 700;
}

.pool__filter-clear {
  flex-shrink: 0;
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid rgba(230, 220, 198, 0.92);
  background: linear-gradient(180deg, rgba(255, 251, 242, 0.98), rgba(248, 241, 227, 0.94));
  color: #9c855c;
  font-size: 11px;
  white-space: nowrap;
  cursor: pointer;
}

.pool__group {
  border-top: 1px solid rgba(220, 211, 192, 0.5);
  padding: 4px 0 8px;
}

.pool__group:first-of-type {
  border-top: 0;
}

.pool__group-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 4px;
  cursor: pointer;
  user-select: none;
}

.pool__group-row:hover {
  opacity: 0.72;
}

.pool__group-main {
  display: flex;
  align-items: center;
  gap: 7px;
  min-width: 0;
}

.pool__group-name {
  color: #17181c;
  font-size: 15px;
  font-weight: 700;
}

.pool__group-count {
  color: #8f7c5f;
  font-size: 12px;
}

.pool__group-hit {
  padding: 2px 7px;
  border-radius: 999px;
  background: linear-gradient(180deg, #eaf8ef, #dff1e6);
  border: 1px solid rgba(29, 138, 85, 0.3);
  color: #167348;
  font-size: 10px;
  white-space: nowrap;
}

.pool__group-caret {
  flex-shrink: 0;
  padding: 3px 9px;
  border-radius: 999px;
  border: 1px solid rgba(230, 220, 198, 0.92);
  background: linear-gradient(180deg, rgba(255, 251, 242, 0.98), rgba(248, 241, 227, 0.94));
  color: #8f7c5f;
  font-size: 10px;
}

.pool__empty {
  padding: 30px 12px;
  border-radius: 10px;
  border: 1px dashed rgba(207, 211, 220, 0.95);
  background: rgba(255, 255, 255, 0.72);
  color: #988f84;
  font-size: 13px;
  text-align: center;
}
</style>

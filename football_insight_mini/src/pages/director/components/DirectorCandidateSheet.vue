<template>
  <FiBottomSheet
    :visible="visible"
    :title="mode === 'foreign' ? '挑选新签外援' : '挑选国内重点补强'"
    :eyebrow="mode === 'foreign' ? '支持按联赛筛选或自由提名' : '中超现役或自由提名（最多 3 人）'"
    height="84vh"
    :z-index="60"
    @update:visible="emit('update:visible', $event)"
  >
    <view class="sheet-content">
      <scroll-view scroll-x class="league-tabs">
        <view class="league-tabs__list">
          <button
            v-for="tab in tabs"
            :key="tab.value"
            class="league-tab"
            :class="{ 'league-tab--active': leagueFilter === tab.value }"
            hover-class="sheet-pressed"
            @tap="emit('update:leagueFilter', tab.value)"
          >
            {{ tab.label }}
          </button>
        </view>
      </scroll-view>

      <view v-if="leagueFilter === 'CUSTOM'" class="custom-form">
        <text class="custom-form__hint">候选池没有你想签的球员？直接自由提名，提交后自动计入风格标签</text>

        <view class="custom-field">
          <text class="custom-field__label">球员姓名 *</text>
          <input
            v-model="customName"
            class="custom-field__input"
            maxlength="20"
            placeholder="例如：金某某 / 某某西亚"
          />
        </view>

        <view class="custom-field">
          <text class="custom-field__label">场上位置 *</text>
          <view class="pos-row">
            <button
              v-for="pos in DIRECTOR_POSITIONS"
              :key="pos.value"
              class="pos-chip"
              :class="{ 'pos-chip--active': customPosition === pos.value }"
              hover-class="sheet-pressed"
              @tap="customPosition = pos.value"
            >
              {{ pos.label }}
            </button>
          </view>
        </view>

        <view class="custom-field">
          <text class="custom-field__label">所在联赛</text>
          <view class="pos-row">
            <button
              v-for="tag in customLeaguePresets"
              :key="tag"
              class="pos-chip"
              :class="{ 'pos-chip--active': customLeagueTag === tag }"
              hover-class="sheet-pressed"
              @tap="customLeagueTag = tag"
            >
              {{ directorLeagueLabel(tag) }}
            </button>
          </view>
        </view>

        <view class="custom-field">
          <text class="custom-field__label">当前俱乐部（选填）</text>
          <input
            v-model="customClub"
            class="custom-field__input"
            maxlength="24"
            placeholder="例如：蔚山HD / 自由身"
          />
        </view>

        <button class="custom-submit" hover-class="sheet-pressed" @tap="handleCustomSubmit">
          确认加入引援名单
        </button>
      </view>

      <template v-else>
        <view class="search-row">
          <input
            :value="searchQuery"
            class="search-input"
            confirm-type="search"
            placeholder="搜索球员姓名或俱乐部"
            @input="onSearchInput"
            @confirm="emit('search')"
          />
          <button class="search-btn" hover-class="sheet-pressed" @tap="emit('search')">搜索</button>
        </view>

        <view v-if="loading" class="state-box">
          <text class="state-text">正在加载候选球员…</text>
        </view>

        <view v-else-if="error" class="state-box">
          <text class="state-text state-text--error">{{ error }}</text>
          <button class="retry-btn" hover-class="sheet-pressed" @tap="emit('search')">重试</button>
        </view>

        <view v-else-if="!candidates.length" class="state-box">
          <text class="state-text">暂无匹配候选人，可切换到「自由提名」手动添加</text>
          <button class="retry-btn" hover-class="sheet-pressed" @tap="emit('update:leagueFilter', 'CUSTOM')">
            去自由提名
          </button>
        </view>

        <view v-else class="candidate-list">
          <view
            v-for="item in candidates"
            :key="item.key"
            class="candidate-row"
            :class="{ 'candidate-row--selected': selectedKeys.has(item.key) }"
          >
            <view
              class="candidate-row__avatar"
              :hover-class="item.avatar_url ? 'sheet-pressed' : 'none'"
              @tap="previewAvatar(item)"
            >
              <FiAvatar :src="item.avatar_url" :size="96" :name="item.player_name" />
            </view>
            <view class="candidate-row__body">
              <view class="candidate-row__top">
                <text class="candidate-row__name">{{ item.player_name }}</text>
                <text class="candidate-row__league">{{ directorLeagueLabel(item.league_tag) }}</text>
                <text class="candidate-row__pos">{{ directorPositionLabel(item.position) }}</text>
              </view>
              <text class="candidate-row__meta">{{ item.current_club }} · {{ item.country }}</text>
              <view v-if="item.tags.length" class="candidate-row__tags">
                <text v-for="tag in item.tags" :key="tag" class="candidate-tag">{{ tag }}</text>
              </view>
            </view>
            <button
              class="candidate-row__action"
              :class="{ 'candidate-row__action--picked': selectedKeys.has(item.key) }"
              hover-class="sheet-pressed"
              @tap="emit('pick', item)"
            >
              {{ selectedKeys.has(item.key) ? '已选' : '签下' }}
            </button>
          </view>
        </view>
      </template>
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import FiAvatar from '../../../components/FiAvatar.vue'
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import type {
  DirectorCandidateDto,
  DirectorLeagueFilter,
  DirectorPosition,
} from '../../../types/director'
import { directorLeagueLabel, directorPositionLabel } from '../../../utils/directorLinks'
import {
  DIRECTOR_DOMESTIC_LEAGUE_TABS,
  DIRECTOR_FOREIGN_LEAGUE_TABS,
  DIRECTOR_POSITIONS,
} from '../helpers'

const props = defineProps<{
  visible: boolean
  mode: 'foreign' | 'domestic'
  leagueFilter: DirectorLeagueFilter
  searchQuery: string
  candidates: DirectorCandidateDto[]
  loading: boolean
  error: string
  selectedKeys: Set<string>
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'update:leagueFilter', value: DirectorLeagueFilter): void
  (e: 'update:searchQuery', value: string): void
  (e: 'search'): void
  (e: 'pick', candidate: DirectorCandidateDto): void
  (e: 'add-custom', payload: { playerName: string; position: DirectorPosition; leagueTag: string; currentClub: string }): void
}>()

const tabs = computed(() =>
  props.mode === 'foreign' ? DIRECTOR_FOREIGN_LEAGUE_TABS : DIRECTOR_DOMESTIC_LEAGUE_TABS,
)

const customLeaguePresets = computed(() =>
  props.mode === 'foreign' ? ['K1', 'K2', 'J1', 'EUR', 'ACL', 'SPL', 'MLS', 'CSL', 'FREE', 'OTHER'] : ['CSL', 'CL1', 'FREE'],
)

const customName = ref('')
const customPosition = ref<DirectorPosition>('FW')
const customLeagueTag = ref('K1')
const customClub = ref('')

watch(
  () => [props.visible, props.mode],
  () => {
    customName.value = ''
    customClub.value = ''
    customPosition.value = 'FW'
    customLeagueTag.value = props.mode === 'foreign' ? 'K1' : 'CSL'
  },
)

function onSearchInput(event: unknown): void {
  const value = (event as { detail?: { value?: string } } | null)?.detail?.value
  emit('update:searchQuery', typeof value === 'string' ? value : '')
}

function previewAvatar(item: DirectorCandidateDto): void {
  const url = item.avatar_url?.trim()
  if (!url) return
  uni.previewImage({
    urls: [url],
    current: url,
  })
}

function handleCustomSubmit(): void {
  emit('add-custom', {
    playerName: customName.value,
    position: customPosition.value,
    leagueTag: customLeagueTag.value,
    currentClub: customClub.value,
  })
}
</script>

<style scoped lang="css">
.sheet-content {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
  padding-bottom: var(--fi-space-24);
}

.league-tabs {
  width: 100%;
  white-space: nowrap;
}

.league-tabs__list {
  display: inline-flex;
  gap: var(--fi-space-10);
}

.league-tab {
  flex-shrink: 0;
  margin: 0;
  padding: var(--fi-space-10) var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-secondary);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.league-tab::after {
  border: 0;
}

.league-tab--active {
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
  border-color: var(--fi-color-text-primary);
}

.search-row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
}

.search-input {
  flex: 1;
  min-width: 0;
  height: var(--fi-space-68);
  padding: 0 var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
}

.search-btn {
  flex-shrink: 0;
  margin: 0;
  padding: var(--fi-space-16) var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.search-btn::after {
  border: 0;
}

.candidate-list {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
}

.candidate-row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-14);
  padding: var(--fi-space-16);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-width) solid var(--fi-color-divider-soft);
}

.candidate-row--selected {
  background: var(--fi-color-primary-soft);
  border-color: var(--fi-color-primary);
}

.candidate-row__avatar {
  flex-shrink: 0;
  width: var(--fi-space-64);
  height: var(--fi-space-64);
  border-radius: var(--fi-radius-round);
  overflow: hidden;
  background: var(--fi-color-surface);
  font-size: var(--fi-font-24);
}

.candidate-row__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.candidate-row__top {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--fi-space-8);
}

.candidate-row__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
}

.candidate-row__league {
  padding: var(--fi-space-4) var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-outsider-soft);
  color: var(--fi-color-outsider);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.candidate-row__pos {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-20);
}

.candidate-row__meta {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.candidate-row__tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fi-space-6);
}

.candidate-tag {
  padding: var(--fi-space-4) var(--fi-space-8);
  border-radius: var(--fi-radius-xs);
  background: var(--fi-color-surface);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-none);
}

.candidate-row__action {
  flex-shrink: 0;
  margin: 0;
  padding: var(--fi-space-10) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  color: var(--fi-color-primary);
  border: var(--fi-border-width) solid var(--fi-color-primary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.candidate-row__action::after {
  border: 0;
}

.candidate-row__action--picked {
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
}

.state-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-16);
  padding: var(--fi-space-40) var(--fi-space-20);
}

.state-text {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  text-align: center;
  line-height: var(--fi-leading-normal);
}

.state-text--error {
  color: var(--fi-color-primary-deep);
}

.retry-btn {
  margin: 0;
  padding: var(--fi-space-12) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.retry-btn::after {
  border: 0;
}

.custom-form {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-18);
}

.custom-form__hint {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.custom-field {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-10);
}

.custom-field__label {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.custom-field__input {
  height: var(--fi-space-72);
  padding: 0 var(--fi-space-18);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
}

.pos-row {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fi-space-10);
}

.pos-chip {
  margin: 0;
  padding: var(--fi-space-10) var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-secondary);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.pos-chip::after {
  border: 0;
}

.pos-chip--active {
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  border-color: var(--fi-color-primary);
}

.custom-submit {
  margin: 0;
  padding: var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.custom-submit::after {
  border: 0;
}

.sheet-pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>

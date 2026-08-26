<template>
  <view class="panel ranking-surface">
    <view class="rankings-controls">
      <view class="scope-toggle">
        <view
          class="scope-toggle__button"
          :class="{ active: scope === 'team' }"
          @click="emit('update:scope', 'team')"
        >
          <text class="scope-toggle__button-text">球队榜</text>
        </view>
        <view
          class="scope-toggle__button"
          :class="{ active: scope === 'player' }"
          @click="emit('update:scope', 'player')"
        >
          <text class="scope-toggle__button-text">球员榜</text>
        </view>
      </view>

      <view class="category-tabs-wrap">
        <scroll-view scroll-x class="pill-row" :scroll-left="categoryScrollLeft" scroll-with-animation>
          <view class="pill-row__list">
            <view
              v-for="item in categoryOptions"
              :id="`ranking-category-${item.slug}`"
              :key="item.slug"
              class="pill-row__item"
              :class="{ active: item.slug === activeCategorySlug }"
              @click="emit('update:activeCategorySlug', item.slug)"
            >
              <text class="pill-row__item-text">{{ item.label }}</text>
            </view>
          </view>
        </scroll-view>
      </view>
    </view>

    <view v-if="scope === 'team' && activeTeamCategory" class="ranking-list">
      <view class="section-heading">
        <view>
          <text class="section-kicker">{{ teamKicker }}</text>
          <text class="section-title">{{ activeTeamCategory.label }}</text>
        </view>
        <view v-if="isStandingsTeamCategory" class="standings-mode-toggle">
          <view
            class="standings-mode-toggle__item"
            :class="{ active: standingsRankingMode === 'with_penalty' }"
            @click.stop="emit('update:standingsRankingMode', 'with_penalty')"
          >
            <text>含罚分</text>
          </view>
          <view
            class="standings-mode-toggle__item"
            :class="{ active: standingsRankingMode === 'without_penalty' }"
            @click.stop="emit('update:standingsRankingMode', 'without_penalty')"
          >
            <text>无罚分</text>
          </view>
        </view>
      </view>

      <view
        v-for="entry in teamEntries"
        :key="`${activeTeamCategory.slug}-${standingsRankingMode}-${entry.team_id}`"
        class="ranking-row ranking-row--interactive"
        hover-class="ranking-row--pressed"
        hover-stay-time="100"
        @click="emit('open-team', entry)"
      >
        <view class="ranking-row__rank-wrap">
          <text class="ranking-row__rank" :class="`ranking-row__rank--${entry.rank_no}`">#{{ entry.rank_no }}</text>
        </view>
        <image :src="entry.avatar_storage_url || ''" mode="aspectFit" class="ranking-row__avatar" />
        <view class="ranking-row__body">
          <text class="ranking-row__name">{{ entry.team_name }}</text>
          <text class="ranking-row__note">点击查看球队信息和赛程</text>
        </view>
        <view class="ranking-row__metric">
          <text class="ranking-row__metric-value">{{ entry.score_value }}</text>
          <text class="ranking-row__metric-note">{{ teamMetricLabel }}</text>
        </view>
      </view>
    </view>

    <view v-if="scope === 'player' && activePlayerCategory" class="ranking-list">
      <view class="section-heading">
        <view>
          <text class="section-kicker">实时球员累计榜</text>
          <text class="section-title">{{ activePlayerCategory.label }}</text>
        </view>
      </view>

      <view
        v-for="entry in activePlayerCategory.entries"
        :key="`${activePlayerCategory.slug}-${entry.player_id}`"
        class="ranking-row"
      >
        <text class="ranking-row__rank" :class="`ranking-row__rank--${entry.rank_no}`">#{{ entry.rank_no }}</text>
        <image :src="entry.avatar_storage_url || ''" mode="aspectFill" class="ranking-row__avatar ranking-row__avatar--player" />
        <view class="ranking-row__body">
          <text class="ranking-row__name">{{ entry.player_name }}</text>
          <text class="ranking-row__note">{{ entry.team_name }}</text>
        </view>
        <view class="ranking-row__metric">
          <text class="ranking-row__metric-value">{{ entry.score_value }}</text>
          <text class="ranking-row__metric-note">{{ activePlayerCategory.label }}</text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, getCurrentInstance, nextTick, ref, watch } from 'vue'
import type {
  PlayerRankingCategory,
  RankingsViewResponse,
  StandingsTable,
  TeamRankingCategory,
  TeamRankingEntry,
} from '../../../types/insight'
import { buildStandingsRankingEntries, type StandingsRankingMode } from '../poster'

const props = defineProps<{
  rankings: RankingsViewResponse
  scope: 'team' | 'player'
  standingsRankingMode: StandingsRankingMode
  activeCategorySlug: string
}>()

const emit = defineEmits<{
  (e: 'update:scope', value: 'team' | 'player'): void
  (e: 'update:standingsRankingMode', value: StandingsRankingMode): void
  (e: 'update:activeCategorySlug', value: string): void
  (e: 'open-team', entry: TeamRankingEntry): void
}>()

const instance = getCurrentInstance()
const categoryScrollLeft = ref(0)

const teamCategories = computed<TeamRankingCategory[]>(() => props.rankings.team_categories ?? [])
const playerCategories = computed<PlayerRankingCategory[]>(() => props.rankings.player_categories ?? [])
const standingsTables = computed<StandingsTable[]>(() => props.rankings.standings_tables ?? [])
const categoryOptions = computed(() =>
  props.scope === 'team'
    ? teamCategories.value.map((item) => ({ slug: item.slug, label: item.label }))
    : playerCategories.value.map((item) => ({ slug: item.slug, label: item.label })),
)
const activeTeamCategory = computed(() =>
  teamCategories.value.find((item) => item.slug === props.activeCategorySlug) ?? null,
)
const activePlayerCategory = computed(() =>
  playerCategories.value.find((item) => item.slug === props.activeCategorySlug) ?? null,
)
const isStandingsTeamCategory = computed(() => activeTeamCategory.value?.slug === 'standings')
const teamEntries = computed<TeamRankingEntry[]>(() => {
  if (!activeTeamCategory.value) {
    return []
  }

  if (!isStandingsTeamCategory.value) {
    return activeTeamCategory.value.entries
  }

  return buildStandingsRankingEntries(standingsTables.value, props.standingsRankingMode)
})
const teamKicker = computed(() =>
  isStandingsTeamCategory.value ? '实时球队排名' : '实时球队累计榜',
)
const teamMetricLabel = computed(() => (isStandingsTeamCategory.value ? '积分' : '总计'))

watch(
  categoryOptions,
  (items) => {
    if (!items.length) {
      if (props.activeCategorySlug) {
        emit('update:activeCategorySlug', '')
      }
      return
    }

    if (!items.some((item) => item.slug === props.activeCategorySlug)) {
      emit('update:activeCategorySlug', items[0]?.slug ?? '')
    }
  },
  { immediate: true },
)

watch(() => props.activeCategorySlug, () => {
  void centerActiveCategory()
})

function hasRectShape(value: unknown): value is { left: number; width: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { left?: unknown }).left === 'number'
    && typeof (value as { width?: unknown }).width === 'number'
}

function hasScrollLeft(value: unknown): value is { scrollLeft: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { scrollLeft?: unknown }).scrollLeft === 'number'
}

async function centerActiveCategory(): Promise<void> {
  if (!instance || !props.activeCategorySlug) {
    return
  }

  await nextTick()

  const query = uni.createSelectorQuery().in(instance)
  query.select('.pill-row').boundingClientRect()
  query.select('.pill-row').scrollOffset(() => {})
  query.select(`#ranking-category-${props.activeCategorySlug}`).boundingClientRect()
  query.exec((result) => {
    const [rawScrollRect, rawScrollOffset, rawPillRect] = (result ?? []) as unknown[]

    if (!hasRectShape(rawScrollRect) || !hasScrollLeft(rawScrollOffset) || !hasRectShape(rawPillRect)) {
      return
    }

    const delta = (rawPillRect.left + rawPillRect.width / 2) - (rawScrollRect.left + rawScrollRect.width / 2)
    const nextScrollLeft = Math.max(0, Math.round(rawScrollOffset.scrollLeft + delta))

    if (nextScrollLeft !== categoryScrollLeft.value) {
      categoryScrollLeft.value = nextScrollLeft
    }
  })
}
</script>

<style scoped lang="css">
.panel {
  background: rgba(255, 255, 255, 0.94);
  border-radius: var(--fi-radius-xl);
  padding: var(--fi-space-20);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
}

.section-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.section-heading > view {
  display: grid;
  gap: var(--fi-space-8);
  min-width: 0;
}

.section-kicker {
  margin: 0;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 3rpx;
}

.section-title {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: 50rpx;
  line-height: 1.08;
  font-weight: var(--fi-weight-extrabold);
}

.rankings-controls {
  display: grid;
  gap: var(--fi-space-14);
  width: 100%;
  padding: 0 0 var(--fi-space-10);
  overflow: hidden;
}

.scope-toggle {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  width: 100%;
  gap: var(--fi-space-6);
  padding: var(--fi-space-6);
  border: var(--fi-primitive-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-round);
  background: rgba(246, 247, 251, 0.92);
}

.standings-mode-toggle {
  display: inline-grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 2rpx;
  flex: 0 0 auto;
  padding: 3rpx;
  border-radius: var(--fi-radius-round);
  background: rgba(242, 243, 247, 0.92);
}

.standings-mode-toggle__item {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 96rpx;
  padding: var(--fi-space-12) var(--fi-space-16);
  border-radius: var(--fi-radius-round);
  color: #7b818d;
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}

.standings-mode-toggle__item.active {
  color: var(--fi-color-text-strong);
  font-weight: var(--fi-weight-bold);
  background: var(--fi-primitive-white);
  box-shadow: 0 6rpx 14rpx rgba(18, 18, 18, 0.05);
}

.scope-toggle__button {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  min-width: 0;
  min-height: 58rpx;
  padding: 0 var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  text-align: center;
  background: transparent;
  color: #7f8490;
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  overflow: hidden;
  box-sizing: border-box;
}

.scope-toggle__button::after {
  border: 0;
}

.scope-toggle__button.active {
  color: var(--fi-primitive-white);
  font-weight: var(--fi-weight-black);
  background: var(--fi-primitive-ink);
  box-shadow: 0 8rpx 18rpx rgba(21, 22, 27, 0.12);
}

.scope-toggle__button-text {
  display: block;
  width: 100%;
  text-align: center;
}

.category-tabs-wrap {
  display: grid;
  overflow: hidden;
}

.pill-row {
  width: 100%;
  white-space: nowrap;
  overflow: hidden;
}

.pill-row__list {
  display: inline-flex;
  gap: var(--fi-space-10);
  min-width: max-content;
  padding: 0 0 4rpx;
}

.pill-row__item {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: fit-content;
  padding: var(--fi-space-18) var(--fi-space-22);
  text-align: center;
  background: rgba(239, 240, 244, 0.92);
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-24);
  color: #696f7a;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  box-sizing: border-box;
}

.pill-row__item.active {
  color: var(--fi-primitive-white);
  font-weight: var(--fi-weight-extrabold);
  background: var(--fi-primitive-ink);
}

.pill-row__item-text {
  display: block;
  text-align: center;
}

.ranking-surface {
  display: grid;
  gap: var(--fi-space-22);
  width: 100%;
  padding: var(--fi-space-20);
  overflow: hidden;
}

.ranking-list {
  display: grid;
  gap: 0;
  overflow: hidden;
}

.ranking-row {
  display: grid;
  grid-template-columns: 72rpx 68rpx minmax(0, 1fr) 112rpx;
  gap: var(--fi-space-16);
  align-items: center;
  width: 100%;
  overflow: hidden;
  padding: var(--fi-space-16) 0;
  border-bottom: var(--fi-primitive-border-width) solid #eff1f5;
}

.ranking-row--interactive {
  padding: var(--fi-space-14) 12rpx;
  border-radius: var(--fi-radius-md);
  transition: transform 180ms ease, background-color 180ms ease;
}

.ranking-row--pressed {
  transform: scale(0.99);
  background: rgba(243, 244, 247, 0.9);
}

.ranking-row__rank-wrap {
  display: flex;
  align-items: center;
  gap: var(--fi-space-6);
  flex-shrink: 0;
}

.ranking-row__rank {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  white-space: nowrap;
}

.ranking-row__rank--1 { color: var(--fi-color-primary); }
.ranking-row__rank--2 { color: #2563eb; }
.ranking-row__rank--3 { color: #16a34a; }

.ranking-row__avatar {
  width: 68rpx;
  height: 68rpx;
  border-radius: var(--fi-radius-round);
  background: #f5f6fa;
  flex-shrink: 0;
}

.ranking-row__body,
.ranking-row__metric {
  display: grid;
  min-width: 0;
}

.ranking-row__body {
  align-content: center;
}

.ranking-row__metric {
  justify-items: end;
  align-content: center;
  width: 112rpx;
  min-width: 112rpx;
  text-align: right;
  flex-shrink: 0;
  overflow: hidden;
}

.ranking-row__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-bold);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ranking-row__note,
.ranking-row__metric-note {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  white-space: nowrap;
}

.ranking-row__metric-value {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-extrabold);
  white-space: nowrap;
}

.ranking-row:last-child {
  border-bottom: 0;
  padding-bottom: 0;
}
</style>

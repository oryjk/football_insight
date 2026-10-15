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

    <RankingsTeamList
      v-if="scope === 'team' && activeTeamCategory"
      :active-team-category="activeTeamCategory"
      :team-categories="teamCategories"
      :standings-tables="standingsTables"
      :standings-ranking-mode="standingsRankingMode"
      @update:standings-ranking-mode="emit('update:standingsRankingMode', $event)"
      @open-team="emit('open-team', $event)"
    />

    <RankingsPlayerList
      v-if="scope === 'player' && activePlayerCategory"
      :active-player-category="activePlayerCategory"
      :player-categories="playerCategories"
      @open-player="emit('open-player', $event)"
      @select-match="(name, match) => emit('select-match', name, match)"
    />
  </view>
</template>

<script setup lang="ts">
import { computed, getCurrentInstance, nextTick, ref, watch } from 'vue'
import type {
  PlayerRankingCategory,
  PlayerRankingEntry,
  PlayerMatchGoals,
  RankingsViewResponse,
  StandingsTable,
  TeamRankingCategory,
  TeamRankingEntry,
} from '../../../types/insight'
import type { StandingsRankingMode } from '../poster'
import RankingsTeamList from './RankingsTeamList.vue'
import RankingsPlayerList from './RankingsPlayerList.vue'

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
  (e: 'open-player', entry: PlayerRankingEntry): void
  (e: 'select-match', playerName: string, match: PlayerMatchGoals): void
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
</style>

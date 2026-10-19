<template>
  <view class="panel ranking-surface">
    <view class="rankings-controls">
      <FiSegmented :model-value="scope" :items="scopeItems" aria-label="榜单类型" @update:model-value="onScopeChange" />

      <view class="category-tabs-wrap">
        <scroll-view scroll-x class="pill-row" :scroll-left="categoryScrollLeft" scroll-with-animation>
          <view :key="scope" class="pill-row__list">
            <view
              v-for="item in categoryOptions"
              :id="`ranking-category-${item.slug}`"
              :key="`${scope}-${item.slug}`"
              class="pill-row__item"
              :class="{ active: item.slug === effectiveSlug }"
              @click="selectCategory(item.slug)"
            >
              <text class="pill-row__item-text">{{ item.label }}</text>
            </view>
          </view>
        </scroll-view>
      </view>
    </view>

    <view :key="bodyKey" class="ranking-surface__body" :class="`ranking-surface__body--${enterMotion}`">
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
  </view>
</template>

<script setup lang="ts">
import { computed, getCurrentInstance, nextTick, reactive, ref, watch } from 'vue'
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
import teamIconIdle from '../../../static/rankings/scope-team-idle.svg'
import teamIconActive from '../../../static/rankings/scope-team-active.svg'
import playerIconIdle from '../../../static/rankings/scope-player-idle.svg'
import playerIconActive from '../../../static/rankings/scope-player-active.svg'
import FiSegmented from '../../../components/FiSegmented.vue'
import type { FiSegmentedItem } from '../../../types/segmented'
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

const scopeItems: FiSegmentedItem[] = [
  { key: 'team', label: '球队榜', icon: teamIconIdle, iconActive: teamIconActive },
  { key: 'player', label: '球员榜', icon: playerIconIdle, iconActive: playerIconActive },
]

function onScopeChange(value: string): void {
  if (value === 'team' || value === 'player') emit('update:scope', value)
}

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
// Each scope owns its selection: team and player categories can share slugs (e.g. goals), so the
// parent's single slug cannot tell which scope it belongs to right after a switch.
const selectedSlugs = reactive<Record<'team' | 'player', string>>({ team: '', player: '' })
const effectiveSlug = computed(() => {
  const options = categoryOptions.value
  const selected = selectedSlugs[props.scope]
  return options.some((item) => item.slug === selected) ? selected : (options[0]?.slug ?? '')
})

function selectCategory(slug: string): void {
  selectedSlugs[props.scope] = slug
}
const activeTeamCategory = computed(() =>
  teamCategories.value.find((item) => item.slug === effectiveSlug.value) ?? null,
)
const activePlayerCategory = computed(() =>
  playerCategories.value.find((item) => item.slug === effectiveSlug.value) ?? null,
)
const enterMotion = ref<'from-left' | 'from-right' | 'fade'>('fade')
const bodyKey = computed(() => `${props.scope}:${effectiveSlug.value}`)

// Sync flush so the re-keyed body is created with the right entrance motion: slide for a scope switch, fade for a category switch.
watch(bodyKey, (next, prev) => {
  const [nextScope] = next.split(':')
  const [prevScope] = prev.split(':')
  enterMotion.value = nextScope === prevScope ? 'fade' : nextScope === 'player' ? 'from-right' : 'from-left'
}, { flush: 'sync' })
watch(
  [categoryOptions, effectiveSlug],
  () => {
    if (effectiveSlug.value !== props.activeCategorySlug) {
      emit('update:activeCategorySlug', effectiveSlug.value)
    }
  },
  { immediate: true },
)

watch([() => props.activeCategorySlug, () => props.scope], () => {
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
  padding: 0 0 var(--fi-space-4);
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
  box-shadow: 0 6rpx 14rpx rgba(21, 22, 27, 0.14);
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

.ranking-surface__body {
  min-width: 0;
}

.ranking-surface__body--fade {
  animation: ranking-body-fade 260ms ease both;
}

.ranking-surface__body--from-right {
  animation: ranking-body-from-right 320ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.ranking-surface__body--from-left {
  animation: ranking-body-from-left 320ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

@keyframes ranking-body-fade {
  from {
    opacity: 0;
    transform: translateY(10rpx);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes ranking-body-from-right {
  from {
    opacity: 0;
    transform: translateX(36rpx);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes ranking-body-from-left {
  from {
    opacity: 0;
    transform: translateX(-36rpx);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
</style>

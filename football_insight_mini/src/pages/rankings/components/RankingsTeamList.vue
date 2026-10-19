<template>
  <view class="ranking-list">
    <view class="section-heading" :class="{ 'section-heading--standings': isStandingsTeamCategory }">
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

    <FiStandingsTable
      v-if="isStandingsTeamCategory"
      :teams="standingsTeams"
      @select="openStandingsTeam"
    />
    <template v-else>
      <view
        v-for="entry in teamEntries"
        :key="`${activeTeamCategory.slug}-${standingsRankingMode}-${entry.team_id}`"
        class="ranking-row ranking-row--interactive ranking-row--team"
        :class="resolveRankRowClass(entry.rank_no)"
        hover-class="ranking-row--pressed"
        hover-stay-time="100"
        @click="emit('open-team', entry)"
      >
        <view class="ranking-row__rank-wrap">
          <text class="ranking-row__rank" :class="resolveRankBadgeClass(entry.rank_no)">{{ formatRankLabel(entry.rank_no) }}</text>
        </view>
        <image :src="entry.avatar_storage_url || ''" mode="aspectFit" class="ranking-row__avatar" />
        <view class="ranking-row__body">
          <text class="ranking-row__name">{{ entry.team_name }}</text>
          <text class="ranking-row__note">点击查看球队信息和赛程</text>
        </view>
        <TeamAbilityHexagon :values="resolveTeamAbilityValues(entry.team_id)" />
        <view class="ranking-row__metric">
          <text class="ranking-row__metric-value">{{ entry.score_value }}</text>
          <text class="ranking-row__metric-note">{{ teamMetricLabel }}</text>
        </view>
      </view>
    </template>
  </view>

</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { StandingsTable, TeamRankingCategory, TeamRankingEntry } from '../../../types/insight'
import { buildStandingsRankingEntries, type StandingsRankingMode } from '../poster'
import { buildTeamAbilityValues, TEAM_ABILITY_FALLBACK } from '../ability'
import { buildStandingsTeams } from '../helpers'
import { formatRankLabel, resolveRankBadgeClass, resolveRankRowClass } from './rankingRowPresentation'
import TeamAbilityHexagon from './TeamAbilityHexagon.vue'
import FiStandingsTable from '../../../components/FiStandingsTable.vue'

const props = defineProps<{
  activeTeamCategory: TeamRankingCategory
  teamCategories: TeamRankingCategory[]
  standingsTables: StandingsTable[]
  standingsRankingMode: StandingsRankingMode
}>()

const emit = defineEmits<{
  (e: 'update:standingsRankingMode', value: StandingsRankingMode): void
  (e: 'open-team', entry: TeamRankingEntry): void
}>()

const activeTeamCategory = computed(() => props.activeTeamCategory)
const teamCategories = computed(() => props.teamCategories)
const standingsTables = computed(() => props.standingsTables)
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
const standingsTeams = computed(() => buildStandingsTeams(standingsTables.value, props.standingsRankingMode))

function openStandingsTeam(row: { key: number }): void {
  const entry = teamEntries.value.find((item) => item.team_id === row.key)
  if (entry) emit('open-team', entry)
}

const teamKicker = computed(() =>
  isStandingsTeamCategory.value ? '实时球队排名' : '实时球队累计榜',
)
const teamMetricLabel = computed(() => (isStandingsTeamCategory.value ? '积分' : '总计'))

const teamAbilityByTeamId = computed<Map<number, number[]>>(() =>
  buildTeamAbilityValues(teamCategories.value),
)

function resolveTeamAbilityValues(teamId: number): number[] {
  return teamAbilityByTeamId.value.get(teamId) ?? TEAM_ABILITY_FALLBACK
}

</script>

<style scoped>
@import "./rankings-shared.css";

.section-heading--standings .section-kicker {
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-medium);
  color: var(--fi-color-primary);
  letter-spacing: var(--fi-space-2);
}

.section-heading--standings .section-title {
  font-size: var(--fi-font-40);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.standings-mode-toggle {
  display: inline-grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-2);
  flex: 0 0 auto;
  padding: var(--fi-space-3);
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
</style>

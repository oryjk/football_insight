<template>
  <view class="ranking-list">
    <view class="section-heading">
      <view>
        <text class="section-kicker">实时球员累计榜</text>
        <text class="section-title">{{ activePlayerCategory.label }}</text>
      </view>
    </view>

    <FiPlayerGoalsTable
      v-if="activePlayerCategory.slug === 'goals'"
      :players="activePlayerCategory.entries"
      player-details
      @open-player="openGoalPlayer"
      @select-match="(name, match) => emit('select-match', name, match)"
    />
    <template v-else>
      <view
        v-for="entry in activePlayerCategory.entries"
        :key="`${activePlayerCategory.slug}-${entry.player_id}`"
        class="ranking-row ranking-row--player"
        :class="resolveRankRowClass(entry.rank_no)"
      >
        <text class="ranking-row__rank" :class="resolveRankBadgeClass(entry.rank_no)">{{ formatRankLabel(entry.rank_no) }}</text>
        <image
          :src="entry.avatar_storage_url || ''"
          mode="aspectFill"
          class="ranking-row__avatar ranking-row__avatar--player ranking-row__avatar--clickable"
          @click.stop="emit('open-player', entry)"
        />
        <view class="ranking-row__body">
          <text class="ranking-row__name">{{ entry.player_name }}</text>
          <text class="ranking-row__note">{{ entry.team_name }}</text>
        </view>
        <TeamAbilityHexagon :values="resolvePlayerAbilityValues(entry.player_id)" />
        <view class="ranking-row__metric">
          <text class="ranking-row__metric-value">{{ entry.score_value }}</text>
          <text class="ranking-row__metric-note">{{ activePlayerCategory.label }}</text>
        </view>
      </view>
    </template>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { PlayerMatchGoals, PlayerRankingCategory, PlayerRankingEntry } from '../../../types/insight'
import { buildPlayerAbilityValues, TEAM_ABILITY_FALLBACK } from '../ability'
import { formatRankLabel, resolveRankBadgeClass, resolveRankRowClass } from './rankingRowPresentation'
import TeamAbilityHexagon from './TeamAbilityHexagon.vue'
import FiPlayerGoalsTable from '../../../components/FiPlayerGoalsTable.vue'

const props = defineProps<{
  activePlayerCategory: PlayerRankingCategory
  playerCategories: PlayerRankingCategory[]
}>()

const emit = defineEmits<{
  (e: 'open-player', entry: PlayerRankingEntry): void
  (e: 'select-match', playerName: string, match: PlayerMatchGoals): void
}>()

const activePlayerCategory = computed(() => props.activePlayerCategory)
const playerCategories = computed(() => props.playerCategories)

function openGoalPlayer(playerId: number): void {
  const entry = activePlayerCategory.value?.entries.find(player => player.player_id === playerId)
  if (entry) emit('open-player', entry)
}
const playerAbilityByPlayerId = computed<Map<number, number[]>>(() =>
  buildPlayerAbilityValues(playerCategories.value),
)

function resolvePlayerAbilityValues(playerId: number): number[] {
  return playerAbilityByPlayerId.value.get(playerId) ?? TEAM_ABILITY_FALLBACK
}

</script>

<style scoped>
@import "./rankings-shared.css";
</style>

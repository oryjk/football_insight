<template>
  <view class="home-page-shell">
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" />
    <view class="page-bg-fade"></view>
    <scroll-view scroll-y class="page-scroll">
      <view class="page" @tap="collapseAiEntry">
        <view class="hero-card hero-card--home">
        <view class="hero-card__top">
          <view class="hero-card__heading">
            <text class="eyebrow">Football Insight</text>
            <text class="hero-card__title">这一轮之后，谁在改变联赛格局</text>
          </view>
          <view
            class="ai-entry"
            :class="{ 'ai-entry--expanded': aiEntryExpanded }"
            hover-class="ai-entry--pressed"
            hover-stay-time="120"
            @tap.stop="handleAiEntryTap"
          >
            <view class="ai-entry__bubble">
              <text class="ai-entry__bubble-text">嗡嗡嗡~ AI 对话</text>
              <text class="ai-entry__bubble-arrow">›</text>
            </view>

            <view class="ai-entry__charm">
              <view class="ai-entry__avatar-shell">
                <view class="ai-entry__avatar-glow"></view>
                <image :src="aiRonaldinhoAvatar" mode="aspectFill" class="ai-entry__avatar" />
                <view class="ai-entry__status">
                  <view class="ai-entry__status-dot"></view>
                </view>
              </view>

              <view class="ai-entry__tag">
                <text class="ai-entry__tag-text">AI 洞察</text>
              </view>
            </view>
          </view>
        </view>

        <view class="hero-card__guide">
          <view class="hero-card__guide-title">先看这三件事</view>
          <view class="hero-card__guide-copy">
            <template v-if="heroGuide.mode === 'team-and-scorer-with-match'">
              <text>先看 </text>
              <text class="inline-highlight">{{ heroGuide.topTeamName }}</text>
              <text> 能否守住榜首，再看 </text>
              <text class="inline-highlight">{{ heroGuide.topScorerName }}</text>
              <text> 是否继续领跑射手榜，最后看最新完赛如何继续改变联赛格局。</text>
            </template>
            <template v-else-if="heroGuide.mode === 'team-and-scorer-with-live-match'">
              <text>先看 </text>
              <text class="inline-highlight">{{ heroGuide.topTeamName }}</text>
              <text> 的榜首走势，再看 </text>
              <text class="inline-highlight">{{ heroGuide.topScorerName }}</text>
              <text> 是否继续领跑射手榜，最后盯住正在进行中的比分变化。</text>
            </template>
            <template v-else-if="heroGuide.mode === 'team-and-scorer'">
              <text>先看 </text>
              <text class="inline-highlight">{{ heroGuide.topTeamName }}</text>
              <text> 的榜首走势，再看 </text>
              <text class="inline-highlight">{{ heroGuide.topScorerName }}</text>
              <text> 领衔的射手竞争。</text>
            </template>
            <template v-else>
              <text>先看榜首走势，再看射手竞争，最后看最近哪场比赛最值得继续跟进。</text>
            </template>
          </view>
          <text v-if="heroGuideNote" class="hero-card__guide-note">{{ heroGuideNote }}</text>
        </view>

        <view v-if="isBriefingReady" class="briefing-grid">
          <view
            v-for="item in briefingItems"
            :key="item.label"
            class="briefing-card briefing-card--metric"
            :class="`briefing-card--${item.accent}`"
          >
            <text class="briefing-card__label">{{ item.label }}</text>

            <view class="briefing-card__body">
              <view class="briefing-card__main">
                <view
                  v-if="item.accent === 'leader' && item.avatars[0]?.src"
                  class="briefing-card__leader-logo"
                >
                  <image
                    :src="item.avatars[0].src"
                    :alt="item.avatars[0].name"
                    mode="aspectFit"
                    class="briefing-card__leader-logo-image"
                  />
                </view>

                <view v-else-if="item.avatars.length" class="briefing-card__entity-group">
                  <image
                    v-for="avatar in item.avatars.slice(0, 3)"
                    :key="`${item.label}-${avatar.name}`"
                    :src="avatar.src || ''"
                    :alt="avatar.name"
                    mode="aspectFill"
                    class="briefing-card__entity-avatar"
                  />
                </view>

                <view class="briefing-card__title-block">
                  <text class="briefing-card__value">{{ item.value }}</text>
                  <text v-if="item.subValue" class="briefing-card__subvalue">{{ item.subValue }}</text>
                </view>
              </view>

              <view v-if="getBriefingMarqueeRows(item.accent).length" class="briefing-card__marquees">
                <view
                  v-for="(row, rowIndex) in getBriefingMarqueeRows(item.accent)"
                  :key="`${item.label}-marquee-${rowIndex}`"
                  class="briefing-card__marquee"
                >
                  <view
                    class="briefing-card__marquee-track"
                    :style="{ animationDuration: `${15 + rowIndex * 2}s` }"
                  >
                    <text
                      v-for="(message, messageIndex) in [...row, ...row]"
                      :key="`${item.label}-marquee-${rowIndex}-${messageIndex}`"
                      class="briefing-card__marquee-item"
                    >
                      {{ message }}
                    </text>
                  </view>
                </view>
              </view>

              <view class="briefing-card__metric">
                <text class="briefing-card__metric-value">{{ item.metricValue }}</text>
                <text class="briefing-card__metric-label">{{ item.metricLabel }}</text>
              </view>
            </view>
          </view>
        </view>
      </view>

      <view class="panel support-home-panel">
        <view class="support-home-panel__header">
          <view class="support-home-panel__heading">
            <text class="section-kicker">我的主队</text>
            <text class="section-title">下一场先为谁站队</text>
          </view>

          <view class="support-home-panel__context">
            <view class="support-home-panel__context-dot"></view>
            <text class="support-home-panel__context-label">{{ supportPanelBadge }}</text>
            <text class="support-home-panel__context-note">{{ supportPanelContextNote }}</text>
          </view>
        </view>

        <FiLoading
          v-if="supportLoading"
          title="助力入口加载中"
          caption="正在确认你的主队和下一场比赛。"
        />

        <view v-else-if="supportErrorMessage" class="state-card state-card--error">
          <text>{{ supportErrorMessage }}</text>
        </view>

        <template v-else-if="!hasAuthToken">
          <text class="support-home-panel__summary">
            登录后才能关注主队、参与赛前助力，并把比赛页面转发出去拉票。
          </text>
          <button class="support-home-panel__action" @click="handleSupportLogin">去登录</button>
        </template>

        <template v-else-if="!supportFavoriteTeam">
          <text class="support-home-panel__summary">
            先选择一支主队，首页就会把它的下一场比赛和助力入口放到第一屏。
          </text>
          <button class="support-home-panel__action" @click="openFavoriteTeamSheet">选择主队</button>
        </template>

        <template v-else-if="supportNextMatch">
          <view class="support-home-panel__favorite">
            <image :src="supportFavoriteTeam.avatar_storage_url || ''" mode="aspectFit" class="support-home-panel__favorite-avatar" />
            <view class="support-home-panel__favorite-body">
              <text class="support-home-panel__favorite-name">{{ supportFavoriteTeam.team_name }}</text>
              <text class="support-home-panel__favorite-note">{{ supportFavoriteTeamLabel }}</text>
            </view>
            <button class="support-home-panel__switch" @click="openFavoriteTeamSheet">切换主队</button>
          </view>

          <view class="support-home-match-card" @click="openSupportMatch">
            <view class="support-home-match-card__meta">
              <text>第 {{ supportNextMatch.round_number }} 轮</text>
              <text>{{ supportNextMatch.match_date }} {{ supportNextMatch.match_time }}</text>
            </view>
            <view class="support-home-match-card__teams">
              <text class="support-home-match-card__team">{{ supportNextMatch.home_team.team_name }}</text>
              <text class="support-home-match-card__vs">{{ supportWindowShortLabel }}</text>
              <text class="support-home-match-card__team support-home-match-card__team--away">{{ supportNextMatch.away_team.team_name }}</text>
            </view>
            <view class="support-home-match-card__bar">
              <view class="support-home-match-card__bar-home" :style="{ width: `${supportNextMatch.home_team.support_share_pct}%` }" />
              <view class="support-home-match-card__bar-away" :style="{ width: `${supportNextMatch.away_team.support_share_pct}%` }" />
            </view>
            <view class="support-home-match-card__footer">
              <text>{{ supportNextMatchLabel }}</text>
              <text>点击进入助力页</text>
            </view>
          </view>
        </template>

        <template v-else>
          <text class="support-home-panel__summary">
            {{ supportFavoriteTeam.team_name }} 当前还没有可展示的下一场助力比赛，等赛程刷新后这里会自动出现。
          </text>
          <button class="support-home-panel__action support-home-panel__action--ghost" @click="openFavoriteTeamSheet">切换主队</button>
        </template>
      </view>

      <view v-if="loading" class="skeleton-stack">
        <view class="hero-card hero-card--home skeleton-panel">
          <view class="hero-card__top">
            <view class="skeleton-copy-group">
              <view class="skeleton-line skeleton-line--kicker" />
              <view class="skeleton-line skeleton-line--hero-title" />
            </view>
          </view>
          <view class="hero-card__guide skeleton-hero-guide">
            <view class="skeleton-line skeleton-line--guide-title" />
            <view class="skeleton-line skeleton-line--body" />
            <view class="skeleton-line skeleton-line--body skeleton-line--body-short" />
          </view>
        </view>

        <view class="briefing-grid skeleton-briefing-grid">
          <view
            v-for="i in 3"
            :key="`skeleton-briefing-${i}`"
            class="briefing-card briefing-card--metric skeleton-panel"
          >
            <view class="skeleton-line skeleton-line--kicker" />
            <view class="skeleton-briefing-body">
              <view class="skeleton-line skeleton-line--briefing-value" />
              <view class="skeleton-line skeleton-line--briefing-metric" />
            </view>
          </view>
        </view>

        <view class="panel skeleton-panel">
          <view class="section-heading section-heading--compact">
            <view class="skeleton-copy-group">
              <view class="skeleton-line skeleton-line--kicker" />
              <view class="skeleton-line skeleton-line--section" />
            </view>
            <view class="skeleton-pill skeleton-pill--short" />
          </view>
          <view class="skeleton-line skeleton-line--body" />
          <view class="skeleton-button" />
        </view>

        <view class="panel skeleton-panel">
          <view class="section-heading">
            <view class="skeleton-copy-group">
              <view class="skeleton-line skeleton-line--kicker" />
              <view class="skeleton-line skeleton-line--title" />
            </view>
            <view class="skeleton-pill" />
          </view>
          <view class="skeleton-line skeleton-line--body" />
          <view class="score-strip skeleton-score-strip">
            <view class="skeleton-line skeleton-line--meta skeleton-line--meta-wide" />
            <view class="skeleton-score-body">
              <view class="skeleton-line skeleton-line--score-team" />
              <view class="skeleton-line skeleton-line--score-value" />
              <view class="skeleton-line skeleton-line--score-team" />
            </view>
          </view>
          <view class="score-strip skeleton-score-strip">
            <view class="skeleton-line skeleton-line--meta skeleton-line--meta-wide" />
            <view class="skeleton-score-body">
              <view class="skeleton-line skeleton-line--score-team" />
              <view class="skeleton-line skeleton-line--score-value" />
              <view class="skeleton-line skeleton-line--score-team" />
            </view>
          </view>
        </view>

        <view class="panel skeleton-panel">
          <view class="section-heading section-heading--compact">
            <view class="skeleton-copy-group">
              <view class="skeleton-line skeleton-line--kicker" />
              <view class="skeleton-line skeleton-line--section" />
            </view>
          </view>
          <view class="ranking-list">
            <view
              v-for="i in 4"
              :key="`skeleton-standing-${i}`"
              class="ranking-row skeleton-ranking-row"
            >
              <view class="skeleton-line skeleton-line--rank" />
              <view class="skeleton-line skeleton-line--avatar" />
              <view class="skeleton-ranking-body">
                <view class="skeleton-line skeleton-line--name" />
                <view class="skeleton-line skeleton-line--note" />
              </view>
              <view class="skeleton-line skeleton-line--points" />
            </view>
          </view>
        </view>

        <view class="panel skeleton-panel">
          <view class="section-heading section-heading--compact">
            <view class="skeleton-copy-group">
              <view class="skeleton-line skeleton-line--kicker" />
              <view class="skeleton-line skeleton-line--section" />
            </view>
          </view>
          <view class="ranking-list">
            <view
              v-for="i in 4"
              :key="`skeleton-scorer-${i}`"
              class="ranking-row skeleton-ranking-row"
            >
              <view class="skeleton-line skeleton-line--rank" />
              <view class="skeleton-line skeleton-line--avatar" />
              <view class="skeleton-ranking-body">
                <view class="skeleton-line skeleton-line--name" />
                <view class="skeleton-line skeleton-line--note" />
              </view>
              <view class="skeleton-line skeleton-line--points" />
            </view>
          </view>
        </view>
      </view>

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
      </view>

      <template v-else-if="overview">
        <view class="panel story-card">
          <view class="section-heading">
            <view>
              <text class="section-kicker">实时脉冲</text>
              <view class="story-card__headline">
                <text>{{ headlineTitleParts.leading }}</text>
                <text v-if="headlineTitleParts.highlighted" class="inline-highlight">{{ headlineTitleParts.highlighted }}</text>
                <text>{{ headlineTitleParts.trailing }}</text>
              </view>
            </view>
            <text class="meta-note">更新于 {{ updatedAtLabel }}</text>
          </view>

          <text class="story-card__copy">{{ headlineBody }}</text>

          <view v-if="pulseMatches.length" class="score-strip-stack">
            <view
              v-for="match in pulseMatches"
              :key="`pulse-${match.status}-${match.match_id}`"
              class="score-strip"
              :class="{ 'score-strip--interactive': hasPulseMatchTechStats(match) }"
              hover-class="score-strip--pressed"
              hover-stay-time="100"
              @click="openPulseMatchTechStats(match)"
            >
              <view class="score-strip__meta">
                <text>第 {{ match.round_number }} 轮</text>
                <view class="score-strip__meta-trailing">
                  <text>{{ match.match_date }} {{ match.match_time }}</text>
                  <text v-if="hasPulseMatchTechStats(match)" class="score-strip__meta-pill">技术统计</text>
                </view>
              </view>
              <view class="score-strip__body">
                <view class="score-strip__team">
                  <text class="score-strip__team-name inline-highlight">{{ match.home_team_name }}</text>
                </view>
                <text class="score-strip__score inline-highlight">{{ match.home_score }} : {{ match.away_score }}</text>
                <view class="score-strip__team score-strip__team--away">
                  <text class="score-strip__team-name inline-highlight">{{ match.away_team_name }}</text>
                </view>
              </view>
              <view v-if="hasPulseMatchTechStats(match)" class="score-strip__hint">
                <text class="score-strip__hint-text">点击查看技术统计</text>
              </view>
            </view>
          </view>

          <view class="watch-list">
            <view v-for="item in watchPoints" :key="item" class="watch-list__item">
              <text>{{ item }}</text>
            </view>
          </view>
        </view>

        <view class="panel">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">积分榜头部</text>
              <text class="section-title">先看联赛头部格局</text>
            </view>
          </view>

          <view class="ranking-list">
            <view
              v-for="team in standings"
              :key="team.team_id"
              class="ranking-row ranking-row--interactive"
              hover-class="ranking-row--pressed"
              hover-stay-time="100"
              @click="openStandingsTeamSheet(team)"
            >
              <text class="ranking-row__rank" :class="`ranking-row__rank--${team.rank_no}`">#{{ team.rank_no }}</text>
              <image :src="team.avatar_storage_url || ''" mode="aspectFit" class="ranking-row__avatar" />
              <view class="ranking-row__body">
                <text class="ranking-row__name">{{ team.team_name }}</text>
                <text class="ranking-row__note">点击查看赛季战绩</text>
              </view>
              <view class="ranking-row__metric">
                <text class="ranking-row__metric-value">{{ team.points }}</text>
                <text class="ranking-row__metric-note">积分</text>
              </view>
            </view>
          </view>
        </view>

        <view class="panel">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">射手榜</text>
              <text class="section-title">当前赛季累计射手榜</text>
            </view>
          </view>

          <view class="ranking-list">
            <view v-for="player in scorers" :key="player.player_id" class="ranking-row">
              <text class="ranking-row__rank" :class="`ranking-row__rank--${player.rank_no}`">#{{ player.rank_no }}</text>
              <image :src="player.avatar_storage_url || ''" mode="aspectFill" class="ranking-row__avatar ranking-row__avatar--player" />
              <view class="ranking-row__body">
                <text class="ranking-row__name">{{ player.player_name }}</text>
                <text class="ranking-row__note">{{ player.team_name }}</text>
              </view>
              <view class="ranking-row__metric">
                <text class="ranking-row__metric-value">{{ player.score_value }}</text>
                <text class="ranking-row__metric-note">累计进球</text>
              </view>
            </view>
          </view>
        </view>

      </template>
      </view>
    </scroll-view>

    <FiAiChatSheet
      :visible="aiChatVisible"
      :current-user="currentAiUser"
      :ai-chat-mode="publicConfig?.ai_chat_mode"
      @close="handleCloseAiChat"
    />

    <view
      v-if="selectedPulseMatch"
      class="sheet-mask sheet-mask--tech-stats"
      @tap="closePulseMatchTechStats"
    >
      <view class="sheet-card tech-stats-sheet" @tap.stop="consumeSheetTap">
        <view class="section-heading section-heading--compact">
          <view>
            <text class="section-kicker">比赛技术统计</text>
            <text class="section-title">技术统计</text>
          </view>
          <button class="tech-stats-sheet__close" @click="closePulseMatchTechStats">关闭</button>
        </view>

        <view class="tech-stats-sheet__summary">
          <text class="tech-stats-sheet__teams">
            {{ selectedPulseMatch.home_team_name }} {{ selectedPulseMatch.home_score }} : {{ selectedPulseMatch.away_score }} {{ selectedPulseMatch.away_team_name }}
          </text>
          <text class="tech-stats-sheet__meta">
            第 {{ selectedPulseMatch.round_number }} 轮 · {{ selectedPulseMatch.match_date }} {{ selectedPulseMatch.match_time }}
          </text>
        </view>

        <view class="tech-stats-sheet__list">
          <view
            v-for="(stat, index) in selectedPulseMatchTechStats"
            :key="stat.key"
            class="tech-stat-row"
            :style="getTechStatRowStyle(index)"
          >
            <text class="tech-stat-row__value">{{ stat.homeValue }}</text>
            <view class="tech-stat-row__track tech-stat-row__track--home">
              <view
                class="tech-stat-row__fill tech-stat-row__fill--home"
                :style="{ width: `${stat.homeBarPercent}%` }"
              />
            </view>
            <text class="tech-stat-row__label">{{ stat.label }}</text>
            <view class="tech-stat-row__track tech-stat-row__track--away">
              <view
                class="tech-stat-row__fill tech-stat-row__fill--away"
                :style="{ width: `${stat.awayBarPercent}%` }"
              />
            </view>
            <text class="tech-stat-row__value tech-stat-row__value--away">{{ stat.awayValue }}</text>
          </view>
        </view>

        <text class="tech-stats-sheet__footnote">当前展示雷速提供的比赛技术统计，后续会继续补充更多指标。</text>
      </view>
    </view>

    <view
      v-if="selectedStandingsTeam"
      class="sheet-mask sheet-mask--tech-stats"
      @tap="closeStandingsTeamSheet"
    >
      <view class="sheet-card tech-stats-sheet team-season-sheet" @tap.stop="consumeSheetTap">
        <view class="section-heading section-heading--compact">
          <view>
            <text class="section-kicker">球队赛季战绩</text>
            <text class="section-title">{{ selectedStandingsTeam.team_name }}</text>
          </view>
          <button class="tech-stats-sheet__close" @click="closeStandingsTeamSheet">关闭</button>
        </view>

        <view class="team-season-sheet__summary">
          <view class="team-season-sheet__summary-main">
            <image
              :src="selectedStandingsTeam.avatar_storage_url || ''"
              mode="aspectFit"
              class="team-season-sheet__summary-avatar"
            />
            <view class="team-season-sheet__summary-copy">
              <text class="team-season-sheet__summary-name">{{ selectedStandingsTeam.team_name }}</text>
              <text class="team-season-sheet__summary-meta">当前第 {{ selectedStandingsTeam.rank_no }} · {{ selectedStandingsTeam.points }} 分</text>
            </view>
          </view>
          <text class="team-season-sheet__summary-record">{{ selectedStandingsTeamRecord }}</text>
        </view>

        <FiLoading
          v-if="teamSeasonMatchesLoading"
          title="赛季战绩加载中"
          caption="正在整理这支球队本赛季的每场比赛。"
        />

        <view v-else-if="teamSeasonMatchesErrorMessage" class="state-card state-card--error team-season-sheet__state">
          <text>{{ teamSeasonMatchesErrorMessage }}</text>
        </view>

        <scroll-view v-else-if="selectedStandingsTeamMatches.length" scroll-y class="team-season-sheet__list">
          <view
            v-for="(match, index) in selectedStandingsTeamMatches"
            :key="match.matchId"
            class="team-season-match-row"
            :style="getTeamMatchRowStyle(index)"
          >
            <view class="team-season-match-row__meta">
              <text>第 {{ match.roundNumber }} 轮 · {{ match.matchDate }} {{ match.matchTime }}</text>
              <text class="team-season-match-row__result" :class="`team-season-match-row__result--${match.resultTone}`">
                {{ match.resultLabel }}
              </text>
            </view>
            <view class="team-season-match-row__body">
              <text class="team-season-match-row__team" :class="{ 'team-season-match-row__team--active': match.isHomeTeam }">
                {{ match.homeTeamName }}
              </text>
              <text class="team-season-match-row__score">{{ match.scoreText }}</text>
              <text class="team-season-match-row__team team-season-match-row__team--away" :class="{ 'team-season-match-row__team--active': !match.isHomeTeam }">
                {{ match.awayTeamName }}
              </text>
            </view>
          </view>
        </scroll-view>

        <view v-else class="team-season-sheet__empty">
          <text>这支球队当前还没有可展示的赛季比赛记录。</text>
        </view>
      </view>
    </view>

    <view v-if="favoriteTeamSheetVisible" class="sheet-mask" @tap="closeFavoriteTeamSheet">
      <view class="sheet-card" @tap.stop="consumeSheetTap">
        <view class="section-heading section-heading--compact">
          <view>
            <text class="section-kicker">选择主队</text>
            <text class="section-title">以后首页默认先给它站队</text>
          </view>
          <text class="meta-note">单主队 MVP</text>
        </view>

        <scroll-view scroll-y class="favorite-team-sheet__list">
          <view
            v-for="team in supportTeams"
            :key="team.team_id"
            class="favorite-team-sheet__row"
            :class="{ 'favorite-team-sheet__row--active': selectedFavoriteTeamId === team.team_id }"
            @click="selectedFavoriteTeamId = team.team_id"
          >
            <image :src="team.avatar_storage_url || ''" mode="aspectFit" class="favorite-team-sheet__avatar" />
            <view class="favorite-team-sheet__body">
              <text class="favorite-team-sheet__name">{{ team.team_name }}</text>
              <text class="favorite-team-sheet__note">{{ team.rank_no ? `当前积分榜第 ${team.rank_no}` : '等待排名同步' }}</text>
            </view>
          </view>
        </scroll-view>

        <view class="sheet-actions">
          <button class="primary-action primary-action--ghost" @click="closeFavoriteTeamSheet">取消</button>
          <button class="primary-action" @click="handleConfirmFavoriteTeam">确认主队</button>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import FiAiChatSheet from '../../components/FiAiChatSheet.vue'
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import { getCurrentUser } from '../../api/auth'
import { getAvailableRounds, getMatches, getOverview, getRankings } from '../../api/insight'
import { getSupportProfile, listSupportTeams, setFavoriteTeam } from '../../api/support'
import { getPublicSystemConfig } from '../../api/system'
import aiRonaldinhoAvatar from '../../static/ai/ronaldinho-avatar.png'
import type { CurrentUser } from '../../types/auth'
import type {
  InsightOverviewResponse,
  MatchCard,
  OverviewMatch,
  OverviewPlayer,
  OverviewStanding,
  PlayerRankingCategory,
  RankingsViewResponse,
  RoundReference,
} from '../../types/insight'
import type { SupportMatchDetail, SupportProfile, SupportTeam } from '../../types/support'
import type { PublicSystemConfig } from '../../types/system'
import { extractApiErrorMessage } from '../../utils/apiError'
import { getAccessToken } from '../../utils/authStorage'
import bgImage from '../../static/home/bg.webp'
import { buildHomeBriefingMarqueeMap, splitBriefingMarqueeRows, type HomeBriefingMarqueeAccent } from '../../utils/homeBriefingMarquees'
import { buildHeadlineTitleParts } from '../../utils/homeViewText'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { reportPageActivity } from '../../utils/userActivity'
import {
  type HomeTeamSeasonMatch,
  type HomePulseLeadMatch,
  resolveHomeAiEntryTapResult,
  resolveHomeGuideLeaders,
  resolveHomeGuideNote,
  resolveHomeGuideReferenceRoundNumber,
  resolveHomeHasAuthToken,
  resolveHomePulseLeadMatch,
  resolveHomePulseMatches,
  resolveHomePulseTechStats,
  resolveHomeTeamSeasonMatches,
  resolveHomeSupportNextMatchLabel,
  resolveHomeSupportWindowShortLabel,
} from './helpers'

type HeroGuide =
  | { mode: 'team-and-scorer-with-match'; topTeamName: string; topScorerName: string }
  | { mode: 'team-and-scorer-with-live-match'; topTeamName: string; topScorerName: string }
  | { mode: 'team-and-scorer'; topTeamName: string; topScorerName: string }
  | { mode: 'fallback' }

interface BriefingItem {
  accent: HomeBriefingMarqueeAccent
  label: string
  value: string
  subValue: string | null
  metricValue: string
  metricLabel: string
  avatars: Array<{ name: string; src: string | null }>
}

const currentSeason = new Date().getFullYear()
const loading = ref(true)
const errorMessage = ref('')
const overview = ref<InsightOverviewResponse | null>(null)
const liveMatches = ref<MatchCard[]>([])
const rounds = ref<RoundReference[]>([])
const rankings = ref<{ player_categories: PlayerRankingCategory[] } | null>(null)
const guideRankings = ref<RankingsViewResponse | null>(null)
const publicConfig = ref<PublicSystemConfig | null>(null)
const currentAiUser = ref<CurrentUser | null>(null)
const aiChatVisible = ref(false)
const aiEntryExpanded = ref(false)
const selectedPulseMatch = ref<HomePulseLeadMatch | null>(null)
const selectedStandingsTeam = ref<OverviewStanding | null>(null)
const allSeasonMatches = ref<MatchCard[] | null>(null)
const teamSeasonMatchesLoading = ref(false)
const teamSeasonMatchesErrorMessage = ref('')
const supportLoading = ref(true)
const supportErrorMessage = ref('')
const supportProfile = ref<SupportProfile | null>(null)
const supportTeams = ref<SupportTeam[]>([])
const favoriteTeamSheetVisible = ref(false)
const selectedFavoriteTeamId = ref<number | null>(null)
const hasAuthToken = ref(resolveHomeHasAuthToken(getAccessToken()))

const standings = computed<OverviewStanding[]>(() => overview.value?.standings_top ?? [])
const scorers = computed<OverviewPlayer[]>(() => overview.value?.top_scorers ?? [])
const recentMatches = computed<OverviewMatch[]>(() => (overview.value?.recent_matches ?? []).slice(0, 4))
const insightSummary = computed(() => overview.value?.insight_summary ?? null)
const pulseMatches = computed(() =>
  resolveHomePulseMatches(liveMatches.value, recentMatches.value),
)
const leadMatch = computed(() =>
  pulseMatches.value[0] ?? resolveHomePulseLeadMatch(liveMatches.value, recentMatches.value),
)
const selectedPulseMatchTechStats = computed(() =>
  selectedPulseMatch.value ? resolveHomePulseTechStats(selectedPulseMatch.value) : [],
)
const selectedStandingsTeamMatches = computed<HomeTeamSeasonMatch[]>(() => {
  if (!selectedStandingsTeam.value || !allSeasonMatches.value) {
    return []
  }

  return resolveHomeTeamSeasonMatches(selectedStandingsTeam.value, allSeasonMatches.value)
})
const selectedStandingsTeamRecord = computed(() => {
  const finishedMatches = selectedStandingsTeamMatches.value.filter((match) =>
    match.resultTone === 'win' || match.resultTone === 'draw' || match.resultTone === 'loss',
  )
  const wins = finishedMatches.filter((match) => match.resultTone === 'win').length
  const draws = finishedMatches.filter((match) => match.resultTone === 'draw').length
  const losses = finishedMatches.filter((match) => match.resultTone === 'loss').length

  return `已赛 ${finishedMatches.length} 场 · ${wins}胜 ${draws}平 ${losses}负`
})
const topTeam = computed<OverviewStanding | null>(() => standings.value[0] ?? null)
const topScorer = computed<OverviewPlayer | null>(() => scorers.value[0] ?? null)
const guideLeaders = computed(() => resolveHomeGuideLeaders({
  rounds: rounds.value,
  liveStandings: standings.value,
  liveScorers: scorers.value,
  referenceRankings: guideRankings.value,
}))
const assistCategory = computed<PlayerRankingCategory | null>(() =>
  rankings.value?.player_categories.find((item) => item.slug === 'assists') ?? null,
)
const topAssist = computed(() => assistCategory.value?.entries[0] ?? null)
const isBriefingReady = computed(() => !loading.value && !errorMessage.value && !!overview.value)
const supportFavoriteTeam = computed(() => supportProfile.value?.favorite_team ?? null)
const supportNextMatch = computed<SupportMatchDetail | null>(() => supportProfile.value?.next_match ?? null)
const supportPanelBadge = computed(() => {
  if (!hasAuthToken.value) {
    return '登录后开启'
  }

  return supportFavoriteTeam.value?.team_name ?? '待选主队'
})
const supportPanelContextNote = computed(() => {
  if (!hasAuthToken.value) {
    return '登录后查看主队助力入口'
  }

  if (!supportFavoriteTeam.value) {
    return '先选主队，再把比赛入口固定到首页'
  }

  return '当前关注球队'
})
const supportFavoriteTeamLabel = computed(() => {
  if (!supportFavoriteTeam.value) {
    return ''
  }

  return supportFavoriteTeam.value.rank_no
    ? `当前积分榜第 ${supportFavoriteTeam.value.rank_no}`
    : '已关注主队'
})
const supportWindowShortLabel = computed(() => {
  return resolveHomeSupportWindowShortLabel(supportNextMatch.value)
})
const supportNextMatchLabel = computed(() => {
  return resolveHomeSupportNextMatchLabel(supportNextMatch.value)
})

onShareAppMessage(() => ({
  title: supportFavoriteTeam.value
    ? `${supportFavoriteTeam.value.team_name} 下一场先为谁站队？`
    : '足球洞察：这一轮之后，谁在改变联赛格局',
  path: '/pages/home/index',
}))

const leadingTeams = computed(() => {
  if (!topTeam.value) {
    return []
  }

  return standings.value.filter((team) => team.points === topTeam.value?.points)
})

const leadingScorers = computed(() => {
  if (!topScorer.value) {
    return []
  }

  return scorers.value.filter((player) => player.score_value === topScorer.value?.score_value)
})

const leadingAssists = computed(() => {
  if (!topAssist.value || !assistCategory.value) {
    return []
  }

  return assistCategory.value.entries.filter((player) => player.score_value === topAssist.value?.score_value)
})

const updatedAtLabel = computed(() => {
  if (!overview.value?.latest_scrape_finished_at) {
    return '等待同步'
  }

  const date = new Date(overview.value.latest_scrape_finished_at)
  return `${date.getMonth() + 1}/${date.getDate()} ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
})

const headlineTitleParts = computed(() =>
  buildHeadlineTitleParts({
    headline: insightSummary.value?.headline ?? null,
    leadMatch: leadMatch.value
      ? {
          homeTeamName: leadMatch.value.home_team_name,
          awayTeamName: leadMatch.value.away_team_name,
          homeScore: leadMatch.value.home_score,
          awayScore: leadMatch.value.away_score,
        }
      : null,
    topTeamName: topTeam.value?.team_name ?? null,
  }),
)

const headlineBody = computed(() => {
  if (insightSummary.value?.summary) {
    return insightSummary.value.summary
  }

  if (topScorer.value) {
    return `${topScorer.value.player_name}暂时领跑赛季累计射手榜，最近赛果和榜单头部值得继续跟踪。`
  }

  return '当前可查看积分榜头部、最近赛果和榜单焦点。'
})

const heroGuide = computed<HeroGuide>(() => {
  const topTeamNames = formatEntityNames(guideLeaders.value.topTeamNames)
  const topScorerNames = formatEntityNames(guideLeaders.value.topScorerNames)

  if (topTeamNames && topScorerNames && leadMatch.value) {
    if (leadMatch.value.status === 'live') {
      return {
        mode: 'team-and-scorer-with-live-match',
        topTeamName: topTeamNames,
        topScorerName: topScorerNames,
      }
    }

    return {
      mode: 'team-and-scorer-with-match',
      topTeamName: topTeamNames,
      topScorerName: topScorerNames,
    }
  }

  if (topTeamNames && topScorerNames) {
    return {
      mode: 'team-and-scorer',
      topTeamName: topTeamNames,
      topScorerName: topScorerNames,
    }
  }

  return { mode: 'fallback' }
})

const heroGuideNote = computed(() => {
  return resolveHomeGuideNote(guideLeaders.value.source)
})

const briefingMarqueeMap = computed(() =>
  buildHomeBriefingMarqueeMap(publicConfig.value?.home_briefing_marquees),
)

const briefingItems = computed<BriefingItem[]>(() => {
  const items: Array<BriefingItem | null> = [
    topTeam.value
      ? {
          accent: 'leader',
          label: '榜首风向',
          value: formatEntityNames(leadingTeams.value.map((team) => team.team_name)),
          subValue: null,
          metricValue: String(topTeam.value.points),
          metricLabel: leadingTeams.value.length > 1 ? '分并列榜首' : '分领跑',
          avatars: leadingTeams.value.map((team) => ({
            name: team.team_name,
            src: team.avatar_storage_url,
          })),
        }
      : null,
    topScorer.value
      ? {
          accent: 'scorer',
          label: '射手头条',
          value: formatEntityNames(leadingScorers.value.map((player) => player.player_name)),
          subValue: formatEntityNames(leadingScorers.value.map((player) => player.team_name)),
          metricValue: String(topScorer.value.score_value),
          metricLabel: leadingScorers.value.length > 1 ? '球并列头名' : '球',
          avatars: leadingScorers.value.map((player) => ({
            name: player.player_name,
            src: player.avatar_storage_url,
          })),
        }
      : null,
    topAssist.value
      ? {
          accent: 'assist',
          label: '助攻头条',
          value: formatEntityNames(leadingAssists.value.map((player) => player.player_name)),
          subValue: formatEntityNames(leadingAssists.value.map((player) => player.team_name)),
          metricValue: String(topAssist.value.score_value),
          metricLabel: leadingAssists.value.length > 1 ? '次并列头名' : '次助攻',
          avatars: leadingAssists.value.map((player) => ({
            name: player.player_name,
            src: player.avatar_storage_url,
          })),
        }
      : null,
  ]

  return items.filter((item): item is BriefingItem => item !== null)
})

const watchPoints = computed(() => {
  if (insightSummary.value?.bullets?.length) {
    return insightSummary.value.bullets
  }

  const items: string[] = []

  if (leadMatch.value) {
    items.push(
      leadMatch.value.status === 'live'
        ? `当前进行中焦点是 ${leadMatch.value.home_team_name} 对阵 ${leadMatch.value.away_team_name}`
        : `最新完赛焦点是 ${leadMatch.value.home_team_name} 对阵 ${leadMatch.value.away_team_name}`,
    )
  }

  if (topTeam.value) {
    items.push(`${topTeam.value.team_name} 继续占据积分榜头部位置`)
  }

  if (topScorer.value && scorers.value[1]) {
    items.push(`${topScorer.value.player_name} 与 ${scorers.value[1].player_name} 正在争夺赛季累计射手榜头部`)
  }

  return items
})

function formatEntityNames(names: string[]): string {
  return names.join('、')
}

function getBriefingMarqueeRows(accent: HomeBriefingMarqueeAccent): string[][] {
  return splitBriefingMarqueeRows(briefingMarqueeMap.value[accent] ?? [])
}

function consumeSheetTap(): void {}

function getTechStatRowStyle(index: number) {
  return {
    '--tech-stat-delay': `${120 + index * 70}ms`,
  }
}

function getTeamMatchRowStyle(index: number) {
  return {
    '--team-match-delay': `${100 + index * 55}ms`,
  }
}

function hasPulseMatchTechStats(match: HomePulseLeadMatch): boolean {
  return resolveHomePulseTechStats(match).length > 0
}

function openPulseMatchTechStats(match: HomePulseLeadMatch) {
  if (!hasPulseMatchTechStats(match)) {
    uni.showToast({ title: '这场比赛暂时还没有技术统计', icon: 'none' })
    return
  }

  selectedPulseMatch.value = match
}

function closePulseMatchTechStats() {
  selectedPulseMatch.value = null
}

async function ensureAllSeasonMatchesLoaded() {
  if (allSeasonMatches.value) {
    return
  }

  if (teamSeasonMatchesLoading.value) {
    return
  }

  teamSeasonMatchesLoading.value = true
  teamSeasonMatchesErrorMessage.value = ''

  try {
    const availableRounds = rounds.value.length ? rounds.value : await getAvailableRounds(currentSeason)
    if (!rounds.value.length) {
      rounds.value = availableRounds
    }

    const responses = await Promise.all(
      availableRounds.map((round) =>
        getMatches({ mode: 'round', season: currentSeason, roundNumber: round.round_number }),
      ),
    )

    const matchMap = new Map<number, MatchCard>()
    for (const response of responses) {
      for (const match of response.matches) {
        matchMap.set(match.match_id, match)
      }
    }

    allSeasonMatches.value = Array.from(matchMap.values())
  } catch (error) {
    teamSeasonMatchesErrorMessage.value = extractApiErrorMessage(error, '球队赛季战绩加载失败，请稍后重试。')
  } finally {
    teamSeasonMatchesLoading.value = false
  }
}

async function openStandingsTeamSheet(team: OverviewStanding) {
  selectedStandingsTeam.value = team
  await ensureAllSeasonMatchesLoaded()
}

function closeStandingsTeamSheet() {
  selectedStandingsTeam.value = null
  teamSeasonMatchesErrorMessage.value = ''
}

async function loadPage() {
  loading.value = true
  errorMessage.value = ''

  try {
    const [overviewResponse, rankingsResponse, liveMatchesResponse, roundsResponse, publicSystemConfig] = await Promise.all([
      getOverview({ mode: 'live', season: currentSeason, roundNumber: null }),
      getRankings({ mode: 'live', season: currentSeason, roundNumber: null }),
      getMatches({ mode: 'live', season: currentSeason, roundNumber: null }),
      getAvailableRounds(currentSeason),
      getPublicSystemConfig(),
    ])

    const guideReferenceRoundNumber = resolveHomeGuideReferenceRoundNumber(roundsResponse)
    const guideRankingsResponse = guideReferenceRoundNumber === null
      ? null
      : await getRankings({ mode: 'round', season: currentSeason, roundNumber: guideReferenceRoundNumber })

    overview.value = overviewResponse
    rankings.value = rankingsResponse
    liveMatches.value = liveMatchesResponse.matches
    rounds.value = roundsResponse
    guideRankings.value = guideRankingsResponse
    publicConfig.value = publicSystemConfig
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '首页数据加载失败，请稍后重试。')
  } finally {
    loading.value = false
  }
}

async function loadSupportData() {
  supportLoading.value = true
  supportErrorMessage.value = ''
  hasAuthToken.value = resolveHomeHasAuthToken(getAccessToken())

  try {
    supportTeams.value = await listSupportTeams()

    if (!hasAuthToken.value) {
      supportProfile.value = null
      return
    }

    supportProfile.value = await getSupportProfile()
    selectedFavoriteTeamId.value = supportProfile.value.favorite_team?.team_id ?? supportTeams.value[0]?.team_id ?? null
  } catch (error) {
    const message = extractApiErrorMessage(error, '助力入口加载失败，请稍后重试。')

    if (
      message.includes('401')
      || message.includes('未登录')
      || message.includes('Unauthorized')
      || message.includes('not logged in')
    ) {
      supportProfile.value = null
      supportErrorMessage.value = ''
      return
    }

    supportErrorMessage.value = message
  } finally {
    supportLoading.value = false
  }
}

async function loadCurrentAiUser() {
  hasAuthToken.value = resolveHomeHasAuthToken(getAccessToken())

  if (!hasAuthToken.value) {
    currentAiUser.value = null
    return
  }

  try {
    currentAiUser.value = await getCurrentUser()
  } catch {
    currentAiUser.value = null
  }
}

async function ensureAiUser(): Promise<CurrentUser | null> {
  hasAuthToken.value = resolveHomeHasAuthToken(getAccessToken())

  if (!hasAuthToken.value) {
    return null
  }

  if (currentAiUser.value) {
    return currentAiUser.value
  }

  await loadCurrentAiUser()
  return currentAiUser.value
}

function promptAiLogin() {
  uni.showModal({
    title: '先登录再聊天',
    content: '登录后才可以和小罗继续对话，现在去“我的”页登录吗？',
    confirmText: '去登录',
    success: ({ confirm }) => {
      if (!confirm) {
        return
      }

      uni.switchTab({
        url: '/pages/user/index',
      })
    },
  })
}

function handleSupportLogin() {
  rememberPostLoginRedirect({
    type: 'switchTab',
    url: '/pages/home/index',
  })
  uni.switchTab({
    url: '/pages/user/index',
  })
}

function openFavoriteTeamSheet() {
  selectedFavoriteTeamId.value = supportFavoriteTeam.value?.team_id ?? supportTeams.value[0]?.team_id ?? null
  favoriteTeamSheetVisible.value = true
}

function closeFavoriteTeamSheet() {
  favoriteTeamSheetVisible.value = false
}

async function handleConfirmFavoriteTeam() {
  if (!selectedFavoriteTeamId.value) {
    uni.showToast({ title: '请先选择一支主队', icon: 'none' })
    return
  }

  try {
    await setFavoriteTeam({ team_id: selectedFavoriteTeamId.value })
    favoriteTeamSheetVisible.value = false
    uni.showToast({ title: '主队已更新', icon: 'success' })
    await loadSupportData()
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '主队设置失败'), icon: 'none' })
  }
}

function openSupportMatch() {
  if (!supportNextMatch.value) {
    return
  }

  uni.navigateTo({
    url: `/pages/support/index?matchId=${supportNextMatch.value.match_id}`,
  })
}

async function handleAiEntryTap() {
  const tapResult = resolveHomeAiEntryTapResult({
    expanded: aiEntryExpanded.value,
    hasAuthToken: hasAuthToken.value,
  })

  if (tapResult === 'expand') {
    aiEntryExpanded.value = true
    return
  }

  if (tapResult === 'prompt-login') {
    aiEntryExpanded.value = false
    promptAiLogin()
    return
  }

  const user = await ensureAiUser()
  if (!user) {
    aiEntryExpanded.value = false
    promptAiLogin()
    return
  }

  aiEntryExpanded.value = false
  aiChatVisible.value = true
}

function handleCloseAiChat() {
  aiChatVisible.value = false
  aiEntryExpanded.value = false
}

function collapseAiEntry() {
  aiEntryExpanded.value = false
}

onShow(() => {
  reportPageActivity('home')
  hasAuthToken.value = resolveHomeHasAuthToken(getAccessToken())
  void loadPage()
  void loadCurrentAiUser()
  void loadSupportData()
})
</script>

<style scoped lang="css">
.skeleton-stack {
  display: grid;
  gap: 24rpx;
}
.skeleton-panel {
  position: relative;
  overflow: hidden;
}
.skeleton-panel::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.62) 48%,
    rgba(255, 255, 255, 0) 100%
  );
  transform: translateX(-100%);
  animation: skeleton-shimmer 1.35s ease-in-out infinite;
  pointer-events: none;
}
.skeleton-copy-group {
  display: grid;
  gap: 12rpx;
  width: 100%;
}
.skeleton-line,
.skeleton-pill,
.skeleton-button {
  background: linear-gradient(180deg, rgba(242, 244, 248, 0.98), rgba(233, 237, 243, 0.92));
  border: 2rpx solid rgba(230, 233, 239, 0.95);
}
.skeleton-line {
  border-radius: 999rpx;
}
.skeleton-line--kicker {
  width: 168rpx;
  height: 24rpx;
}
.skeleton-line--title {
  width: 430rpx;
  max-width: 100%;
  height: 56rpx;
  border-radius: 24rpx;
}
.skeleton-line--section {
  width: 320rpx;
  max-width: 100%;
  height: 40rpx;
  border-radius: 24rpx;
}
.skeleton-line--meta {
  margin-top: 18rpx;
  height: 24rpx;
}
.skeleton-line--meta-wide {
  width: 72%;
}
.skeleton-line--body {
  width: 88%;
  height: 26rpx;
  margin-top: 18rpx;
}
.skeleton-line--body-short {
  width: 56%;
}
.skeleton-pill {
  width: 150rpx;
  height: 52rpx;
  border-radius: 999rpx;
  flex-shrink: 0;
}
.skeleton-pill--short {
  width: 120rpx;
}
.skeleton-button {
  width: 100%;
  height: 72rpx;
  margin-top: 18rpx;
  border-radius: 999rpx;
}

.skeleton-line--hero-title {
  width: 560rpx;
  max-width: 100%;
  height: 64rpx;
  border-radius: 24rpx;
}
.skeleton-hero-guide {
  margin-top: 24rpx;
  padding-top: 24rpx;
  border-top: 2rpx solid #ececf1;
}
.skeleton-line--guide-title {
  width: 220rpx;
  height: 36rpx;
  margin-bottom: 12rpx;
}

.skeleton-briefing-grid {
  margin-top: 20rpx;
  display: grid;
  gap: 10rpx;
}
.skeleton-briefing-body {
  margin-top: 18rpx;
  display: grid;
  gap: 12rpx;
}
.skeleton-line--briefing-value {
  width: 260rpx;
  height: 32rpx;
}
.skeleton-line--briefing-metric {
  width: 180rpx;
  height: 24rpx;
}

.skeleton-score-strip {
  padding: 24rpx 22rpx;
  display: grid;
  gap: 14rpx;
  border-radius: 28rpx;
  border: 2rpx solid #ececf1;
  background: linear-gradient(180deg, rgba(249, 249, 252, 0.9), rgba(255, 255, 255, 1));
  margin-top: 16rpx;
}
.skeleton-score-body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  gap: 18rpx;
}
.skeleton-line--score-team {
  width: 100%;
  height: 30rpx;
}
.skeleton-line--score-value {
  width: 120rpx;
  height: 60rpx;
  border-radius: 20rpx;
}

.skeleton-ranking-row {
  gap: 16rpx;
  align-items: center;
  padding: 12rpx 10rpx;
}
.skeleton-line--rank {
  width: 72rpx;
  height: 24rpx;
}
.skeleton-line--avatar {
  width: 68rpx;
  height: 68rpx;
  border-radius: 999rpx;
}
.skeleton-ranking-body {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: 8rpx;
}
.skeleton-line--name {
  width: 240rpx;
  height: 30rpx;
}
.skeleton-line--note {
  width: 160rpx;
  height: 22rpx;
}
.skeleton-line--points {
  width: 72rpx;
  height: 36rpx;
  justify-self: end;
}

@keyframes skeleton-shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

.home-page-shell {
  min-height: 100vh;
  position: relative;
}

.page-scroll {
  height: 100vh;
  position: relative;
  z-index: 1;
}

.page {
  padding: 24rpx 16rpx 40rpx;
  display: flex;
  flex-direction: column;
  gap: 16rpx;
}

.hero-card,
.panel,
.state-card {
  background: rgba(255, 255, 255, 0.94);
  border-radius: 36rpx;
  border: 2rpx solid rgba(236, 236, 241, 0.95);
  box-shadow: 0 20rpx 48rpx rgba(26, 28, 36, 0.06);
}

 .hero-card,
.panel,
.state-card {
  padding: 20rpx;
}

.hero-card--home {
  padding: 24rpx 22rpx 22rpx;
  border-radius: 40rpx;
  background:
    radial-gradient(circle at top center, rgba(255, 255, 255, 0.98), rgba(250, 250, 252, 0.92) 42%, rgba(247, 248, 251, 0.95) 100%),
    rgba(255, 255, 255, 0.96);
}

.hero-card__top,
.section-heading,
.match-card__meta,
.ranking-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.hero-card__top {
  align-items: flex-start;
  gap: 16rpx;
}

.hero-card__heading {
  display: flex;
  flex-direction: column;
  gap: 10rpx;
  flex: 1;
  min-width: 0;
}

.eyebrow,
.section-kicker {
  display: block;
  margin: 0;
  color: #8f9198;
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 3rpx;
}

.hero-card__title,
.section-title,
.story-card__headline {
  color: #121212;
  font-weight: 800;
}

.hero-card__title {
  max-width: none;
  font-size: 48rpx;
  line-height: 1.08;
  letter-spacing: -0.02em;
  color: #2a2c31;
}

.hero-card__badge,
.meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  flex-shrink: 0;
  white-space: nowrap;
  line-height: 1;
  box-sizing: border-box;
  padding: 18rpx 24rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(229, 223, 205, 0.92);
  background: linear-gradient(180deg, rgba(252, 250, 245, 0.98), rgba(247, 243, 232, 0.94));
  color: #93876a;
  font-size: 24rpx;
}

.meta-note {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  flex: 0 0 auto;
  flex-shrink: 0;
  max-width: 100%;
  white-space: nowrap;
  line-height: 1;
  color: #a3916f;
  font-size: 24rpx;
  font-weight: 700;
  letter-spacing: 1rpx;
  padding: 8rpx 0 0;
}

.meta-note::before {
  content: '';
  width: 12rpx;
  height: 12rpx;
  margin-right: 10rpx;
  border-radius: 999rpx;
  background: linear-gradient(180deg, rgba(214, 184, 131, 0.9), rgba(197, 163, 103, 0.72));
  box-shadow: 0 0 0 6rpx rgba(214, 184, 131, 0.14);
}

.hero-card__guide {
  margin-top: 28rpx;
  padding-top: 28rpx;
  border-top: 2rpx solid #ececf1;
}

.hero-card__guide-title {
  color: #121212;
  font-size: 40rpx;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 10rpx;
}

.hero-card__guide-title::before {
  content: '';
  width: 4rpx;
  height: 32rpx;
  background: linear-gradient(180deg, #f97316, #ea580c);
  border-radius: 999rpx;
}

.hero-card__guide-copy {
  margin-top: 14rpx;
  color: #6b707b;
  font-size: 28rpx;
  line-height: 1.65;
}

.hero-card__guide-note {
  margin-top: 10rpx;
  color: #8f9198;
  font-size: 22rpx;
  line-height: 1.55;
}

.ai-entry {
  position: relative;
  display: block;
  width: 54rpx;
  height: 54rpx;
  flex: 0 0 54rpx;
  flex-shrink: 0;
  min-height: 54rpx;
  overflow: visible;
}

.ai-entry--pressed {
  transform: scale(0.97);
}

.ai-entry__bubble {
  position: absolute;
  right: calc(100% + 12rpx);
  top: 50%;
  z-index: 3;
  max-width: 0;
  opacity: 0;
  overflow: hidden;
  display: inline-flex;
  align-items: center;
  gap: 8rpx;
  padding: 0;
  border-radius: 999rpx;
  border: 2rpx solid rgba(221, 225, 234, 0);
  background: rgba(255, 255, 255, 0.52);
  box-shadow: 0 14rpx 28rpx rgba(31, 37, 45, 0.07);
  backdrop-filter: blur(20rpx);
  white-space: nowrap;
  transform: translate(16rpx, -50%);
  transform-origin: right center;
  transition:
    max-width 240ms ease,
    opacity 180ms ease,
    transform 240ms ease,
    padding 240ms ease,
    border-color 240ms ease;
}

.ai-entry--expanded .ai-entry__bubble {
  max-width: 256rpx;
  opacity: 1;
  padding: 14rpx 18rpx;
  border-color: rgba(221, 225, 234, 0.96);
  transform: translate(0, -50%);
}

.ai-entry__bubble-text {
  color: #48505b;
  font-size: 22rpx;
  line-height: 1;
}

.ai-entry__bubble-arrow {
  color: #7e8794;
  font-size: 24rpx;
  line-height: 1;
}

.ai-entry__charm {
  position: relative;
  width: 54rpx;
  height: 54rpx;
}

.ai-entry__avatar-shell {
  position: relative;
  width: 54rpx;
  height: 54rpx;
  flex: 0 0 auto;
  animation: ai-entry-float 3.2s ease-in-out infinite;
}

.ai-entry__avatar-glow {
  position: absolute;
  inset: -6rpx;
  border-radius: 999rpx;
  background: radial-gradient(circle, rgba(255, 213, 106, 0.44), rgba(255, 213, 106, 0));
  opacity: 0.88;
}

.ai-entry__avatar {
  position: relative;
  width: 100%;
  height: 100%;
  border-radius: 999rpx;
  border: 2rpx solid rgba(255, 255, 255, 0.94);
  box-shadow: 0 10rpx 18rpx rgba(25, 29, 36, 0.12);
}

.ai-entry__status {
  position: absolute;
  right: -3rpx;
  bottom: -3rpx;
  width: 20rpx;
  height: 20rpx;
  border-radius: 999rpx;
  background: rgba(255, 255, 255, 0.94);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 8rpx 16rpx rgba(24, 27, 33, 0.12);
}

.ai-entry__status-dot {
  width: 9rpx;
  height: 9rpx;
  border-radius: 999rpx;
  background: #1db35f;
  box-shadow: 0 0 0 0 rgba(29, 179, 95, 0.45);
  animation: ai-entry-status-pulse 1.8s ease-out infinite;
}

.ai-entry__tag {
  position: absolute;
  left: 42rpx;
  top: 50%;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6rpx 10rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(255, 255, 255, 0.6);
  background: rgba(255, 255, 255, 0.42);
  backdrop-filter: blur(14rpx);
  transform: translateY(-50%);
  box-shadow: 0 10rpx 20rpx rgba(31, 37, 45, 0.05);
  pointer-events: none;
}

.ai-entry__tag-text {
  color: #6f735f;
  font-size: 16rpx;
  line-height: 1;
}

.inline-highlight {
  display: inline;
  color: #121212;
  font-weight: 700;
  padding: 0 0.08em;
  background-image: linear-gradient(90deg, rgba(255, 132, 20, 0.98) 0%, rgba(255, 161, 32, 0.98) 56%, rgba(255, 205, 74, 0.94) 100%);
  background-repeat: no-repeat;
  background-size: 100% 0.42em;
  background-position: left bottom;
}

.briefing-grid {
  margin-top: 20rpx;
  display: grid;
  gap: 10rpx;
}

.briefing-card {
  position: relative;
  overflow: hidden;
  min-height: 240rpx;
  padding: 18rpx 14rpx 18rpx 28rpx;
  border-radius: 28rpx;
  border: 2rpx solid #ececf1;
}

.briefing-card--leader {
  background:
    radial-gradient(circle at 84% 22%, rgba(234, 88, 12, 0.10), transparent 36%),
    linear-gradient(135deg, rgba(255, 247, 237, 0.98), rgba(255, 255, 255, 0.96) 52%, rgba(255, 250, 245, 0.98) 100%);
}

.briefing-card--scorer {
  background:
    radial-gradient(circle at 82% 24%, rgba(249, 115, 22, 0.12), transparent 40%),
    linear-gradient(135deg, rgba(255, 247, 237, 0.98), rgba(255, 255, 255, 0.96) 52%, rgba(255, 250, 245, 0.98) 100%);
}

.briefing-card--assist {
  background:
    radial-gradient(circle at 82% 24%, rgba(245, 158, 11, 0.12), transparent 40%),
    linear-gradient(135deg, rgba(255, 251, 235, 0.98), rgba(255, 255, 255, 0.96) 52%, rgba(255, 250, 245, 0.98) 100%);
}

.briefing-card::before {
  content: '';
  position: absolute;
  inset: 0 auto 0 0;
  width: 10rpx;
  border-radius: 28rpx 0 0 28rpx;
}

.briefing-card--leader::before {
  background: linear-gradient(180deg, #ea580c 0%, #c2410c 100%);
}

.briefing-card--scorer::before {
  background: linear-gradient(180deg, #f97316 0%, #ea580c 100%);
}

.briefing-card--assist::before {
  background: linear-gradient(180deg, #f59e0b 0%, #d97706 100%);
}

.briefing-card__label {
  color: #8f9198;
  font-size: 24rpx;
}

.briefing-card__body {
  height: calc(100% - 32rpx);
  display: grid;
  grid-template-columns: minmax(188rpx, 0.9fr) minmax(0, 1.36fr) 124rpx;
  column-gap: 10rpx;
  align-items: center;
}

.briefing-card__main {
  display: grid;
  align-content: start;
  min-width: 0;
}

.briefing-card__leader-logo,
.briefing-card__entity-group {
  min-height: 78rpx;
  margin-bottom: 4rpx;
}

.briefing-card__leader-logo {
  display: flex;
  align-items: center;
}

.briefing-card__leader-logo-image {
  width: 76rpx;
  height: 76rpx;
}

.briefing-card__entity-group {
  display: flex;
  align-items: center;
}

.briefing-card__entity-avatar {
  width: 66rpx;
  height: 66rpx;
  border-radius: 999rpx;
  margin-right: -12rpx;
  border: 4rpx solid rgba(255, 255, 255, 0.96);
  background: #ffffff;
}

.briefing-card__entity-avatar:last-child {
  margin-right: 0;
}

.briefing-card__title-block {
  display: grid;
  gap: 6rpx;
}

.briefing-card__value {
  color: #121212;
  font-size: 32rpx;
  line-height: 1.04;
  font-weight: 800;
}

.briefing-card__subvalue {
  color: #8f9198;
  font-size: 20rpx;
  line-height: 1.22;
}

.briefing-card__marquees {
  display: grid;
  align-self: stretch;
  align-content: center;
  gap: 4rpx;
  overflow: hidden;
  min-width: 0;
  padding-right: 0;
}

.briefing-card__marquee {
  overflow: hidden;
  white-space: nowrap;
}

.briefing-card__marquee-track {
  display: inline-flex;
  align-items: center;
  gap: 24rpx;
  min-width: max-content;
  animation-name: briefing-marquee-scroll;
  animation-timing-function: linear;
  animation-iteration-count: infinite;
}

.briefing-card__marquee-item {
  position: relative;
  padding-left: 16rpx;
  color: rgba(84, 89, 99, 0.92);
  font-size: 22rpx;
  line-height: 1.28;
}

.briefing-card__marquee-item::before {
  content: '•';
  position: absolute;
  left: 0;
  color: rgba(255, 138, 24, 0.92);
  font-size: 24rpx;
}

.briefing-card__metric {
  display: grid;
  width: 124rpx;
  align-content: center;
  justify-items: end;
  justify-self: end;
  align-self: stretch;
  text-align: right;
  gap: 8rpx;
}

.briefing-card__metric-value {
  color: #f97316;
  font-size: 58rpx;
  line-height: 0.92;
  font-weight: 800;
}

.briefing-card__metric-label {
  color: #121212;
  font-size: 24rpx;
  font-weight: 700;
  line-height: 1.18;
  background-image: linear-gradient(to top, #ffb347 0, #ffb347 0.4em, transparent 0.4em, transparent 100%);
}

@keyframes briefing-marquee-scroll {
  from {
    transform: translateX(0);
  }

  to {
    transform: translateX(-50%);
  }
}

.story-card__headline,
.section-title {
  display: block;
  font-size: 44rpx;
  line-height: 1.16;
}

.section-heading {
  align-items: flex-start;
  gap: 12rpx;
}

.section-heading > view {
  display: grid;
  gap: 8rpx;
  min-width: 0;
}

.story-card__copy {
  display: block;
  margin-top: 18rpx;
  color: #555863;
  font-size: 28rpx;
  line-height: 1.65;
}

.score-strip-stack {
  margin-top: 16rpx;
  display: grid;
  gap: 12rpx;
}

.score-strip {
  padding: 18rpx 16rpx;
  display: grid;
  gap: 10rpx;
  border-radius: 28rpx;
  border: 2rpx solid #ececf1;
  background: linear-gradient(180deg, rgba(249, 249, 252, 0.9), rgba(255, 255, 255, 1));
}

.score-strip--interactive {
  background:
    radial-gradient(circle at top right, rgba(255, 145, 41, 0.12), transparent 32%),
    linear-gradient(180deg, rgba(249, 249, 252, 0.92), rgba(255, 255, 255, 1));
  border-color: rgba(255, 145, 41, 0.2);
}

.score-strip--pressed {
  transform: scale(0.992);
}

.score-strip__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18rpx;
}

.score-strip__meta-trailing {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12rpx;
  flex-wrap: wrap;
}

.score-strip__meta text {
  color: #8f9198;
  font-size: 22rpx;
}

.score-strip__meta-pill {
  padding: 10rpx 16rpx;
  border-radius: 999rpx;
  background: rgba(255, 145, 41, 0.12);
  color: #d66b10 !important;
  font-weight: 700;
}

.score-strip__body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  gap: 18rpx;
}

.score-strip__team {
  min-width: 0;
}

.score-strip__team--away {
  text-align: right;
}

.score-strip__team-name {
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 30rpx;
  font-weight: 700;
}

.score-strip__score {
  color: #121212;
  font-size: 60rpx;
  line-height: 0.92;
  font-weight: 800;
}

.score-strip__hint {
  display: flex;
  justify-content: flex-end;
}

.score-strip__hint-text {
  color: #d66b10;
  font-size: 22rpx;
  line-height: 1.2;
  font-weight: 700;
}

.tech-stats-sheet {
  max-height: 72vh;
  transform-origin: center bottom;
  animation: tech-stats-sheet-enter 280ms cubic-bezier(0.2, 0.9, 0.22, 1) both;
  padding-bottom: calc(40rpx + env(safe-area-inset-bottom) + 100rpx);
}

.tech-stats-sheet__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: 12rpx 18rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
  color: #6d7280;
  font-size: 24rpx;
}

.tech-stats-sheet__summary {
  margin-top: 18rpx;
  padding: 24rpx;
  border-radius: 28rpx;
  border: 2rpx solid rgba(255, 145, 41, 0.16);
  background:
    radial-gradient(circle at top right, rgba(255, 145, 41, 0.12), transparent 36%),
    linear-gradient(180deg, rgba(255,255,255,0.98), rgba(250,246,239,0.92));
}

.tech-stats-sheet__teams {
  display: block;
  color: #121212;
  font-size: 32rpx;
  line-height: 1.4;
  font-weight: 800;
}

.tech-stats-sheet__meta {
  display: block;
  margin-top: 10rpx;
  color: #8f9198;
  font-size: 24rpx;
}

.tech-stats-sheet__list {
  margin-top: 24rpx;
  display: grid;
  gap: 18rpx;
}

.tech-stat-row {
  display: grid;
  grid-template-columns: 44rpx minmax(0, 1fr) auto minmax(0, 1fr) 44rpx;
  align-items: center;
  gap: 16rpx;
  padding: 18rpx 0;
  border-top: 2rpx solid #f0f1f5;
  opacity: 0;
  transform: translateY(14rpx);
  animation: tech-stat-row-enter 320ms cubic-bezier(0.24, 0.88, 0.28, 1) both;
  animation-delay: var(--tech-stat-delay, 120ms);
}

.tech-stat-row:first-child {
  border-top: none;
}

.tech-stat-row__value {
  color: #121212;
  font-size: 28rpx;
  font-weight: 800;
  text-align: left;
}

.tech-stat-row__value--away {
  text-align: right;
}

.tech-stat-row__label {
  min-width: 84rpx;
  color: #2a2c31;
  font-size: 30rpx;
  font-weight: 800;
  text-align: center;
}

.tech-stat-row__track {
  height: 16rpx;
  border-radius: 999rpx;
  background: #16171c;
  overflow: hidden;
  display: flex;
  align-items: center;
}

.tech-stat-row__track--home {
  justify-content: flex-end;
}

.tech-stat-row__fill {
  height: 100%;
  border-radius: 999rpx;
  background: linear-gradient(90deg, #ff8b2b, #f59e0b);
  transform: scaleX(0);
  animation: tech-stat-fill-grow 480ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
  animation-delay: calc(var(--tech-stat-delay, 120ms) + 70ms);
}

.tech-stat-row__fill--home {
  background: linear-gradient(90deg, #f6b14b, #f08a12);
  transform-origin: right center;
}

.tech-stat-row__fill--away {
  background: linear-gradient(90deg, #f08a12, #f6b14b);
  transform-origin: left center;
}

.tech-stats-sheet__footnote {
  display: block;
  margin-top: 24rpx;
  color: #8f9198;
  font-size: 24rpx;
  line-height: 1.6;
}

.team-season-sheet__summary {
  margin-top: 18rpx;
  padding: 24rpx;
  border-radius: 28rpx;
  border: 2rpx solid rgba(255, 145, 41, 0.16);
  background:
    radial-gradient(circle at top right, rgba(255, 145, 41, 0.12), transparent 36%),
    linear-gradient(180deg, rgba(255,255,255,0.98), rgba(250,246,239,0.92));
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20rpx;
}

.team-season-sheet__summary-main {
  display: flex;
  align-items: center;
  gap: 18rpx;
  min-width: 0;
}

.team-season-sheet__summary-avatar {
  width: 82rpx;
  height: 82rpx;
  flex: 0 0 auto;
}

.team-season-sheet__summary-copy {
  min-width: 0;
  display: grid;
  gap: 8rpx;
}

.team-season-sheet__summary-name {
  color: #121212;
  font-size: 32rpx;
  line-height: 1.2;
  font-weight: 800;
}

.team-season-sheet__summary-meta,
.team-season-sheet__summary-record {
  color: #8f9198;
  font-size: 24rpx;
}

.team-season-sheet__summary-record {
  flex: 0 0 auto;
  text-align: right;
  font-weight: 700;
}

.team-season-sheet__list {
  margin-top: 22rpx;
  max-height: 52vh;
}

.team-season-sheet__state,
.team-season-sheet__empty {
  margin-top: 22rpx;
}

.team-season-sheet__empty {
  padding: 28rpx 12rpx;
  color: #8f9198;
  font-size: 26rpx;
  text-align: center;
}

.team-season-match-row {
  padding: 20rpx 0;
  border-top: 2rpx solid #f0f1f5;
  display: grid;
  gap: 14rpx;
  opacity: 0;
  transform: translateY(14rpx);
  animation: team-season-row-enter 320ms cubic-bezier(0.24, 0.88, 0.28, 1) both;
  animation-delay: var(--team-match-delay, 100ms);
}

.team-season-match-row:first-child {
  border-top: none;
}

.team-season-match-row__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
  color: #8f9198;
  font-size: 22rpx;
}

.team-season-match-row__result {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 96rpx;
  padding: 8rpx 16rpx;
  border-radius: 999rpx;
  font-size: 22rpx;
  font-weight: 800;
}

.team-season-match-row__result--win {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.team-season-match-row__result--draw {
  background: rgba(148, 163, 184, 0.16);
  color: #475569;
}

.team-season-match-row__result--loss {
  background: rgba(239, 68, 68, 0.12);
  color: #b91c1c;
}

.team-season-match-row__result--live {
  background: rgba(249, 115, 22, 0.12);
  color: #d97706;
}

.team-season-match-row__result--scheduled {
  background: rgba(59, 130, 246, 0.12);
  color: #2563eb;
}

.team-season-match-row__body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  gap: 18rpx;
}

.team-season-match-row__team {
  color: #7b818d;
  font-size: 28rpx;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.team-season-match-row__team--active {
  color: #121212;
}

.team-season-match-row__team--away {
  text-align: right;
}

.team-season-match-row__score {
  color: #121212;
  font-size: 40rpx;
  line-height: 1;
  font-weight: 800;
}

.watch-list {
  margin-top: 16rpx;
  display: grid;
  gap: 12rpx;
}

.watch-list__item {
  position: relative;
  padding-left: 26rpx;
  color: #151515;
  font-size: 26rpx;
  line-height: 1.55;
}

.watch-list__item::before {
  content: '';
  position: absolute;
  top: 14rpx;
  left: 0;
  width: 10rpx;
  height: 10rpx;
  border-radius: 999rpx;
  background: #131313;
}

.ranking-list,
.match-stack {
  margin-top: 12rpx;
  display: grid;
  gap: 10rpx;
}

.ranking-row {
  gap: 16rpx;
  align-items: center;
}

.ranking-row--interactive {
  padding: 10rpx 8rpx;
  border-radius: 24rpx;
  transition: transform 180ms ease, background-color 180ms ease;
}

.ranking-row--pressed {
  transform: scale(0.992);
  background: rgba(255, 145, 41, 0.08);
}

.ranking-row__rank {
  width: 72rpx;
  color: #8f9198;
  font-size: 24rpx;
  font-weight: 700;
}

.ranking-row__rank--1 {
  color: #dc2626;
}

.ranking-row__rank--2 {
  color: #2563eb;
}

.ranking-row__rank--3 {
  color: #16a34a;
}

.ranking-row__avatar {
  width: 68rpx;
  height: 68rpx;
  border-radius: 999rpx;
  background: #f5f6fa;
}

.ranking-row__avatar--player {
  object-fit: cover;
}

.ranking-row__body,
.ranking-row__metric {
  display: grid;
}

.ranking-row__body {
  flex: 1;
  min-width: 0;
}

.ranking-row__metric {
  justify-items: end;
}

.ranking-row__name {
  color: #121212;
  font-size: 30rpx;
  font-weight: 700;
}

.ranking-row__note,
.ranking-row__metric-note,
.match-card__meta text {
  color: #8f9198;
  font-size: 22rpx;
}

.ranking-row__metric-value {
  color: #121212;
  font-size: 36rpx;
  font-weight: 800;
}

.match-card {
  padding: 22rpx;
  border-radius: 28rpx;
  border: 2rpx solid #ececf1;
  background: linear-gradient(180deg, rgba(249, 249, 252, 0.9), rgba(255, 255, 255, 1));
}

.match-card__teams {
  margin-top: 12rpx;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: 18rpx;
  align-items: center;
}

.match-card__team-name {
  color: #151515;
  font-size: 28rpx;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.match-card__team-name--away {
  text-align: right;
}

.match-card__score {
  color: #121212;
  font-size: 56rpx;
  font-weight: 800;
  line-height: 0.92;
}

.state-card--error text {
  font-size: 28rpx;
  color: #c03a2b;
}
.support-home-panel__summary {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: 28rpx;
  line-height: 1.7;
}
.support-home-panel__header {
  display: grid;
  gap: 14rpx;
}
.support-home-panel__heading {
  display: grid;
  gap: 8rpx;
}
.support-home-panel__context {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8rpx;
  padding-top: 14rpx;
  border-top: 2rpx solid rgba(236, 236, 241, 0.92);
}
.support-home-panel__context-dot {
  width: 10rpx;
  height: 10rpx;
  border-radius: 999rpx;
  background: linear-gradient(180deg, #f6b44e, #d89b34);
  box-shadow: 0 0 0 6rpx rgba(216, 155, 52, 0.14);
}
.support-home-panel__context-label {
  color: #9c7e45;
  font-size: 24rpx;
  font-weight: 700;
  line-height: 1.2;
}
.support-home-panel__context-note {
  color: #8f9198;
  font-size: 22rpx;
  line-height: 1.2;
}
.support-home-panel__action {
  margin-top: 18rpx;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 18rpx 30rpx;
  border-radius: 999rpx;
  background: #15161b;
  color: #ffffff;
  font-size: 26rpx;
  line-height: 1;
}
.support-home-panel__action--ghost {
  background: #f5f6fa;
  color: #5f6673;
}
.support-home-panel__favorite {
  margin-top: 18rpx;
  display: flex;
  align-items: center;
  gap: 16rpx;
}
.support-home-panel__favorite-avatar,
.favorite-team-sheet__avatar {
  width: 72rpx;
  height: 72rpx;
}
.support-home-panel__favorite-body,
.favorite-team-sheet__body {
  flex: 1;
  min-width: 0;
}
.support-home-panel__favorite-name,
.favorite-team-sheet__name {
  color: #121212;
  font-size: 30rpx;
  font-weight: 700;
}
.support-home-panel__favorite-note,
.favorite-team-sheet__note {
  display: block;
  margin-top: 6rpx;
  color: #8f9198;
  font-size: 22rpx;
}
.support-home-panel__switch {
  padding: 14rpx 20rpx;
  border-radius: 999rpx;
  background: #f5f6fa;
  color: #5f6673;
  font-size: 22rpx;
  line-height: 1;
}
.support-home-match-card {
  margin-top: 18rpx;
  padding: 22rpx;
  border-radius: 28rpx;
  border: 2rpx solid rgba(255, 140, 43, 0.22);
  background:
    radial-gradient(circle at top right, rgba(255, 145, 41, 0.12), transparent 34%),
    linear-gradient(180deg, rgba(255,255,255,0.98), rgba(250,246,239,0.96));
}
.support-home-match-card__meta,
.support-home-match-card__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: #8f9198;
  font-size: 22rpx;
}
.support-home-match-card__teams {
  margin-top: 14rpx;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: 14rpx;
  align-items: center;
}
.support-home-match-card__team {
  color: #121212;
  font-size: 30rpx;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.support-home-match-card__team--away {
  text-align: right;
}
.support-home-match-card__vs {
  color: #f97316;
  font-size: 22rpx;
  font-weight: 700;
}
.support-home-match-card__bar {
  margin-top: 16rpx;
  height: 14rpx;
  border-radius: 999rpx;
  overflow: hidden;
  background: #ececf1;
  display: flex;
}
.support-home-match-card__bar-home {
  height: 100%;
  background: linear-gradient(90deg, #ff8b2b, #ffb347);
}
.support-home-match-card__bar-away {
  height: 100%;
  background: linear-gradient(90deg, #4d8dff, #7bb4ff);
}
.sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 40;
  background: rgba(18, 20, 28, 0.36);
  backdrop-filter: blur(8rpx);
  display: flex;
  align-items: flex-end;
}
.sheet-mask--tech-stats {
  animation: tech-stats-mask-fade 220ms ease-out both;
}
.sheet-card {
  width: 100%;
  max-height: 78vh;
  border-radius: 36rpx 36rpx 0 0;
  background: rgba(255,255,255,0.98);
  padding: 28rpx 24rpx 40rpx;
  box-shadow: 0 -24rpx 56rpx rgba(12,14,20,0.12);
  overflow-y: auto;
}
.favorite-team-sheet__list {
  margin-top: 20rpx;
  max-height: 52vh;
}
.favorite-team-sheet__row {
  padding: 18rpx 0;
  display: flex;
  align-items: center;
  gap: 16rpx;
  border-bottom: 2rpx solid #f0f1f5;
}
.favorite-team-sheet__row--active {
  background: rgba(255, 145, 41, 0.08);
}
.sheet-actions {
  margin-top: 24rpx;
  display: flex;
  gap: 16rpx;
}
.primary-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: 20rpx 30rpx;
  border-radius: 999rpx;
  background: #15161b;
  color: #ffffff;
  font-size: 28rpx;
  white-space: nowrap;
  line-height: 1;
}
.primary-action--ghost {
  background: #f6f7fb;
  color: #6d7280;
}

@keyframes ai-entry-float {
  0%, 100% {
    transform: translateY(0) rotate(-1.5deg);
  }
  50% {
    transform: translateY(-4rpx) rotate(1.5deg);
  }
}

@keyframes ai-entry-status-pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(29, 179, 95, 0.45);
  }
  100% {
    box-shadow: 0 0 0 14rpx rgba(29, 179, 95, 0);
  }
}

@keyframes tech-stats-mask-fade {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes tech-stats-sheet-enter {
  from {
    opacity: 0;
    transform: translateY(32rpx) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes tech-stat-row-enter {
  from {
    opacity: 0;
    transform: translateY(14rpx);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes tech-stat-fill-grow {
  from {
    transform: scaleX(0);
  }
  to {
    transform: scaleX(1);
  }
}

@keyframes team-season-row-enter {
  from {
    opacity: 0;
    transform: translateY(14rpx);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
.page-bg-img {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 600rpx;
  pointer-events: none;
  z-index: 0;
}
.page-bg-fade {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 600rpx;
  background: linear-gradient(180deg, transparent 45%, rgba(247,248,250,0.55) 78%, #f7f8fa 100%);
  pointer-events: none;
  z-index: 0;
}

</style>

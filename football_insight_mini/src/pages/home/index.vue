<template>
  <view class="home-page-shell">
    <FiBrandNav open-on-current-page @open-ai="openAiFromBrandNav" />
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" />
    <view class="page-bg-fade"></view>
    <view class="page-scroll">
      <view class="page">
        <view class="hero-card hero-card--home">
        <image class="page-hero-dots page-hero-dots--top" :src="memberCardDotsImage" mode="aspectFill" />
        <image class="page-hero-dots page-hero-dots--bottom" :src="memberCardDotsImage" mode="aspectFill" />
        <view class="hero-card__top">
          <view class="hero-card__heading">
            <text class="eyebrow">Football Insight</text>
            <text class="hero-card__title">这一轮之后，谁在改变联赛格局</text>
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

                <view v-else-if="item.accent === 'leader' && item.avatars.length" class="briefing-card__entity-group">
                  <image
                    v-for="avatar in item.avatars.slice(0, 3)"
                    :key="`${item.label}-${avatar.name}`"
                    :src="avatar.src || ''"
                    :alt="avatar.name"
                    mode="aspectFill"
                    class="briefing-card__entity-avatar"
                  />
                </view>

                <view v-if="item.entities.length" class="briefing-card__entity-list">
                  <view
                    v-for="entity in item.entities"
                    :key="`${item.label}-${entity.name}-${entity.caption}`"
                    class="briefing-card__entity-row"
                  >
                    <image
                      :src="entity.avatar || ''"
                      :alt="entity.name"
                      mode="aspectFill"
                      class="briefing-card__entity-row-avatar"
                    />
                    <view class="briefing-card__entity-row-body">
                      <text class="briefing-card__entity-row-name">{{ entity.name }}</text>
                      <text class="briefing-card__entity-row-team">{{ entity.caption }}</text>
                    </view>
                  </view>
                </view>

                <view v-else class="briefing-card__title-block">
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
            <text class="section-title support-home-panel__title">我的主队</text>
          </view>

          <view v-if="!supportFavoriteTeam" class="support-home-panel__context">
            <view class="support-home-panel__context-dot"></view>
            <text class="support-home-panel__context-label">{{ supportPanelBadge }}</text>
            <text class="support-home-panel__context-note">{{ supportPanelContextNote }}</text>
          </view>
          <text v-if="supportRefreshing" class="support-home-panel__refreshing">更新中</text>
        </view>

        <FiLoading
          v-if="showSupportLoading"
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
        </template>

        <template v-else-if="!supportFavoriteTeam">
          <text class="support-home-panel__summary">
            先选择一支主队，首页就会把它的下一场比赛和助力入口放到第一屏。
          </text>
          <button class="support-home-panel__action" @click="openFavoriteTeamSheet">选择主队</button>
        </template>

        <template v-else-if="supportNextMatch">
          <view class="support-home-panel__favorite">
            <view class="support-home-panel__favorite-avatar-shell">
              <image :src="supportFavoriteTeam.avatar_storage_url || ''" mode="aspectFit" class="support-home-panel__favorite-avatar" />
            </view>
            <view class="support-home-panel__favorite-body">
              <text class="support-home-panel__favorite-name">{{ supportFavoriteTeam.team_name }}</text>
              <text class="support-home-panel__favorite-note">{{ supportFavoriteTeamLabel }}</text>
            </view>
            <button class="support-home-panel__switch" @click="openFavoriteTeamSheet">切换主队</button>
          </view>

          <view class="support-home-match-card" @click="openSupportMatch">
            <view class="support-home-match-card__meta">
              <view class="support-home-match-card__meta-pill support-home-match-card__meta-pill--round">
                <text>第 {{ supportNextMatch.round_number }} 轮</text>
              </view>
              <view class="support-home-match-card__meta-pill support-home-match-card__meta-pill--time">
                <text v-if="supportNextMatchWeekdayLabel" class="support-home-match-card__weekday">{{ supportNextMatchWeekdayLabel }}</text>
                <view class="support-home-match-card__datetime">
                  <text>{{ supportNextMatch.match_date }}</text>
                  <text>{{ supportNextMatch.match_time }}</text>
                </view>
              </view>
            </view>
            <view class="support-home-match-card__teams">
              <view class="support-home-match-card__team-block">
                <text class="support-home-match-card__team">{{ supportNextMatch.home_team.team_name }}</text>
                <text class="support-home-match-card__rank">{{ supportNextMatchHomeRankLabel }}</text>
              </view>
              <text class="support-home-match-card__vs">{{ supportWindowShortLabel }}</text>
              <view class="support-home-match-card__team-block support-home-match-card__team-block--away">
                <text class="support-home-match-card__team support-home-match-card__team--away">{{ supportNextMatch.away_team.team_name }}</text>
                <text class="support-home-match-card__rank support-home-match-card__rank--away">{{ supportNextMatchAwayRankLabel }}</text>
              </view>
            </view>
            <view class="support-home-match-card__bar">
              <view class="support-home-match-card__bar-home" :style="{ width: `${supportNextMatch.home_team.support_share_pct}%` }" />
              <view class="support-home-match-card__bar-away" :style="{ width: `${supportNextMatch.away_team.support_share_pct}%` }" />
            </view>
            <view class="support-home-match-card__footer">
              <text>{{ supportNextMatchLabel }}</text>
              <text class="support-home-match-card__action">进入助力页</text>
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
              <image
                class="score-strip__corner"
                :src="pulseMatchCornerImage"
                mode="aspectFill"
              />
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
    </view>

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
            <view class="team-season-sheet__title">
              <image
                :src="selectedStandingsTeam.avatar_storage_url || ''"
                mode="aspectFit"
                class="team-season-sheet__title-avatar"
              />
              <text class="section-title team-season-sheet__title-name">{{ selectedStandingsTeam.team_name }}</text>
            </view>
          </view>
          <button class="tech-stats-sheet__close" @click="closeStandingsTeamSheet">关闭</button>
        </view>

        <view class="team-season-sheet__summary">
          <view class="team-season-sheet__summary-main">
            <view class="team-season-sheet__summary-meta-grid">
              <view class="team-season-sheet__summary-metric">
                <text class="team-season-sheet__summary-value">第 {{ selectedStandingsTeam.rank_no }}</text>
                <text class="team-season-sheet__summary-label">积分榜排名</text>
              </view>
              <view class="team-season-sheet__summary-metric">
                <text class="team-season-sheet__summary-value">{{ selectedStandingsTeam.points }} 分</text>
                <text class="team-season-sheet__summary-label">当前积分</text>
              </view>
              <view class="team-season-sheet__summary-metric">
                <text class="team-season-sheet__summary-value">{{ selectedStandingsTeamRecordText }}</text>
                <text class="team-season-sheet__summary-label">赛季战绩</text>
              </view>
            </view>
          </view>
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
              <view class="team-season-match-row__meta-right">
                <text class="team-season-match-row__venue" :class="match.isHomeTeam ? 'team-season-match-row__venue--home' : 'team-season-match-row__venue--away'">
                  {{ match.venueLabel }}
                </text>
                <text class="team-season-match-row__result" :class="`team-season-match-row__result--${match.resultTone}`">
                  {{ match.resultLabel }}
                </text>
              </view>
            </view>
            <view class="team-season-match-row__body">
              <view class="team-season-match-row__side team-season-match-row__side--left">
                <image class="team-season-match-row__avatar" :src="match.teamAvatar || ''" mode="aspectFit" />
                <text class="team-season-match-row__team team-season-match-row__team--active">{{ match.teamName }}</text>
              </view>
              <text class="team-season-match-row__score">{{ match.scoreText }}</text>
              <view class="team-season-match-row__side team-season-match-row__side--right">
                <text class="team-season-match-row__team team-season-match-row__team--away">{{ match.opponentName }}</text>
                <image class="team-season-match-row__avatar" :src="match.opponentAvatar || ''" mode="aspectFit" />
              </view>
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

    <FiLoginFloat v-if="loginPromptVisible" @action="handleSupportLogin" />
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiAiChatSheet from '../../components/FiAiChatSheet.vue'
import FiLoginFloat from '../../components/FiLoginFloat.vue'
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import { getCurrentUser } from '../../api/auth'
import { getAvailableRounds, getMatches, getOverview, getRankings } from '../../api/insight'
import { getSupportProfile, listSupportTeams, setFavoriteTeam } from '../../api/support'
import { getPublicSystemConfig } from '../../api/system'
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
import { getAccessToken, setAccessToken } from '../../utils/authStorage'
import bgImage from '../../static/user/phoenix-stadium-bg.jpg'
import memberCardDotsImage from '../../static/user/member-card-dots.png'
import pulseMatchCornerImage from '../../static/home/pulse-match-corner.png'
import { buildHomeBriefingMarqueeMap, splitBriefingMarqueeRows, type HomeBriefingMarqueeAccent } from '../../utils/homeBriefingMarquees'
import { buildHeadlineTitleParts } from '../../utils/homeViewText'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { reportPageActivity } from '../../utils/userActivity'
import { consumeOpenAiChatIntent } from '../../utils/aiEntryIntent'
import {
  type HomeTeamSeasonMatch,
  type HomePulseLeadMatch,
  resolveHomeGuideLeaders,
  resolveHomeGuideNote,
  resolveHomeGuideReferenceRoundNumber,
  resolveHomeHasAuthToken,
  isHomeAuthExpiredMessage,
  resolveHomeLoadPlan,
  resolveHomePulseLeadMatch,
  resolveHomePulseMatches,
  resolveHomePulseTechStats,
  resolveHomeTeamSeasonMatches,
  resolveHomeSupportNextMatchLabel,
  resolveHomeSupportTeamRankLabel,
  resolveHomeSupportMatchWeekdayLabel,
  resolveHomeSupportWindowShortLabel,
  shouldShowHomeSupportLoading,
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
  entities: Array<{ name: string; caption: string; avatar: string | null }>
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
const selectedPulseMatch = ref<HomePulseLeadMatch | null>(null)
const selectedStandingsTeam = ref<OverviewStanding | null>(null)
const allSeasonMatches = ref<MatchCard[] | null>(null)
const teamSeasonMatchesLoading = ref(false)
const teamSeasonMatchesErrorMessage = ref('')
const supportLoading = ref(true)
const supportErrorMessage = ref('')
const supportProfile = ref<SupportProfile | null>(null)
const supportTeams = ref<SupportTeam[]>([])
const supportTeamsLoading = ref(false)
const favoriteTeamSheetVisible = ref(false)
const selectedFavoriteTeamId = ref<number | null>(null)
const hasAuthToken = ref(resolveHomeHasAuthToken(getAccessToken()))
const loginPromptVisible = ref(false)

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
const selectedStandingsTeamRecordParts = computed(() => {
  const finishedMatches = selectedStandingsTeamMatches.value.filter((match) =>
    match.resultTone === 'win' || match.resultTone === 'draw' || match.resultTone === 'loss',
  )
  const wins = finishedMatches.filter((match) => match.resultTone === 'win').length
  const draws = finishedMatches.filter((match) => match.resultTone === 'draw').length
  const losses = finishedMatches.filter((match) => match.resultTone === 'loss').length

  return {
    finishedMatches: finishedMatches.length,
    wins,
    draws,
    losses,
  }
})
const selectedStandingsTeamRecordText = computed(() => {
  const record = selectedStandingsTeamRecordParts.value
  return `${record.wins}胜 ${record.draws}平 ${record.losses}负`
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
const showSupportLoading = computed(() =>
  shouldShowHomeSupportLoading({
    loading: supportLoading.value,
    hasCachedProfile: !!supportProfile.value,
  }),
)
const supportRefreshing = computed(() => supportLoading.value && !!supportProfile.value)
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
const supportNextMatchWeekdayLabel = computed(() => {
  return resolveHomeSupportMatchWeekdayLabel(supportNextMatch.value?.match_date)
})
const supportTeamRankMap = computed(() => {
  return new Map(supportTeams.value.map((team) => [team.team_id, team.rank_no]))
})
const supportNextMatchHomeRankLabel = computed(() => {
  const teamId = supportNextMatch.value?.home_team.team_id
  return resolveHomeSupportTeamRankLabel(teamId ? supportTeamRankMap.value.get(teamId) : null)
})
const supportNextMatchAwayRankLabel = computed(() => {
  const teamId = supportNextMatch.value?.away_team.team_id
  return resolveHomeSupportTeamRankLabel(teamId ? supportTeamRankMap.value.get(teamId) : null)
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
          entities: [],
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
          entities: leadingScorers.value.map((player) => ({
            name: player.player_name,
            caption: player.team_name,
            avatar: player.avatar_storage_url,
          })),
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
          entities: leadingAssists.value.map((player) => ({
            name: player.player_name,
            caption: player.team_name,
            avatar: player.avatar_storage_url,
          })),
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
    const loadPlan = resolveHomeLoadPlan(hasAuthToken.value)
    if (!loadPlan.critical.includes('overview')) {
      throw new Error('首页首屏加载计划缺少 overview')
    }

    overview.value = await getOverview({ mode: 'live', season: currentSeason, roundNumber: null })
    void loadDeferredHomeData()
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '首页数据加载失败，请稍后重试。')
  } finally {
    loading.value = false
  }
}

async function loadDeferredHomeData() {
  try {
    const [rankingsResponse, liveMatchesResponse, roundsResponse, publicSystemConfig] = await Promise.all([
      getRankings({ mode: 'live', season: currentSeason, roundNumber: null }),
      getMatches({ mode: 'live', season: currentSeason, roundNumber: null }),
      getAvailableRounds(currentSeason),
      getPublicSystemConfig(),
    ])

    rankings.value = rankingsResponse
    liveMatches.value = liveMatchesResponse.matches
    rounds.value = roundsResponse
    publicConfig.value = publicSystemConfig

    const guideReferenceRoundNumber = resolveHomeGuideReferenceRoundNumber(roundsResponse)
    guideRankings.value = guideReferenceRoundNumber === null
      ? null
      : await getRankings({ mode: 'round', season: currentSeason, roundNumber: guideReferenceRoundNumber })
  } catch (error) {
    console.warn('[home] deferred data load failed', error)
  }
}

async function loadSupportTeams() {
  if (supportTeams.value.length || supportTeamsLoading.value) {
    return
  }

  supportTeamsLoading.value = true
  try {
    supportTeams.value = await listSupportTeams()
  } catch (error) {
    console.warn('[home] support teams load failed', error)
  } finally {
    supportTeamsLoading.value = false
  }
}

async function loadSupportData() {
  supportErrorMessage.value = ''
  hasAuthToken.value = resolveHomeHasAuthToken(getAccessToken())
  loginPromptVisible.value = false

  if (!hasAuthToken.value) {
    supportProfile.value = null
    supportTeams.value = []
    loginPromptVisible.value = true
    supportLoading.value = false
    return
  }

  supportLoading.value = true

  try {
    supportProfile.value = await getSupportProfile()
    selectedFavoriteTeamId.value = supportProfile.value.favorite_team?.team_id ?? selectedFavoriteTeamId.value
    void loadSupportTeams()
  } catch (error) {
    const message = extractApiErrorMessage(error, '助力入口加载失败，请稍后重试。')

    if (isHomeAuthExpiredMessage(message)) {
      setAccessToken(null)
      hasAuthToken.value = false
      supportProfile.value = null
      supportErrorMessage.value = ''
      loginPromptVisible.value = true
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
    loginPromptVisible.value = true
    return
  }

  try {
    currentAiUser.value = await getCurrentUser()
  } catch {
    currentAiUser.value = null
  }

  if (!currentAiUser.value && hasAuthToken.value) {
    setAccessToken(null)
    hasAuthToken.value = false
    loginPromptVisible.value = true
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

async function openAiChatDirectly(): Promise<void> {
  const user = await ensureAiUser()
  if (!user) {
    promptAiLogin()
    return
  }

  aiChatVisible.value = true
}

function openAiFromBrandNav(): void {
  void openAiChatDirectly()
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

async function openFavoriteTeamSheet() {
  await loadSupportTeams()
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

function handleCloseAiChat() {
  aiChatVisible.value = false
}

onShow(() => {
  reportPageActivity('home')
  hasAuthToken.value = resolveHomeHasAuthToken(getAccessToken())
  void loadPage()
  void loadSupportData()

  if (consumeOpenAiChatIntent()) {
    void openAiChatDirectly()
  }
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
  padding-top: var(--fi-brand-nav-height);
  position: relative;
  z-index: 1;
  box-sizing: border-box;
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
  position: relative;
  overflow: hidden;
  padding: 26rpx 24rpx 20rpx;
  border-radius: 36rpx;
  background: rgba(255, 255, 255, 0.96);
  border-color: rgba(229, 231, 236, 0.98);
  box-shadow: 0 18rpx 46rpx rgba(26, 28, 36, 0.07);
}

.page-hero-dots {
  position: absolute;
  z-index: 0;
  width: 292rpx;
  height: 180rpx;
  opacity: 0.34;
  pointer-events: none;
}

.page-hero-dots--top {
  top: -54rpx;
  right: -54rpx;
}

.page-hero-dots--bottom {
  left: -86rpx;
  bottom: 136rpx;
  opacity: 0.18;
  transform: scaleX(-1);
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
  position: relative;
  z-index: 1;
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
  font-size: 46rpx;
  line-height: 1.12;
  letter-spacing: 0;
  color: #17181d;
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
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: #f6f7fb;
  color: #6d7280;
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
  color: #8f9198;
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
  background: #9aa0aa;
  box-shadow: 0 0 0 6rpx rgba(154, 160, 170, 0.12);
}

.hero-card__guide {
  position: relative;
  z-index: 1;
  margin-top: 24rpx;
  padding: 22rpx 0 0;
  border-top: 2rpx solid rgba(231, 232, 238, 0.94);
}

.hero-card__guide-title {
  color: #191b20;
  font-size: 30rpx;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 12rpx;
}

.hero-card__guide-title::before {
  content: '';
  width: 8rpx;
  height: 8rpx;
  background: #dc2626;
  border-radius: 999rpx;
  box-shadow: 0 0 0 8rpx rgba(220, 38, 38, 0.12);
}

.hero-card__guide-copy {
  margin-top: 12rpx;
  color: #606672;
  font-size: 27rpx;
  line-height: 1.62;
}

.hero-card__guide-note {
  margin-top: 10rpx;
  color: #8f9198;
  font-size: 22rpx;
  line-height: 1.55;
}

.inline-highlight {
  display: inline;
  color: #17181d;
  font-weight: 700;
  padding: 0 0.08em;
  background-image: linear-gradient(90deg, rgba(220, 38, 38, 0.22), rgba(220, 38, 38, 0.22));
  background-repeat: no-repeat;
  background-size: 100% 0.36em;
  background-position: left bottom;
}

.briefing-grid {
  position: relative;
  z-index: 1;
  margin-top: 22rpx;
  display: grid;
  gap: 0;
  border-top: 2rpx solid rgba(231, 232, 238, 0.88);
}

.briefing-card {
  position: relative;
  overflow: visible;
  min-height: 150rpx;
  padding: 20rpx 0 20rpx 20rpx;
  border-radius: 0;
  border: 0;
  border-bottom: 2rpx solid rgba(231, 232, 238, 0.88);
  background: transparent;
}

.briefing-card::before {
  content: '';
  position: absolute;
  left: 0;
  top: 26rpx;
  bottom: 26rpx;
  width: 4rpx;
  border-radius: 999rpx;
}

.briefing-card--leader::before {
  background: #b91c1c;
}

.briefing-card--scorer::before {
  background: #dc2626;
}

.briefing-card--assist::before {
  background: #ef4444;
}

.briefing-card__label {
  color: #8a8f9a;
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1;
}

.briefing-card__body {
  margin-top: 14rpx;
  display: grid;
  grid-template-columns: minmax(210rpx, 0.95fr) minmax(0, 1.35fr) 120rpx;
  column-gap: 16rpx;
  align-items: center;
}

.briefing-card__main {
  display: flex;
  align-items: center;
  gap: 14rpx;
  min-width: 0;
}

.briefing-card__leader-logo,
.briefing-card__entity-group {
  flex: 0 0 auto;
}

.briefing-card__leader-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 60rpx;
  height: 60rpx;
  border-radius: 18rpx;
  background: #f7f8fa;
  border: 2rpx solid rgba(231, 232, 238, 0.92);
}

.briefing-card__leader-logo-image {
  width: 48rpx;
  height: 48rpx;
}

.briefing-card__entity-group {
  display: flex;
  align-items: center;
  min-width: 78rpx;
}

.briefing-card__entity-avatar {
  width: 54rpx;
  height: 54rpx;
  border-radius: 999rpx;
  margin-right: -14rpx;
  border: 4rpx solid rgba(255, 255, 255, 0.96);
  background: #f7f8fa;
  box-shadow: 0 8rpx 18rpx rgba(24, 27, 33, 0.07);
}

.briefing-card__entity-avatar:last-child {
  margin-right: 0;
}

.briefing-card__title-block {
  display: grid;
  gap: 5rpx;
  min-width: 0;
}

.briefing-card__entity-list {
  display: grid;
  gap: 12rpx;
  min-width: 0;
  width: 100%;
}

.briefing-card__entity-row {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: 12rpx;
}

.briefing-card__entity-row-avatar {
  flex: 0 0 auto;
  width: 48rpx;
  height: 48rpx;
  border-radius: 999rpx;
  background: #f7f8fa;
}

.briefing-card__entity-row-body {
  display: grid;
  gap: 4rpx;
  min-width: 0;
}

.briefing-card__entity-row-name {
  color: #17181d;
  font-size: 28rpx;
  line-height: 1.12;
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.briefing-card__entity-row-team {
  color: #8a8f9a;
  font-size: 20rpx;
  line-height: 1.16;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.briefing-card__value {
  color: #17181d;
  font-size: 30rpx;
  line-height: 1.12;
  font-weight: 800;
}

.briefing-card__subvalue {
  color: #8a8f9a;
  font-size: 21rpx;
  line-height: 1.22;
}

.briefing-card__marquees {
  display: grid;
  align-self: stretch;
  align-content: center;
  gap: 6rpx;
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
  gap: 20rpx;
  min-width: max-content;
  animation-name: briefing-marquee-scroll;
  animation-timing-function: linear;
  animation-iteration-count: infinite;
}

.briefing-card__marquee-item {
  position: relative;
  padding-left: 16rpx;
  color: rgba(86, 91, 101, 0.88);
  font-size: 21rpx;
  line-height: 1.3;
}

.briefing-card__marquee-item::before {
  content: '•';
  position: absolute;
  left: 0;
  color: rgba(220, 38, 38, 0.9);
  font-size: 24rpx;
}

.briefing-card__metric {
  display: grid;
  width: 120rpx;
  align-content: center;
  justify-items: end;
  justify-self: end;
  align-self: stretch;
  text-align: right;
  gap: 8rpx;
}

.briefing-card__metric-value {
  color: #dc2626;
  font-size: 50rpx;
  line-height: 0.95;
  font-weight: 800;
}

.briefing-card__metric-label {
  color: #31343b;
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1.18;
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
  position: relative;
  overflow: hidden;
  padding: 18rpx 16rpx;
  display: grid;
  gap: 10rpx;
  border-radius: 28rpx;
  border: 2rpx solid #ececf1;
  background-color: #ffffff;
}

.score-strip--interactive {
  border-color: rgba(220, 38, 38, 0.18);
}

.score-strip__corner {
  position: absolute;
  z-index: 0;
  top: -58rpx;
  left: -74rpx;
  width: 214rpx;
  height: 132rpx;
  opacity: 0.52;
  pointer-events: none;
  transform: scaleX(-1);
}

.score-strip > view,
.score-strip > text {
  position: relative;
  z-index: 1;
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
  background: rgba(220, 38, 38, 0.1);
  color: #dc2626 !important;
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
  color: #dc2626;
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
  position: sticky;
  top: 0;
  z-index: 3;
  margin-top: 18rpx;
  padding: 22rpx 24rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: #ffffff;
  box-shadow: 0 12rpx 28rpx rgba(18, 20, 28, 0.06);
}

.tech-stats-sheet__teams {
  display: block;
  color: #121212;
  font-size: 30rpx;
  line-height: 1.35;
  font-weight: 800;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tech-stats-sheet__meta {
  display: block;
  margin-top: 10rpx;
  color: #8f9198;
  font-size: 24rpx;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
  height: 14rpx;
  border-radius: 999rpx;
  background: #eceef3;
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
  background: #15161b;
  transform: scaleX(0);
  animation: tech-stat-fill-grow 480ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
  animation-delay: calc(var(--tech-stat-delay, 120ms) + 70ms);
}

.tech-stat-row__fill--home {
  background: #dc2626;
  transform-origin: right center;
}

.tech-stat-row__fill--away {
  background: #dc2626;
  transform-origin: left center;
}

.team-season-sheet__title {
  display: flex;
  align-items: center;
  gap: 14rpx;
  min-width: 0;
  margin-top: 8rpx;
}

.team-season-sheet__title-avatar {
  width: 56rpx;
  height: 56rpx;
  flex: 0 0 auto;
}

.team-season-sheet__title-name {
  min-width: 0;
  margin-top: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.team-season-sheet__summary {
  margin-top: 18rpx;
  padding: 22rpx 24rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: #ffffff;
  display: block;
}

.team-season-sheet__summary-main {
  display: block;
  min-width: 0;
}

.team-season-sheet__summary-meta-grid {
  display: grid;
  grid-template-columns: minmax(0, 0.9fr) minmax(0, 1fr) minmax(160rpx, 1.45fr);
  align-items: center;
  gap: 12rpx;
  min-width: 0;
}

.team-season-sheet__summary-metric {
  display: grid;
  gap: 6rpx;
  min-width: 0;
  padding-left: 14rpx;
  border-left: 2rpx solid rgba(235, 236, 241, 0.95);
}

.team-season-sheet__summary-metric:first-child {
  border-left: 0;
  padding-left: 0;
}

.team-season-sheet__summary-value {
  color: #121212;
  font-size: 28rpx;
  line-height: 1.1;
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.team-season-sheet__summary-label {
  color: #8f9198;
  font-size: 20rpx;
  line-height: 1.15;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
.team-season-match-row__meta-right {
  display: inline-flex;
  align-items: center;
  gap: 8rpx;
}
.team-season-match-row__venue {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 56rpx;
  padding: 4rpx 12rpx;
  border-radius: 999rpx;
  font-size: 20rpx;
  font-weight: 800;
}
.team-season-match-row__venue--home {
  background: rgba(220, 38, 38, 0.12);
  color: #dc2626;
}
.team-season-match-row__venue--away {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.team-season-match-row__result {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 56rpx;
  padding: 4rpx 12rpx;
  border-radius: 999rpx;
  font-size: 20rpx;
  font-weight: 800;
}

.team-season-match-row__result--win {
  background: rgba(220, 38, 38, 0.12);
  color: #dc2626;
}

.team-season-match-row__result--draw {
  background: rgba(234, 179, 8, 0.12);
  color: #854d0e;
}

.team-season-match-row__result--loss {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.team-season-match-row__result--live {
  background: rgba(220, 38, 38, 0.12);
  color: #dc2626;
}

.team-season-match-row__result--scheduled {
  background: rgba(59, 130, 246, 0.12);
  color: #2563eb;
}

.team-season-match-row__body {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 18rpx;
}

.team-season-match-row__side {
  display: flex;
  align-items: center;
  gap: 12rpx;
}
.team-season-match-row__side--left {
  justify-content: flex-start;
}
.team-season-match-row__side--right {
  justify-content: flex-end;
}

.team-season-match-row__avatar {
  width: 36rpx;
  height: 36rpx;
  flex-shrink: 0;
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
  background: rgba(220, 38, 38, 0.08);
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
  gap: 12rpx;
}
.support-home-panel__heading {
  display: grid;
  gap: 0;
}
.support-home-panel__title {
  font-size: 36rpx;
  line-height: 1.12;
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
  background: #26a269;
  box-shadow: 0 0 0 6rpx rgba(38, 162, 105, 0.12);
}
.support-home-panel__context-label {
  color: #2f7f5f;
  font-size: 24rpx;
  font-weight: 700;
  line-height: 1.2;
}
.support-home-panel__context-note {
  color: #8f9198;
  font-size: 22rpx;
  line-height: 1.2;
}
.support-home-panel__refreshing {
  justify-self: start;
  color: #6d7280;
  font-size: 20rpx;
  line-height: 1;
  padding: 8rpx 14rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
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
  position: relative;
  margin-top: 22rpx;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 18rpx;
  padding: 22rpx 0 24rpx;
  border-top: 2rpx solid rgba(231, 232, 238, 0.92);
  border-bottom: 2rpx solid rgba(231, 232, 238, 0.92);
}
.support-home-panel__favorite::before {
  content: '';
  position: absolute;
  left: 0;
  top: 22rpx;
  bottom: 22rpx;
  width: 4rpx;
  border-radius: 999rpx;
  background: #26a269;
}
.support-home-panel__favorite-avatar-shell {
  margin-left: 18rpx;
  width: 88rpx;
  height: 88rpx;
  border-radius: 24rpx;
  background: #f7f9f8;
  border: 2rpx solid rgba(223, 229, 226, 0.96);
  display: flex;
  align-items: center;
  justify-content: center;
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
  display: block;
  font-size: 36rpx;
  line-height: 1;
  font-weight: 800;
}
.support-home-panel__favorite-note,
.favorite-team-sheet__note {
  display: block;
  margin-top: 6rpx;
  color: #8f9198;
  font-size: 22rpx;
}
.support-home-panel__switch {
  padding: 14rpx 22rpx;
  border-radius: 999rpx;
  background: #f5f6fa;
  color: #5f6673;
  font-size: 23rpx;
  font-weight: 700;
  line-height: 1;
}
.support-home-match-card {
  margin-top: 18rpx;
  padding: 22rpx 20rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(229, 231, 236, 0.98);
  background: #ffffff;
  box-shadow: 0 10rpx 28rpx rgba(18, 20, 28, 0.04);
}
.support-home-match-card__meta,
.support-home-match-card__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: #8f9198;
  font-size: 22rpx;
}
.support-home-match-card__meta {
  gap: 12rpx;
}
.support-home-match-card__meta-pill {
  min-height: 44rpx;
  border-radius: 999rpx;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  font-size: 22rpx;
  line-height: 1;
  font-weight: 800;
}
.support-home-match-card__meta-pill--round {
  padding: 0 18rpx;
  background: #15161b;
  color: #ffffff;
}
.support-home-match-card__meta-pill--time {
  margin-left: auto;
  gap: 10rpx;
  padding: 6rpx 14rpx 6rpx 8rpx;
  background: #f5f6fa;
  color: #5d6673;
}
.support-home-match-card__weekday {
  min-width: 58rpx;
  padding: 9rpx 12rpx;
  border-radius: 999rpx;
  box-sizing: border-box;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: #15161b;
  color: #ffffff;
  font-size: 24rpx;
  font-weight: 950;
  line-height: 1;
}
.support-home-match-card__datetime {
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: 8rpx;
  white-space: nowrap;
}
.support-home-match-card__teams {
  margin-top: 18rpx;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: 16rpx;
  align-items: center;
}
.support-home-match-card__team-block {
  min-width: 0;
  display: grid;
  gap: 8rpx;
}
.support-home-match-card__team-block--away {
  justify-items: end;
}
.support-home-match-card__team {
  color: #121212;
  font-size: 32rpx;
  line-height: 1.12;
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.support-home-match-card__team--away {
  text-align: right;
}
.support-home-match-card__rank {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  justify-self: start;
  min-height: 32rpx;
  padding: 0 12rpx;
  border-radius: 999rpx;
  background: #f5f6fa;
  color: #6d7280;
  font-size: 20rpx;
  font-weight: 800;
  line-height: 1;
}
.support-home-match-card__rank--away {
  justify-self: end;
}
.support-home-match-card__vs {
  color: #dc2626;
  font-size: 24rpx;
  font-weight: 900;
  line-height: 1;
  padding: 8rpx 14rpx;
  border-radius: 999rpx;
  background: rgba(220, 38, 38, 0.08);
}
.support-home-match-card__bar {
  margin-top: 18rpx;
  height: 12rpx;
  border-radius: 999rpx;
  overflow: hidden;
  background: #eef0f4;
  display: flex;
}
.support-home-match-card__bar-home {
  height: 100%;
  background: #26a269;
}
.support-home-match-card__bar-away {
  height: 100%;
  background: #a7adb8;
}
.support-home-match-card__action {
  color: #dc2626;
  font-weight: 800;
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
  background: rgba(220, 38, 38, 0.08);
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

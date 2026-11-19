import type { PageLayoutBlock, PageLayoutKey, PublicPageLayout } from '../types/pageLayout'

export const DEFAULT_PAGE_BLOCK_KEYS: Record<PageLayoutKey, readonly string[]> = {
  home: ['daily_topics', 'my_team_support', 'fan_entries', 'story_pulse', 'standings', 'top_scorers'],
  rankings: ['rankings_standings', 'rankings_surface', 'matches_progress', 'matches_upcoming', 'matches_results'],
  insights: ['ticket_watch_entry', 'secondary_entries', 'team_selector', 'match_outlook', 'team_insight'],
  seat_swap: ['notice', 'region_map', 'pool_list'],
  user: ['account_hero', 'account_info', 'season_recap_entry', 'account_actions', 'settings_entry'],
  ticket_watch: [
    'header',
    'current_match',
    'focus_panel',
    'tracked_interests',
    'recent_reflux',
    'best_windows',
    'inventory_stack',
    'supply_insights',
    'history_selector',
    'history_match',
    'history_trend',
    'share_poster',
  ],
  season_recap: ['heading', 'summary', 'theme_picker', 'matches', 'formula'],
  membership_purchase: ['availability_notice', 'tier_list', 'benefits', 'extra_links'],
  support: ['hero', 'scoreboard', 'stance_actions', 'poster'],
  topics: ['heading', 'list'],
  student: ['intro', 'contact_qr', 'services', 'orders', 'email_reminder'],
  user_settings: ['version_info', 'review_status', 'review_control', 'h5_test_login'],
  team_board: ['hero', 'launcher', 'posts'],
  insights_tactics: ['heading', 'team_selector', 'tactics_panel'],
  insights_transfer: ['heading', 'home_team_bar', 'transfer_panel'],
  lineup: ['team_picker', 'status_bar', 'formation_bar', 'pitch', 'hint', 'action_bar'],
  lineup_list: ['title', 'team_picker', 'sort_bar', 'list', 'cta'],
  transfer_board: ['title', 'tabs', 'list'],
  director: ['header', 'team_selector', 'foreign_section', 'domestic_section'],
  topic_detail: ['header', 'audio_player', 'roundup_content', 'article_gallery', 'article_body', 'article_timeline'],
  lineup_view: ['summary_panel', 'pitch', 'reaction_bar', 'navigation_bar'],
  transfer_view: ['header', 'card', 'actions'],
  telegraph_entry: ['timeline'],
  telegraph_comment: ['thread'],
  director_view: ['header', 'comparison', 'view_tabs', 'plan_overview', 'comments', 'circle_panel'],
}

export function sanitizeImageUrl(url: unknown): string | null {
  if (typeof url !== 'string') return null
  const trimmed = url.trim()
  if (!trimmed) return null
  if (/^https?:\/\/[^\s]+$/i.test(trimmed)) return trimmed
  return null
}

export function createDefaultPageLayout(pageKey: PageLayoutKey): PublicPageLayout {
  return {
    page_key: pageKey,
    background_image_url: null,
    blocks: DEFAULT_PAGE_BLOCK_KEYS[pageKey].map((key) => ({
      key,
      visible: true,
      background_image_url: null,
    })),
    version: 0,
    updated_at: null,
  }
}

export function normalizePageLayout(
  pageKey: PageLayoutKey,
  raw?: Partial<PublicPageLayout> | null,
): PublicPageLayout {
  const defaultKeys = DEFAULT_PAGE_BLOCK_KEYS[pageKey]
  const validSet = new Set(defaultKeys)
  const seen = new Set<string>()
  const blocks: PageLayoutBlock[] = []

  if (Array.isArray(raw?.blocks)) {
    for (const item of raw.blocks) {
      if (!item || typeof item.key !== 'string' || !validSet.has(item.key) || seen.has(item.key)) {
        continue
      }
      seen.add(item.key)
      blocks.push({
        key: item.key,
        visible: item.visible !== false,
        background_image_url: sanitizeImageUrl(item.background_image_url),
      })
    }
  }

  for (const key of defaultKeys) {
    if (!seen.has(key)) {
      blocks.push({
        key,
        visible: true,
        background_image_url: null,
      })
    }
  }

  return {
    page_key: pageKey,
    background_image_url: sanitizeImageUrl(raw?.background_image_url),
    blocks,
    version: typeof raw?.version === 'number' && Number.isFinite(raw.version) ? raw.version : 0,
    updated_at: typeof raw?.updated_at === 'string' ? raw.updated_at : null,
  }
}

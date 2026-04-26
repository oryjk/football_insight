export type TeamBoardInsightKind = 'goals_for' | 'assists_for' | 'goals_against'

export interface TeamBoardTeam {
  team_id: number
  team_name: string
  rank_no: number
  avatar_storage_url: string | null
}

export interface TeamBoardSnapshotItem {
  item_id: number | null
  name: string
  avatar_storage_url: string | null
  value: number
  share: number
}

export interface TeamBoardSnapshotSection {
  title: string
  metric_label: string
  items: TeamBoardSnapshotItem[]
}

export interface TeamBoardSnapshot {
  current_season: number
  round_number: number | null
  team_id: number
  team_name: string
  rank_no: number
  avatar_storage_url: string | null
  insight_kind: TeamBoardInsightKind
  insight_label: string
  summary_label: string
  summary_value: number
  sections: TeamBoardSnapshotSection[]
}

export interface TeamBoardComposerPreset {
  insight_kind: TeamBoardInsightKind
  label: string
  snapshot: TeamBoardSnapshot
}

export interface TeamBoardAuthor {
  user_id: string
  display_name: string
  avatar_url: string | null
}

export interface TeamBoardComment {
  comment_id: string
  post_id: string
  author: TeamBoardAuthor
  content: string
  created_at: string
}

export interface TeamBoardPost {
  post_id: string
  team_id: number
  insight_kind: TeamBoardInsightKind
  insight_label: string
  title: string
  commentary: string
  author: TeamBoardAuthor
  snapshot: TeamBoardSnapshot
  like_count: number
  comment_count: number
  liked_by_viewer: boolean
  created_at: string
  comments: TeamBoardComment[]
}

export interface TeamBoardViewResponse {
  current_season: number
  round_number: number | null
  team: TeamBoardTeam
  composer_presets: TeamBoardComposerPreset[]
  posts: TeamBoardPost[]
}

export interface CreateTeamBoardPostPayload {
  insight_kind: TeamBoardInsightKind
  title: string
  commentary: string
}

export interface AddTeamBoardCommentPayload {
  content: string
}

export interface TeamBoardLikeSummary {
  post_id: string
  liked_by_viewer: boolean
  like_count: number
}

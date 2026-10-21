export interface RecapMatch { match_id: number; match_date: string; opponent: string; opponent_avatar_url?: string | null; is_home: boolean; city?: string; round_trip_km?: number | null }
export interface RecapSummary { away_km: number; home_km: number | null; total_km: number; matches: number; home_matches: number; away_matches: number; cities: number }
export interface RecapSeasonTotals { home_matches: number; away_matches: number; completed_matches: number }
export interface SeasonRecap { season_totals?: RecapSeasonTotals; season: number; matches: RecapMatch[]; selected_match_ids: number[]; home_one_way_km: number | null; summary: RecapSummary }
export interface SaveSeasonRecap { selected_match_ids: number[]; home_one_way_km: number | null }
export interface RecapCodeImage { image_base64: string; mime_type: 'image/png' | 'image/jpeg' }

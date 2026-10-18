import { request } from '../utils/request'
import type { SeasonRecap, SaveSeasonRecap, RecapCodeImage } from '../types/seasonRecap'
export function getSeasonRecap(season: number): Promise<SeasonRecap> { return request({ url: `/season-recap/${season}`, auth: true }) }
export function saveSeasonRecap(season: number, data: SaveSeasonRecap): Promise<SeasonRecap> { return request({ url: `/season-recap/${season}`, method: 'PUT', auth: true, data }) }
export function getSeasonRecapCode(season: number): Promise<RecapCodeImage> { return request({ url: `/season-recap/${season}/code`, method: 'POST', auth: true }) }

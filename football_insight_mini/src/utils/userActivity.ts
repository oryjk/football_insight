import { recordPageActivity, type ActivityPageKey } from '../api/activity'
import { getAccessToken } from './authStorage'

export function reportPageActivity(pageKey: ActivityPageKey): void {
  if (!getAccessToken()) {
    return
  }

  void recordPageActivity(pageKey).catch((error) => {
    console.warn('[activity] failed to report page activity', pageKey, error)
  })
}

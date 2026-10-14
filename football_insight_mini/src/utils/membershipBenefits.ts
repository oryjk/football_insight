import type { MembershipTierRule } from '../types/system'
import { resolveMembershipTierGuide } from './membershipRules'

export interface MembershipBenefitsViewer {
  membership_benefits_enabled?: boolean
}

export function resolveMembershipBenefitsLocked(
  viewer: MembershipBenefitsViewer | null | undefined,
): boolean {
  return viewer?.membership_benefits_enabled === false
}

export interface MembershipBenefitItem {
  key: string
  iconName: 'ticket' | 'activity' | 'radar' | 'history'
  title: string
  caption: string
}

interface BuildMembershipBenefitOptions {
  membershipCode: string
  refreshLabel: string
}

export function buildMembershipBenefitItems(
  options: BuildMembershipBenefitOptions,
): MembershipBenefitItem[] {
  const tierNumber = parseMembershipTierNumber(options.membershipCode)

  return [
    {
      key: 'watch',
      iconName: 'ticket',
      title: '余票监控',
      caption: options.refreshLabel,
    },
    {
      key: 'recent-reflux',
      iconName: 'activity',
      title: '最近回流速览',
      caption: resolveRecentRefluxBenefitCaption(tierNumber),
    },
    {
      key: 'tracking',
      iconName: 'radar',
      title: '更多钓区跟踪',
      caption: tierNumber >= 4 ? '跟踪上限更高' : '常用钓区可跟踪',
    },
    {
      key: 'history',
      iconName: 'history',
      title: '历史回流复盘',
      caption: tierNumber >= 5 ? '更全面复盘' : '回看近期变化',
    },
  ]
}

function parseMembershipTierNumber(code: string): number {
  const parsed = Number.parseInt(code.trim().replace(/^V/i, ''), 10)
  return Number.isFinite(parsed) ? parsed : 1
}

function resolveRecentRefluxBenefitCaption(tierNumber: number): string {
  if (tierNumber >= 8) {
    return '3 分钟内'
  }

  if (tierNumber >= 7) {
    return '10 分钟内'
  }

  if (tierNumber >= 6) {
    return '30 分钟内'
  }

  return 'V6 起开放'
}

export interface MembershipBenefitPreview {
  code: string
  toneClass: string
  items: MembershipBenefitItem[]
}

export function buildMembershipBenefitPreview(
  tier: string | null | undefined,
  rules?: MembershipTierRule[] | null,
): MembershipBenefitPreview | null {
  if (!tier?.trim()) return null

  const guide = resolveMembershipTierGuide(tier, rules)
  const hasConfiguredRule = rules?.some(rule => rule.code.trim().toUpperCase() === guide.code)
  return {
    code: guide.code,
    toneClass: guide.toneClass,
    items: buildMembershipBenefitItems({
      membershipCode: guide.code,
      refreshLabel: hasConfiguredRule ? guide.refreshLabel : '以后端配置为准',
    }),
  }
}

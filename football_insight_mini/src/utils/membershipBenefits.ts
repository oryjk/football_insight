export interface MembershipBenefitsViewer {
  membership_benefits_enabled?: boolean
}

export function resolveMembershipBenefitsLocked(
  viewer: MembershipBenefitsViewer | null | undefined,
): boolean {
  return viewer?.membership_benefits_enabled === false
}

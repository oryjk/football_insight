/** 中超前两名的亚冠资格：第 1 名进亚冠精英联赛，第 2 名进亚冠二级联赛。 */
export interface AclQualification {
  tier: 'elite' | 'second'
  /** 行内小标签，两三个字以内。 */
  label: string
  /** 无障碍朗读用的完整名称。 */
  fullLabel: string
}

export function resolveAclQualification(rankNo: number): AclQualification | null {
  if (rankNo === 1) return { tier: 'elite', label: '亚冠精英', fullLabel: '亚冠精英联赛区' }
  if (rankNo === 2) return { tier: 'second', label: '亚冠二级', fullLabel: '亚冠二级联赛区' }
  return null
}

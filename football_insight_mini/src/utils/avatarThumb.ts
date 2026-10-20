export type AvatarThumbSize = 96 | 240

/** MinIO 球员目录的头像 → 同目录 thumbs/<size>/<文件名>.jpg；其他地址原样返回。 */
export function avatarThumbUrl(url: string | null | undefined, size: AvatarThumbSize): string {
  if (!url) return ''
  const match = /^(https?:\/\/[^?#]*\/minio\/[^?#]*\/players\/)([^/?#]+)$/.exec(url)
  return match ? `${match[1]}thumbs/${size}/${match[2]}.jpg` : url
}

/** 依次尝试的地址：小图 → 原图（去重）；都失败时由组件显示首字。 */
export function avatarCandidates(url: string | null | undefined, size: AvatarThumbSize): string[] {
  if (!url) return []
  return [...new Set([avatarThumbUrl(url, size), url])]
}

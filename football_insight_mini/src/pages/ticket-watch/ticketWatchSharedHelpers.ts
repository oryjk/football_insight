export function resolveTicketWatchBlockKey(block: { block_key?: string | null; block_name: string }): string {
  const blockKey = block.block_key?.trim()
  return blockKey || block.block_name
}

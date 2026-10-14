export function topicCardPresentation(title: string): { tone: string; label: string } {
  if (/前瞻|预告|迎战/.test(title)) return { tone: 'preview', label: '赛前看点' }
  if (/争冠|夺冠|冠军|分差|悬念/.test(title)) return { tone: 'championship', label: '争冠追踪' }
  if (/阿洛伊西|主帅|主教练|质疑/.test(title)) return { tone: 'analysis', label: '球队观察' }
  return { tone: 'news', label: '球队动态' }
}

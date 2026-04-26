interface LeadMatchSummary {
  homeTeamName: string
  awayTeamName: string
  homeScore: string
  awayScore: string
}

interface BuildHeadlineTitlePartsOptions {
  headline: string | null
  leadMatch: LeadMatchSummary | null
  topTeamName: string | null
}

interface HeadlineTitleParts {
  leading: string
  highlighted: string
  trailing: string
}

function normalizeHeadline(headline: string | null): string {
  return (headline ?? '').replace(/\s+/g, ' ').trim()
}

function buildMatchHighlight(match: LeadMatchSummary): string {
  return `${match.homeTeamName} ${match.homeScore}:${match.awayScore} ${match.awayTeamName}`
}

function splitAroundHighlight(source: string, highlighted: string): HeadlineTitleParts {
  const index = source.indexOf(highlighted)

  if (index === -1) {
    return {
      leading: source,
      highlighted: '',
      trailing: '',
    }
  }

  return {
    leading: source.slice(0, index),
    highlighted,
    trailing: source.slice(index + highlighted.length),
  }
}

export function buildHeadlineTitleParts(
  options: BuildHeadlineTitlePartsOptions,
): HeadlineTitleParts {
  const headline = normalizeHeadline(options.headline)

  if (headline) {
    if (options.leadMatch) {
      const matchHighlight = buildMatchHighlight(options.leadMatch)

      if (headline.includes(matchHighlight)) {
        return splitAroundHighlight(headline, matchHighlight)
      }
    }

    if (options.topTeamName && headline.includes(options.topTeamName)) {
      return splitAroundHighlight(headline, options.topTeamName)
    }

    return {
      leading: headline,
      highlighted: '',
      trailing: '',
    }
  }

  return {
    leading: '这一轮之后，谁在改变联赛格局',
    highlighted: '',
    trailing: '',
  }
}

# CFL Schedule Fallback Design

## Goal

Keep Sina as the primary CSL data source while using the Chinese Football League
official schedule API to recover rounds that Sina cannot return completely. The
merged schedule must preserve postponed fixtures, avoid duplicate match rows,
and let the product identify the latest active round correctly.

## Confirmed Data Semantics

- The 2026 round 18 response from the official API contains eight matches.
- Four matches missing from the current Sina response have
  `match_status = Postponed`; they are not finished or removed.
- Round 18 therefore remains incomplete, while rounds 19 and 20 may still be
  complete and round 21 may be the current round.
- Sina `cur_round` may be `0`, so it is not a reliable sole source for the
  product's current round.

## Source Strategy

The scraper fetches the schedule one round at a time.

1. Request the round from Sina.
2. If Sina succeeds with exactly eight distinct matches, use that round.
3. If Sina fails or returns fewer than eight matches, request the official API
   with `week` equal to the round number.
4. Merge both responses by season, round, home team, and away team. Sina wins
   for matches present in both sources because it supplies the existing numeric
   identifiers and richer downstream-compatible data. The official source adds
   missing matches and supplies their official status.
5. Reject duplicate, ambiguous, overfull, or still-incomplete rounds.

For 2026, the default official tournament calendar ID is
`e6818x4pwankpph8awr91m1hw`. An environment variable can override it. A season
without a configured calendar may continue when Sina is complete, but an
incomplete Sina round must fail rather than silently write partial data.

## Team And Match Identity

Official contestant IDs and names differ from Sina IDs and display names. The
fallback adapter maps official teams to the known Sina team catalog for the
season. Known sponsor and spelling variants are explicit aliases. Any unmapped
or non-unique team aborts the scrape so the system never guesses an identity.

The database gains a natural uniqueness rule over season, round, home team ID,
and away team ID. This key lets an official fallback update an existing Sina
fixture without creating a duplicate. Existing `match_id` values remain stable
because other product tables reference them. For a genuinely new official-only
fixture, the scraper creates a deterministic, JavaScript-safe numeric ID from
the official match ID; a later Sina response updates the same natural-key row
without changing that stored numeric ID.

## Persistence Reconciliation

`f_i_matches` gains:

- `source_active BOOLEAN NOT NULL DEFAULT TRUE`
- `last_seen_run_id UUID NULL`
- source metadata sufficient to audit whether the latest schedule data came
  from Sina or the official fallback

Within one database transaction, every match in a validated merged snapshot is
upserted, marked active, and stamped with the current run ID. Rows for the same
season that are absent from the complete merged snapshot become inactive; they
are not deleted. A reappearing fixture becomes active again.

Soft deactivation is allowed only when all expected rounds were fetched and
every round contains exactly eight distinct fixtures, currently 240 matches for
a 30-round season. Any HTTP failure, parsing error, identity ambiguity, or
structural incompleteness rolls back the transaction and leaves existing match
activity unchanged.

## Status Mapping

The official adapter maps statuses into the scraper's stored status codes:

- `Fixture` -> scheduled
- active match statuses -> live
- `Played` -> finished
- `Postponed` -> postponed

The Rust read adapter exposes postponed explicitly instead of treating an
unknown code as live. The mini-program/H5 match card shows `延期`; postponed
matches never enter kickoff-based live inference and do not show scores as if
they were in progress.

## Current Round And Progress

Normal match and progress queries include only `source_active = TRUE` rows.
Round progress still counts a round as complete only when every active fixture
is finished.

The current round is the highest-numbered incomplete round that has actually
started. A round has started when it contains an explicit live or finished
match, or a non-postponed fixture whose kickoff time has passed. If no
incomplete round has started, the first upcoming incomplete round is selected.

With the confirmed 2026 data, round 18 stays incomplete because four fixtures
are postponed, rounds 19 and 20 stay complete, and round 21 is current.

## Frontend Safety Bound

An explicit backend status always wins. A scheduled match may be inferred as
live only from kickoff through three hours after kickoff. After that window it
returns to scheduled unless the backend reports `live` or `finished`. This
prevents stale upstream status from making a match appear live forever and does
not affect postponed matches.

## Testing And Operations

Tests cover official response parsing, incomplete-round fallback, source merge
precedence, identity failures, status mapping, atomic reconciliation, current
round selection, and frontend status rendering. Each behavior is introduced by
a failing regression test before implementation.

Local verification includes the full scraper, Rust, and frontend suites plus
Rust formatting, Clippy, boundary scanning, TypeScript checks, and the H5 build.
This change does not authorize a production migration, scraper run, backend
deployment, or restart. Local and production runtime status must be reported
separately.

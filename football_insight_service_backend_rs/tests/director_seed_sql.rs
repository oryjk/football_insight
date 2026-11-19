use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct SeedCandidateRow {
    code: String,
    player_name: String,
    position: String,
    country: String,
    is_foreign: bool,
    league_tag: String,
    current_club: String,
    tags_json: String,
    target_team_id: Option<i64>,
    avatar_url: Option<String>,
    sort_order: i32,
    active: bool,
}

fn strip_sql_line_comments(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len());
    for line in sql.lines() {
        let mut in_single_quote = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '\'' {
                if in_single_quote && i + 1 < chars.len() && chars[i + 1] == '\'' {
                    out.push('\'');
                    out.push('\'');
                    i += 2;
                    continue;
                }
                in_single_quote = !in_single_quote;
                out.push('\'');
                i += 1;
            } else if !in_single_quote
                && chars[i] == '-'
                && i + 1 < chars.len()
                && chars[i + 1] == '-'
            {
                break;
            } else {
                out.push(chars[i]);
                i += 1;
            }
        }
        out.push('\n');
    }
    out
}

fn split_tuples(values_block: &str) -> Vec<String> {
    let mut tuples = Vec::new();
    let mut in_quote = false;
    let mut depth = 0usize;
    let mut current = String::new();
    let chars: Vec<char> = values_block.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '\'' {
            if in_quote && i + 1 < chars.len() && chars[i + 1] == '\'' {
                if depth > 0 {
                    current.push('\'');
                    current.push('\'');
                }
                i += 2;
                continue;
            }
            in_quote = !in_quote;
            if depth > 0 {
                current.push(ch);
            }
            i += 1;
            continue;
        }

        if !in_quote {
            if ch == '(' {
                depth += 1;
                if depth == 1 {
                    current.clear();
                    i += 1;
                    continue;
                }
            } else if ch == ')' {
                assert!(depth > 0, "unexpected closing parenthesis in VALUES block");
                depth -= 1;
                if depth == 0 {
                    tuples.push(current.trim().to_string());
                    current.clear();
                    i += 1;
                    continue;
                }
            }
        }

        if depth > 0 {
            current.push(ch);
        }
        i += 1;
    }

    assert!(!in_quote, "unterminated SQL string literal in VALUES block");
    assert_eq!(depth, 0, "unbalanced parentheses in VALUES block");
    tuples
}

fn split_fields(tuple_body: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();
    let chars: Vec<char> = tuple_body.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '\'' {
            if in_quote && i + 1 < chars.len() && chars[i + 1] == '\'' {
                current.push('\'');
                current.push('\'');
                i += 2;
                continue;
            }
            in_quote = !in_quote;
            current.push(ch);
            i += 1;
            continue;
        }

        if !in_quote && ch == ',' {
            fields.push(current.trim().to_string());
            current.clear();
            i += 1;
            continue;
        }

        current.push(ch);
        i += 1;
    }

    if !current.trim().is_empty() {
        fields.push(current.trim().to_string());
    }
    fields
}

fn unquote_sql_string(raw: &str) -> String {
    let trimmed = raw.trim();
    assert!(
        trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2,
        "expected single-quoted SQL string, got: {raw}"
    );
    trimmed[1..trimmed.len() - 1].replace("''", "'")
}

fn parse_optional_sql_string(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("NULL") {
        None
    } else {
        Some(unquote_sql_string(trimmed))
    }
}

fn parse_sql_bool(raw: &str) -> bool {
    match raw.trim().to_ascii_uppercase().as_str() {
        "TRUE" => true,
        "FALSE" => false,
        other => panic!("expected SQL boolean TRUE/FALSE, got: {other}"),
    }
}

fn parse_seed_rows(sql: &str) -> Vec<SeedCandidateRow> {
    let cleaned = strip_sql_line_comments(sql);
    let values_idx = cleaned
        .find("VALUES")
        .expect("seed SQL must contain VALUES clause");
    let conflict_idx = cleaned
        .find("ON CONFLICT (code) DO UPDATE")
        .expect("seed SQL must contain ON CONFLICT (code) DO UPDATE");
    assert!(
        values_idx < conflict_idx,
        "VALUES clause must precede ON CONFLICT (code) DO UPDATE"
    );

    let values_block = &cleaned[values_idx + "VALUES".len()..conflict_idx];
    let tuples = split_tuples(values_block);
    assert!(!tuples.is_empty(), "seed SQL must contain at least one row");

    tuples
        .into_iter()
        .enumerate()
        .map(|(idx, tuple)| {
            let fields = split_fields(&tuple);
            assert_eq!(
                fields.len(),
                12,
                "row #{idx} must have 12 columns, got {}: {tuple}",
                fields.len()
            );

            let tags_raw = fields[7].trim();
            let tags_literal = tags_raw
                .strip_suffix("::jsonb")
                .expect("tags field must be explicitly cast with ::jsonb");

            let target_team_id = if fields[8].trim().eq_ignore_ascii_case("NULL") {
                None
            } else {
                Some(
                    fields[8]
                        .trim()
                        .parse::<i64>()
                        .unwrap_or_else(|e| panic!("invalid target_team_id in row #{idx}: {e}")),
                )
            };

            let sort_order = fields[10]
                .trim()
                .parse::<i32>()
                .unwrap_or_else(|e| panic!("invalid sort_order in row #{idx}: {e}"));

            SeedCandidateRow {
                code: unquote_sql_string(&fields[0]),
                player_name: unquote_sql_string(&fields[1]),
                position: unquote_sql_string(&fields[2]),
                country: unquote_sql_string(&fields[3]),
                is_foreign: parse_sql_bool(&fields[4]),
                league_tag: unquote_sql_string(&fields[5]),
                current_club: unquote_sql_string(&fields[6]),
                tags_json: unquote_sql_string(tags_literal),
                target_team_id,
                avatar_url: parse_optional_sql_string(&fields[9]),
                sort_order,
                active: parse_sql_bool(&fields[11]),
            }
        })
        .collect()
}

#[test]
fn director_seed_sql_is_idempotent_and_meets_pool_requirements() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sql_path = manifest_dir.join("migrations/20261119130000_seed_director_candidates.sql");
    let sql = fs::read_to_string(&sql_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", sql_path.display()));

    assert!(
        sql.contains("INSERT INTO f_i_director_candidates"),
        "seed SQL must contain INSERT INTO f_i_director_candidates"
    );
    assert!(
        sql.contains("ON CONFLICT (code) DO UPDATE"),
        "seed SQL must contain ON CONFLICT (code) DO UPDATE for idempotency"
    );

    let expected_columns = [
        "code",
        "player_name",
        "position",
        "country",
        "is_foreign",
        "league_tag",
        "current_club",
        "tags",
        "target_team_id",
        "avatar_url",
        "sort_order",
        "active",
    ];
    let insert_start = sql.find("INSERT INTO f_i_director_candidates").unwrap();
    let values_start = sql.find("VALUES").unwrap();
    let header_slice = &sql[insert_start..values_start];
    for col in expected_columns {
        assert!(
            header_slice.contains(col),
            "INSERT column list is missing `{col}`"
        );
    }

    let rows = parse_seed_rows(&sql);

    assert!(
        rows.len() >= 50,
        "expected >= 50 total candidates, got {}",
        rows.len()
    );

    let mut seen_codes = HashSet::new();
    let mut k_league_count = 0usize;
    let mut non_k_foreign_count = 0usize;
    let mut domestic_count = 0usize;
    let mut domestic_positions = HashSet::new();
    let mut target_null_count = 0usize;
    let mut target_77680_count = 0usize;

    for row in &rows {
        assert!(
            !row.code.is_empty()
                && !row.code.starts_with('-')
                && !row.code.ends_with('-')
                && !row.code.contains("--")
                && row
                    .code
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
            "invalid kebab-case code: `{}`",
            row.code
        );
        assert!(
            seen_codes.insert(row.code.clone()),
            "duplicate candidate code: `{}`",
            row.code
        );

        assert!(
            !row.player_name.trim().is_empty(),
            "empty player_name for `{}`",
            row.code
        );
        assert!(
            !row.country.trim().is_empty(),
            "empty country for `{}`",
            row.code
        );
        assert!(
            !row.current_club.trim().is_empty(),
            "empty current_club for `{}`",
            row.code
        );
        assert!(
            matches!(row.position.as_str(), "GK" | "DF" | "MF" | "FW"),
            "invalid position `{}` for `{}`",
            row.position,
            row.code
        );

        let tags: Vec<String> = serde_json::from_str(&row.tags_json).unwrap_or_else(|err| {
            panic!(
                "invalid JSON string array in tags for `{}` (`{}`): {err}",
                row.code, row.tags_json
            )
        });
        assert!(
            (1..=3).contains(&tags.len()),
            "expected 1..=3 tags for `{}`, got {}",
            row.code,
            tags.len()
        );
        for tag in &tags {
            assert!(
                !tag.trim().is_empty(),
                "blank tag found in candidate `{}`",
                row.code
            );
        }

        if let Some(ref url) = row.avatar_url {
            assert!(
                !url.trim().is_empty(),
                "avatar_url should be NULL or non-empty for `{}`",
                row.code
            );
        }
        assert!(
            row.sort_order > 0,
            "sort_order must be positive for `{}`",
            row.code
        );
        assert!(row.active, "seed candidate `{}` should be active", row.code);

        match row.target_team_id {
            None => target_null_count += 1,
            Some(77680) => target_77680_count += 1,
            Some(other) => assert!(other > 0, "target_team_id must be positive, got {other}"),
        }

        if row.is_foreign {
            if matches!(row.league_tag.as_str(), "K1" | "K2") {
                k_league_count += 1;
            } else {
                assert!(
                    matches!(row.league_tag.as_str(), "J1" | "BRA" | "EUR" | "FREE"),
                    "unexpected non-K foreign league_tag `{}` for `{}`",
                    row.league_tag,
                    row.code
                );
                non_k_foreign_count += 1;
            }
        } else {
            assert!(
                matches!(row.league_tag.as_str(), "CSL" | "CL1" | "FREE"),
                "unexpected domestic league_tag `{}` for `{}`",
                row.league_tag,
                row.code
            );
            domestic_count += 1;
            domestic_positions.insert(row.position.as_str());
        }
    }

    assert!(
        k_league_count >= 24,
        "expected >= 24 K-League (K1/K2) candidates, got {k_league_count}"
    );
    assert!(
        non_k_foreign_count >= 16,
        "expected >= 16 non-K-League foreign candidates, got {non_k_foreign_count}"
    );
    assert!(
        domestic_count >= 12,
        "expected >= 12 domestic candidates, got {domestic_count}"
    );
    for pos in ["GK", "DF", "MF", "FW"] {
        assert!(
            domestic_positions.contains(pos),
            "domestic candidates must cover position `{pos}`"
        );
    }
    assert!(
        target_null_count > target_77680_count && target_77680_count > 0,
        "expected majority NULL target_team_id ({target_null_count}) with some 77680 highlights ({target_77680_count})"
    );
}

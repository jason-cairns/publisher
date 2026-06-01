use super::markers::{MarkerDecodeError, PublisherMarker};
use super::{ParseError, ParsedFile, PublicationParser, normalize_source_path, scopes};
use crate::model::{ParseWarning, ParseWarningKind};

pub(super) fn record_marker(
    parser: &PublicationParser,
    parsed: &mut ParsedFile,
    marker: Result<PublisherMarker, MarkerDecodeError>,
) -> Result<(), ParseError> {
    match marker {
        Ok(marker) => record_decoded_marker(parser, parsed, marker),
        Err(error) => {
            parsed.warnings.push(ParseWarning {
                source_path: Some(parsed.source_path.clone()),
                kind: ParseWarningKind::UnsupportedPublisherCall,
                message: format!("unsupported publisher marker preserved for review: {error}"),
            });
            Ok(())
        }
    }
}

fn record_decoded_marker(
    parser: &PublicationParser,
    parsed: &mut ParsedFile,
    marker: PublisherMarker,
) -> Result<(), ParseError> {
    match marker {
        PublisherMarker::Publish { path } => {
            for child_path in expand_publish_path(parser, &path)? {
                parsed.child_paths.push(child_path);
            }
        }
        PublisherMarker::PublicationScope { id, title, tags } => {
            scopes::record_publication_scope(parsed, id, title, tags);
        }
        // `#in-scope(..ids)[..]` is an authoring-time region context switch handled at
        // render entrypoint generation, not a graph edge or scope declaration.
        PublisherMarker::InScope { ids: _ } => {}
    }

    Ok(())
}

fn expand_publish_path(parser: &PublicationParser, path: &str) -> Result<Vec<String>, ParseError> {
    let normalized = normalize_source_path(path);
    if !contains_glob_meta(&normalized) {
        return Ok(vec![normalized]);
    }

    let mut matches = parser
        .all_typ_files()?
        .into_iter()
        .filter(|candidate| glob_path_matches(&normalized, candidate))
        .collect::<Vec<_>>();
    matches.sort();

    if matches.is_empty() {
        Ok(vec![normalized])
    } else {
        Ok(matches)
    }
}

fn contains_glob_meta(path: &str) -> bool {
    path.bytes().any(|byte| byte == b'*' || byte == b'?')
}

fn glob_path_matches(pattern: &str, candidate: &str) -> bool {
    let pattern_segments = pattern.split("/").collect::<Vec<_>>();
    let candidate_segments = candidate.split("/").collect::<Vec<_>>();
    glob_segments_match(&pattern_segments, &candidate_segments)
}

fn glob_segments_match(pattern: &[&str], candidate: &[&str]) -> bool {
    if pattern.is_empty() {
        return candidate.is_empty();
    }

    if pattern[0] == "**" {
        return glob_segments_match(&pattern[1..], candidate)
            || (!candidate.is_empty() && glob_segments_match(pattern, &candidate[1..]));
    }

    if candidate.is_empty() {
        return false;
    }

    glob_segment_matches(pattern[0], candidate[0])
        && glob_segments_match(&pattern[1..], &candidate[1..])
}

fn glob_segment_matches(pattern: &str, candidate: &str) -> bool {
    let pattern = pattern.as_bytes();
    let candidate = candidate.as_bytes();
    let mut dp = vec![vec![false; candidate.len() + 1]; pattern.len() + 1];
    dp[0][0] = true;

    for pattern_index in 1..=pattern.len() {
        if pattern[pattern_index - 1] == b'*' {
            dp[pattern_index][0] = dp[pattern_index - 1][0];
        }
    }

    for pattern_index in 1..=pattern.len() {
        for candidate_index in 1..=candidate.len() {
            dp[pattern_index][candidate_index] = match pattern[pattern_index - 1] {
                b'*' => {
                    dp[pattern_index - 1][candidate_index] || dp[pattern_index][candidate_index - 1]
                }
                b'?' => dp[pattern_index - 1][candidate_index - 1],
                byte => {
                    byte == candidate[candidate_index - 1]
                        && dp[pattern_index - 1][candidate_index - 1]
                }
            };
        }
    }

    dp[pattern.len()][candidate.len()]
}

use super::markers::{MarkerDecodeError, PublisherMarker};
use super::{ParseError, ParsedFile, PublicationParser, normalize_source_path, scopes};
use crate::model::{ParseWarning, ParseWarningKind};

pub(super) fn record_marker(
    _parser: &PublicationParser,
    parsed: &mut ParsedFile,
    marker: Result<PublisherMarker, MarkerDecodeError>,
) -> Result<(), ParseError> {
    match marker {
        Ok(marker) => record_decoded_marker(parsed, marker),
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
    parsed: &mut ParsedFile,
    marker: PublisherMarker,
) -> Result<(), ParseError> {
    match marker {
        PublisherMarker::Publish { path } => {
            parsed.child_paths.push(normalize_source_path(&path));
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

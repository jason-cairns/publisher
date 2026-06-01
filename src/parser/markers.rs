use std::fmt;

use typst::foundations::{Dict, Label, Value};
use typst::introspection::MetadataElem;
use typst::layout::PagedDocument;
use typst::utils::PicoStr;

use super::world::MarkerWorld;
use super::{ParseError, PublicationParser};

const MARKER_LABEL: &str = "publisher-marker";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PublisherMarker {
    Publish {
        path: String,
    },
    PublicationScope {
        id: String,
        title: Option<String>,
        tags: Vec<String>,
    },
    ScopePayload {
        kind: String,
        value: String,
    },
    InScope {
        ids: Vec<String>,
    },
}

pub(super) struct MarkerEvaluation {
    pub(super) markers: Vec<Result<PublisherMarker, MarkerDecodeError>>,
    pub(super) support_files: std::collections::BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MarkerDecodeError {
    message: String,
}

impl MarkerDecodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for MarkerDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

pub(super) fn evaluate_markers(
    parser: &PublicationParser,
    source_path: &str,
) -> Result<MarkerEvaluation, ParseError> {
    let world = MarkerWorld::new(&parser.root_dir, source_path);
    let warned = typst::compile::<PagedDocument>(&world);
    let document = warned.output.map_err(|diagnostics| ParseError::Typst {
        source_path: source_path.to_string(),
        diagnostics: diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.to_string())
            .collect(),
    })?;

    let label = Label::new(PicoStr::intern(MARKER_LABEL)).expect("marker label is non-empty");
    let markers = document
        .introspector
        .query(&typst::foundations::Selector::Label(label))
        .into_iter()
        .filter_map(|content| {
            let metadata = content.to_packed::<MetadataElem>()?;
            Some(decode_marker(&metadata.value))
        })
        .collect();

    Ok(MarkerEvaluation {
        markers,
        support_files: world.support_files(),
    })
}

fn decode_marker(value: &Value) -> Result<PublisherMarker, MarkerDecodeError> {
    let dict = expect_dict(value, "publisher marker value")?;
    let kind = required_string(dict, "kind")?;

    match kind.as_str() {
        "publish" => Ok(PublisherMarker::Publish {
            path: required_string(dict, "path")?,
        }),
        "scope" => Ok(PublisherMarker::PublicationScope {
            id: required_string(dict, "id")?,
            title: optional_content_text(dict, "title")?,
            tags: string_array(dict, "tags")?,
        }),
        "payload" => Ok(PublisherMarker::ScopePayload {
            kind: required_string(dict, "payload_kind")?,
            value: required_string(dict, "value")?,
        }),
        "in-scope" => Ok(PublisherMarker::InScope {
            ids: string_array(dict, "ids")?,
        }),
        other => Err(MarkerDecodeError::new(format!(
            "unknown marker kind {other:?}"
        ))),
    }
}

fn expect_dict<'a>(value: &'a Value, context: &str) -> Result<&'a Dict, MarkerDecodeError> {
    match value {
        Value::Dict(dict) => Ok(dict),
        other => Err(MarkerDecodeError::new(format!(
            "{context} must be a dictionary, got {}",
            other.ty().short_name()
        ))),
    }
}

fn field<'a>(dict: &'a Dict, key: &str) -> Option<&'a Value> {
    dict.get(key)
        .ok()
        .filter(|value| !matches!(value, Value::None))
}

fn required_string(dict: &Dict, key: &str) -> Result<String, MarkerDecodeError> {
    match field(dict, key) {
        Some(Value::Str(value)) => Ok(value.to_string()),
        Some(other) => Err(MarkerDecodeError::new(format!(
            "field {key:?} must be a string, got {}",
            other.ty().short_name()
        ))),
        None => Err(MarkerDecodeError::new(format!(
            "missing required field {key:?}"
        ))),
    }
}

fn optional_content_text(dict: &Dict, key: &str) -> Result<Option<String>, MarkerDecodeError> {
    match field(dict, key) {
        Some(Value::Content(content)) => Ok(Some(content.plain_text().trim().to_string())),
        Some(other) => Err(MarkerDecodeError::new(format!(
            "field {key:?} must be content, got {}",
            other.ty().short_name()
        ))),
        None => Ok(None),
    }
}

fn string_array(dict: &Dict, key: &str) -> Result<Vec<String>, MarkerDecodeError> {
    match field(dict, key) {
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| match value {
                Value::Str(value) => Ok(value.to_string()),
                other => Err(MarkerDecodeError::new(format!(
                    "field {key:?} entries must be strings, got {}",
                    other.ty().short_name()
                ))),
            })
            .collect(),
        Some(other) => Err(MarkerDecodeError::new(format!(
            "field {key:?} must be an array, got {}",
            other.ty().short_name()
        ))),
        None => Ok(Vec::new()),
    }
}

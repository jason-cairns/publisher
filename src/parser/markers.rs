use std::fmt;

use typst::foundations::{Dict, Label, Repr, Value};
use typst::introspection::MetadataElem;
use typst::layout::PagedDocument;
use typst::utils::PicoStr;

use super::world::MarkerWorld;
use super::{ParseError, PublicationParser};

const MARKER_LABEL: &str = "publisher-marker";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PublisherMarker {
    Child {
        path: String,
    },
    Publish {
        path: String,
    },
    Children {
        pattern: String,
    },
    Scope {
        kind: String,
        name: Option<String>,
    },
    PublicationScope {
        id: String,
        title: Option<String>,
        tags: Vec<String>,
    },
    Outline {
        depth: Option<i64>,
        title: Option<String>,
        target: Option<String>,
    },
    Bibliography {
        source: String,
        scope: Option<String>,
    },
    Reference {
        target: String,
    },
    NavSuppress,
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
        "child" => Ok(PublisherMarker::Child {
            path: required_string(dict, "path")?,
        }),
        "publish" => Ok(PublisherMarker::Publish {
            path: required_string(dict, "path")?,
        }),
        "children" => Ok(PublisherMarker::Children {
            pattern: required_string(dict, "pattern")?,
        }),
        "scope" => {
            if field(dict, "id").is_some() {
                Ok(PublisherMarker::PublicationScope {
                    id: required_string(dict, "id")?,
                    title: optional_content_text(dict, "title")?,
                    tags: string_array(dict, "tags")?,
                })
            } else {
                Ok(PublisherMarker::Scope {
                    kind: required_string(dict, "scope_kind")?,
                    name: optional_string(dict, "name")?,
                })
            }
        }
        "outline" => Ok(PublisherMarker::Outline {
            depth: optional_i64(dict, "depth")?,
            title: optional_content_text(dict, "title")?,
            target: optional_raw_typst(dict, "target")?,
        }),
        "bibliography" => Ok(PublisherMarker::Bibliography {
            source: required_string(dict, "source")?,
            scope: optional_string(dict, "scope")?,
        }),
        "reference" => Ok(PublisherMarker::Reference {
            target: required_label(dict, "target")?,
        }),
        "nav-suppress" => Ok(PublisherMarker::NavSuppress),
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

fn optional_string(dict: &Dict, key: &str) -> Result<Option<String>, MarkerDecodeError> {
    match field(dict, key) {
        Some(Value::Str(value)) => Ok(Some(value.to_string())),
        Some(other) => Err(MarkerDecodeError::new(format!(
            "field {key:?} must be a string, got {}",
            other.ty().short_name()
        ))),
        None => Ok(None),
    }
}

fn optional_i64(dict: &Dict, key: &str) -> Result<Option<i64>, MarkerDecodeError> {
    match field(dict, key) {
        Some(Value::Int(value)) => Ok(Some(*value)),
        Some(other) => Err(MarkerDecodeError::new(format!(
            "field {key:?} must be an integer, got {}",
            other.ty().short_name()
        ))),
        None => Ok(None),
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

fn optional_raw_typst(dict: &Dict, key: &str) -> Result<Option<String>, MarkerDecodeError> {
    Ok(field(dict, key).map(|value| value.repr().to_string()))
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

fn required_label(dict: &Dict, key: &str) -> Result<String, MarkerDecodeError> {
    match field(dict, key) {
        Some(Value::Label(value)) => Ok(value.resolve().to_string()),
        Some(other) => Err(MarkerDecodeError::new(format!(
            "field {key:?} must be a label, got {}",
            other.ty().short_name()
        ))),
        None => Err(MarkerDecodeError::new(format!(
            "missing required field {key:?}"
        ))),
    }
}

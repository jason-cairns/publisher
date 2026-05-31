use std::collections::BTreeMap;

use super::markers::{MarkerDecodeError, PublisherMarker};
use super::{ParseError, ParsedFile, PublicationParser, normalize_source_path, projections};
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
        PublisherMarker::Child { path } => {
            parsed.child_paths.push(normalize_source_path(&path));
        }
        PublisherMarker::Publish { path } => {
            parsed.child_paths.push(normalize_source_path(&path));
        }
        PublisherMarker::Children { pattern } => {
            let children = parser.expand_children_glob(&parsed.source_path, &pattern)?;
            parsed.child_paths.extend(children);
        }
        PublisherMarker::Scope { kind, name } => {
            projections::record_scope(parsed, &CallArgs::scope(kind, name));
        }
        PublisherMarker::PublicationScope { id, title, tags } => {
            projections::record_publication_scope(parsed, id, title, tags);
        }
        PublisherMarker::Outline {
            depth,
            title,
            target,
        } => {
            projections::record_outline(parsed, &CallArgs::outline(depth, title, target));
        }
        PublisherMarker::Bibliography { source, scope } => {
            projections::record_bibliography(parsed, &CallArgs::bibliography(source, scope));
        }
        PublisherMarker::Reference { target } => {
            projections::record_reference(parsed, &CallArgs::reference(target));
        }
        PublisherMarker::NavSuppress => {
            projections::record_nav_suppression(parsed);
        }
    }

    Ok(())
}

#[derive(Clone, Debug, Default)]
pub(super) struct CallArgs {
    positional: Vec<ArgValue>,
    named: BTreeMap<String, ArgValue>,
}

impl CallArgs {
    fn scope(kind: String, name: Option<String>) -> Self {
        let mut args = Self::default();
        args.named
            .insert("kind".to_string(), ArgValue::String(kind));
        if let Some(name) = name {
            args.named
                .insert("name".to_string(), ArgValue::String(name));
        }
        args
    }

    fn outline(depth: Option<i64>, title: Option<String>, target: Option<String>) -> Self {
        let mut args = Self::default();
        if let Some(depth) = depth {
            args.named
                .insert("depth".to_string(), ArgValue::Number(depth));
        }
        if let Some(title) = title {
            args.named
                .insert("title".to_string(), ArgValue::ContentText(title));
        }
        if let Some(target) = target {
            args.named
                .insert("target".to_string(), ArgValue::Raw(target));
        }
        args
    }

    fn bibliography(source: String, scope: Option<String>) -> Self {
        let mut args = Self {
            positional: vec![ArgValue::String(source)],
            named: BTreeMap::new(),
        };
        if let Some(scope) = scope {
            args.named
                .insert("scope".to_string(), ArgValue::String(scope));
        }
        args
    }

    fn reference(target: String) -> Self {
        Self {
            positional: vec![ArgValue::Label(target)],
            named: BTreeMap::new(),
        }
    }

    pub(super) fn first_string(&self) -> Option<String> {
        self.positional
            .iter()
            .find_map(ArgValue::as_string)
            .cloned()
    }

    pub(super) fn first_label(&self) -> Option<String> {
        self.positional.iter().find_map(ArgValue::as_label).cloned()
    }

    pub(super) fn named_string(&self, name: &str) -> Option<String> {
        self.named.get(name).and_then(ArgValue::as_string).cloned()
    }

    pub(super) fn named_i64(&self, name: &str) -> Option<i64> {
        self.named.get(name).and_then(ArgValue::as_i64)
    }

    pub(super) fn named_content_text(&self, name: &str) -> Option<String> {
        self.named
            .get(name)
            .and_then(ArgValue::as_content_text)
            .cloned()
    }

    pub(super) fn named_raw(&self, name: &str) -> Option<String> {
        self.named.get(name).map(ArgValue::raw)
    }
}

#[derive(Clone, Debug)]
enum ArgValue {
    String(String),
    Number(i64),
    Label(String),
    ContentText(String),
    Raw(String),
}

impl ArgValue {
    fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    fn as_label(&self) -> Option<&String> {
        match self {
            Self::Label(value) => Some(value),
            _ => None,
        }
    }

    fn as_content_text(&self) -> Option<&String> {
        match self {
            Self::ContentText(value) => Some(value),
            _ => None,
        }
    }

    fn raw(&self) -> String {
        match self {
            Self::String(value) => format!("{value:?}"),
            Self::Number(value) => value.to_string(),
            Self::Label(value) => format!("<{value}>"),
            Self::ContentText(value) => format!("[{value}]"),
            Self::Raw(value) => value.clone(),
        }
    }
}

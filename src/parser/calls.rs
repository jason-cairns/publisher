use std::collections::BTreeMap;

use typst_syntax::ast::{self, AstNode};

use super::{
    ParseError, ParsedFile, PublicationParser, normalize_source_path, plain_markup, projections,
};
use crate::model::{ParseWarning, ParseWarningKind};

pub(super) fn record_call(
    parser: &PublicationParser,
    parsed: &mut ParsedFile,
    call: ast::FuncCall<'_>,
) -> Result<(), ParseError> {
    let callee = raw_expr(call.callee());
    let args = CallArgs::from(call.args());

    match callee.as_str() {
        "publisher.child" => {
            if let Some(path) = args.first_string() {
                parsed.child_paths.push(normalize_source_path(&path));
            }
        }
        "publisher.children" => {
            if let Some(pattern) = args.first_string() {
                let children = parser.expand_children_glob(&parsed.source_path, &pattern)?;
                parsed.child_paths.extend(children);
            }
        }
        "publisher.scope" => projections::record_scope(parsed, &args),
        "publisher.outline" => projections::record_outline(parsed, args),
        "publisher.bibliography" => projections::record_bibliography(parsed, args),
        "publisher.ref" => projections::record_reference(parsed, args),
        "publisher.nav.suppress" => projections::record_nav_suppression(parsed),
        _ if callee.starts_with("publisher.") => {
            parsed.warnings.push(ParseWarning {
                source_path: Some(parsed.source_path.clone()),
                kind: ParseWarningKind::UnsupportedPublisherCall,
                message: format!("unsupported publisher call preserved for review: {callee}"),
            });
        }
        _ => {}
    }

    Ok(())
}

#[derive(Clone, Debug, Default)]
pub(super) struct CallArgs {
    positional: Vec<ArgValue>,
    named: BTreeMap<String, ArgValue>,
}

impl<'a> From<ast::Args<'a>> for CallArgs {
    fn from(args: ast::Args<'a>) -> Self {
        let mut parsed = Self::default();
        for arg in args.items() {
            match arg {
                ast::Arg::Pos(expr) => parsed.positional.push(ArgValue::from_expr(expr)),
                ast::Arg::Named(named) => {
                    parsed.named.insert(
                        named.name().as_str().to_string(),
                        ArgValue::from_expr(named.expr()),
                    );
                }
                ast::Arg::Spread(spread) => parsed
                    .positional
                    .push(ArgValue::Raw(raw_expr(spread.expr()))),
            }
        }
        parsed
    }
}

impl CallArgs {
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
    fn from_expr(expr: ast::Expr<'_>) -> Self {
        match expr {
            ast::Expr::Str(value) => Self::String(value.get().to_string()),
            ast::Expr::Int(value) => Self::Number(value.get()),
            ast::Expr::Label(value) => Self::Label(value.get().to_string()),
            ast::Expr::ContentBlock(value) => {
                Self::ContentText(plain_markup(value.body().to_untyped()))
            }
            _ => Self::Raw(raw_expr(expr)),
        }
    }

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

fn raw_expr(expr: ast::Expr<'_>) -> String {
    expr.to_untyped().clone().into_text().to_string()
}

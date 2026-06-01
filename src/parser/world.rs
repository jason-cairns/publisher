use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::fonts::FontSearcher;

use crate::package_path;

pub(super) struct MarkerWorld {
    root_dir: PathBuf,
    main: FileId,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    support_files: Mutex<BTreeSet<String>>,
}

impl MarkerWorld {
    pub(super) fn new(root_dir: &Path, source_path: &str) -> Self {
        let fonts = FontSearcher::new().include_system_fonts(false).search();

        Self {
            root_dir: root_dir.to_path_buf(),
            main: file_id(source_path),
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts.iter().filter_map(|slot| slot.get()).collect(),
            support_files: Mutex::new(BTreeSet::new()),
        }
    }

    pub(super) fn support_files(&self) -> BTreeSet<String> {
        self.support_files.lock().unwrap().clone()
    }

    fn resolve(&self, id: FileId) -> FileResult<PathBuf> {
        if let Some(path) = package_path::resolve(id)? {
            return Ok(path);
        }

        id.vpath()
            .resolve(&self.root_dir)
            .ok_or_else(|| FileError::AccessDenied)
    }
}

impl World for MarkerWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        let path = self.resolve(id)?;
        if path.extension().is_some_and(|extension| extension != "typ") {
            return Err(FileError::NotSource);
        }

        let mut text = fs::read_to_string(&path).map_err(|source| file_error(source, &path))?;
        if id == self.main {
            text = sanitize_for_marker_evaluation(&text);
        }
        if id != self.main {
            self.support_files.lock().unwrap().insert(source_path(id));
        }
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = self.resolve(id)?;
        let bytes = fs::read(&path).map_err(|source| file_error(source, &path))?;
        Ok(Bytes::new(bytes))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Datetime::from_ymd(2026, 5, 24)
    }
}

fn file_id(source_path: &str) -> FileId {
    FileId::new(None, VirtualPath::new(source_path))
}

fn source_path(id: FileId) -> String {
    id.vpath()
        .as_rootless_path()
        .to_string_lossy()
        .replace('\\', "/")
}

fn file_error(error: io::Error, path: &Path) -> FileError {
    FileError::from_io(error, path)
}

fn sanitize_for_marker_evaluation(text: &str) -> String {
    let mut sanitized = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            sanitized.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            sanitized.push(ch);
        } else if ch == '@' && chars.peek().is_some_and(|next| is_label_char(*next)) {
            while chars.peek().is_some_and(|next| is_label_char(*next)) {
                chars.next();
            }
            sanitized.push_str("[]");
        } else {
            sanitized.push(ch);
        }
    }

    sanitized
}

fn is_label_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '/')
}

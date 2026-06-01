use std::env;
use std::path::PathBuf;

use typst::diag::FileResult;
use typst::syntax::FileId;

pub(crate) fn resolve(id: FileId) -> FileResult<Option<PathBuf>> {
    let Some(spec) = id.package() else {
        return Ok(None);
    };

    let mut path = data_dir().ok_or(typst::diag::FileError::AccessDenied)?;
    path.push("typst");
    path.push("packages");
    path.push(spec.namespace.as_str());
    path.push(spec.name.as_str());
    path.push(spec.version.to_string());
    path.push(id.vpath().as_rootless_path());
    Ok(Some(path))
}

fn data_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("PUBLISHER_TYPST_DATA_DIR") {
        return Some(path.into());
    }
    if let Some(path) = env::var_os("XDG_DATA_HOME") {
        return Some(path.into());
    }

    let home = env::var_os("HOME").map(PathBuf::from);
    match env::consts::OS {
        "macos" => home.map(|path| path.join("Library/Application Support")),
        "windows" => env::var_os("APPDATA").map(PathBuf::from),
        _ => home.map(|path| path.join(".local/share")),
    }
}

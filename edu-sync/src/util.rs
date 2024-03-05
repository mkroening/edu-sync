use std::{borrow::Cow, ffi::OsStr, mem, path::PathBuf, sync::OnceLock};

use directories::ProjectDirs;
use regex::{NoExpand, Regex};

pub fn project_dirs() -> &'static ProjectDirs {
    static PROJECT_DIRS: OnceLock<ProjectDirs> = OnceLock::new();

    PROJECT_DIRS.get_or_init(|| {
        ProjectDirs::from("org", "Edu Sync", "Edu Sync")
            .expect("no valid home directory path could be retrieved from the operating system")
    })
}

pub fn shared_http() -> reqwest::Client {
    static SHARED: OnceLock<reqwest::Client> = OnceLock::new();

    SHARED.get_or_init(reqwest::Client::new).clone()
}

fn split_file_at_dot(file: &OsStr) -> (&OsStr, Option<&OsStr>) {
    let slice = file.as_encoded_bytes();
    if slice == b".." {
        return (file, None);
    }

    // The unsafety here stems from converting between &OsStr and &[u8]
    // and back. This is safe to do because (1) we only look at ASCII
    // contents of the encoding and (2) new &OsStr values are produced
    // only from ASCII-bounded slices of existing &OsStr values.
    let i = match slice[1..].iter().position(|b| *b == b'.') {
        Some(i) => i + 1,
        None => return (file, None),
    };
    let before = &slice[..i];
    let after = &slice[i + 1..];
    unsafe {
        (
            OsStr::from_encoded_bytes_unchecked(before),
            Some(OsStr::from_encoded_bytes_unchecked(after)),
        )
    }
}

pub trait PathBufExt {
    fn push_file_name_suffix(&mut self, path: impl AsRef<OsStr>);

    fn push_file_prefix_suffix(&mut self, path: impl AsRef<OsStr>);
}

impl PathBufExt for PathBuf {
    fn push_file_name_suffix(&mut self, suffix: impl AsRef<OsStr>) {
        if self.file_name().is_some() {
            let mut path = mem::take(self).into_os_string();
            path.push(suffix);
            *self = path.into();
        }
    }

    fn push_file_prefix_suffix(&mut self, suffix: impl AsRef<OsStr>) {
        if let Some(file_name) = self.file_name() {
            let (before, after) = split_file_at_dot(file_name);
            let mut file_name = before.to_os_string();
            file_name.push(suffix);
            if let Some(after) = after {
                file_name.push(".");
                file_name.push(after);
            }
            self.set_file_name(file_name);
        }
    }
}

pub fn sanitize_path_component(path_component: &str) -> Cow<'_, str> {
    static RE: OnceLock<Regex> = OnceLock::new();

    RE.get_or_init(|| Regex::new(r"[\\/]+|^\.\.??$").unwrap())
        .replace_all(path_component, NoExpand("_"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_filename_test() {
        assert_eq!(sanitize_path_component(r"/a//b/\c\\d\"), "_a_b_c_d_");
        assert_eq!(sanitize_path_component(".a.b."), ".a.b.");
        assert_eq!(sanitize_path_component("."), "_");
        assert_eq!(sanitize_path_component(".."), "_");
        assert_eq!(sanitize_path_component("..."), "...");
    }

    #[test]
    fn push_extension_test() {
        let mut path = PathBuf::from("foo.rs");
        path.push_file_name_suffix(".txt");
        assert_eq!(path, PathBuf::from("foo.rs.txt"));

        let mut path = PathBuf::from("foo");
        path.push_file_name_suffix(".txt");
        assert_eq!(path, PathBuf::from("foo.txt"));
    }

    #[test]
    fn push_prefix_suffix_test() {
        let mut path = PathBuf::from("foo");
        path.push_file_prefix_suffix("_1");
        assert_eq!(path, PathBuf::from("foo_1"));

        let mut path = PathBuf::from("foo.rs");
        path.push_file_prefix_suffix("_1");
        assert_eq!(path, PathBuf::from("foo_1.rs"));

        let mut path = PathBuf::from("foo.tar.gz");
        path.push_file_prefix_suffix("_1");
        assert_eq!(path, PathBuf::from("foo_1.tar.gz"));
    }
}

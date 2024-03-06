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

pub trait PathBufExt {
    fn push_file_name_suffix(&mut self, path: impl AsRef<OsStr>);
}

impl PathBufExt for PathBuf {
    fn push_file_name_suffix(&mut self, suffix: impl AsRef<OsStr>) {
        if self.file_name().is_some() {
            let mut path = mem::take(self).into_os_string();
            path.push(suffix);
            *self = path.into();
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
}

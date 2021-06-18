use std::{borrow::Cow, ffi::OsStr, path::PathBuf};

use directories::ProjectDirs;
use lazy_static::lazy_static;
use regex::{NoExpand, Regex};

pub fn project_dirs() -> &'static ProjectDirs {
    lazy_static! {
        static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("org", "Edu Sync", "Edu Sync")
            .expect("no valid home directory path could be retrieved from the operating system");
    }

    &PROJECT_DIRS
}

pub fn shared_http() -> reqwest::Client {
    lazy_static! {
        static ref SHARED: reqwest::Client = reqwest::Client::new();
    }

    SHARED.clone()
}

pub trait PathBufExt {
    fn push_extension(&mut self, path: impl AsRef<OsStr>);
}

impl PathBufExt for PathBuf {
    fn push_extension(&mut self, extension: impl AsRef<OsStr>) {
        if let Some(current) = self.extension() {
            let mut current = current.to_os_string();
            current.push(".");
            current.push(extension);
            self.set_extension(current);
        } else {
            self.set_extension(extension);
        }
    }
}

pub fn sanitize_path_component(path_component: &str) -> Cow<'_, str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[\\/]+|^\.\.??$").unwrap();
    }

    RE.replace_all(path_component, NoExpand("_"))
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
        path.push_extension("txt");
        assert_eq!(path, PathBuf::from("foo.rs.txt"));

        let mut path = PathBuf::from("foo");
        path.push_extension("txt");
        assert_eq!(path, PathBuf::from("foo.txt"));
    }
}

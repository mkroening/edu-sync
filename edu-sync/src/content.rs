use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use edu_ws::{
    response::content::{Content as WsContent, Type},
    token::Token,
};
use filetime::FileTime;
use reqwest::Url;
use tokio::{
    fs::{self, File},
    io::{self, AsyncWriteExt},
    task,
};

use crate::util::{self, PathBufExt};

#[derive(Debug, Clone)]
pub struct Content {
    ws_content: WsContent,
    path: PathBuf,
}

#[derive(Debug)]
pub enum SyncStatus {
    Downloadable(Download),
    NotSupported(Type, PathBuf),
    Outdated(PathBuf),
    UpToDate(PathBuf),
    Modified(PathBuf),
}

impl Content {
    #[must_use]
    pub fn new(ws_content: WsContent, module_path: PathBuf) -> Self {
        let path = {
            let mut path = module_path;

            if let Some(content_path) = &ws_content.path {
                if let Ok(content_path) = content_path.strip_prefix("/") {
                    path.push(content_path);
                } else {
                    path.push(content_path);
                }
            }

            if !<&str>::try_from(path.file_name().unwrap())
                .unwrap()
                .ends_with(&ws_content.name)
            {
                path.push(&ws_content.name);
            }

            if ws_content.ty == Type::Url {
                path.push_file_name_suffix(".html");
            }

            path
        };

        Self { ws_content, path }
    }

    fn mtime(&self) -> FileTime {
        FileTime::from_unix_time(self.ws_content.modified.unix_timestamp(), 0)
    }

    fn download(self) -> SyncStatus {
        let mtime = self.mtime();
        match self.ws_content.ty {
            Type::File => {
                let common = CommonDownload::new(self.path, mtime);
                let url = self.ws_content.url.unwrap();
                let size = self.ws_content.size;
                SyncStatus::Downloadable(Download::File(FileDownload { url, size, common }))
            }
            Type::Url => {
                let common = CommonDownload::new(self.path, mtime);
                let url = self.ws_content.url.unwrap();
                SyncStatus::Downloadable(Download::Url(UrlDownload { url, common }))
            }
            Type::Content => {
                let common = CommonDownload::new(self.path, mtime);
                let content = self.ws_content.content.unwrap();
                SyncStatus::Downloadable(Download::Content(ContentDownload { content, common }))
            }
            Type::Folder => SyncStatus::NotSupported(Type::Folder, self.path),
        }
    }

    pub async fn sync(self) -> SyncStatus {
        match cmp_mtime(&self.path, &self.mtime()).await.ok() {
            None => self.download(),
            Some(Ordering::Less) => SyncStatus::Outdated(self.path),
            Some(Ordering::Equal) => SyncStatus::UpToDate(self.path),
            Some(Ordering::Greater) => SyncStatus::Modified(self.path),
        }
    }
}

#[derive(Debug)]
pub enum Download {
    File(FileDownload),
    Url(UrlDownload),
    Content(ContentDownload),
}

impl Download {
    pub fn size(&self) -> u64 {
        match self {
            Download::File(file_download) => file_download.size(),
            Download::Url(url_download) => url_download.size() as u64,
            Download::Content(content_download) => content_download.size() as u64,
        }
    }
}

#[derive(Debug)]
pub struct ContentDownload {
    content: String,
    common: CommonDownload,
}

impl ContentDownload {
    pub async fn run(&mut self) -> io::Result<()> {
        let (mut file, path) = self.common.create_file().await?;
        file.write_all(self.content.as_bytes()).await?;
        self.common.finish(&mut file, path).await?;
        Ok(())
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.common.dst_path.as_path()
    }

    pub fn size(&self) -> usize {
        self.content.len()
    }
}

#[derive(Debug)]
pub struct FileDownload {
    url: Url,
    size: u64,
    common: CommonDownload,
}

impl FileDownload {
    pub async fn run(
        &mut self,
        token: &Token,
        mut report_progress: impl FnMut(u64) + Send,
    ) -> io::Result<()> {
        let (mut file, path) = self.common.create_file().await?;
        token.apply(&mut self.url);
        let mut response = util::shared_http()
            .get(self.url.clone())
            .send()
            .await
            .unwrap();
        let mut progress = 0;
        while let Some(chunk) = response.chunk().await.unwrap() {
            file.write_all(&chunk).await?;
            progress += chunk.len() as u64;
            report_progress(progress);
        }
        self.common.finish(&mut file, path).await?;
        Ok(())
    }

    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.common.dst_path.as_path()
    }
}

#[derive(Debug)]
pub struct UrlDownload {
    url: Url,
    common: CommonDownload,
}

impl UrlDownload {
    pub async fn run(&mut self) -> io::Result<()> {
        let (mut file, path) = self.common.create_file().await?;
        let buf = format!(include_str!("url_format.html"), url = self.url);
        file.write_all(buf.as_bytes()).await?;
        self.common.finish(&mut file, path).await?;
        Ok(())
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.common.dst_path.as_path()
    }

    #[must_use]
    pub fn size(&self) -> usize {
        include_str!("url_format.html").len() - "{url}".len() + self.url.as_str().len()
    }
}

#[derive(Debug)]
pub struct CommonDownload {
    dst_path: PathBuf,
    mtime: FileTime,
}

impl CommonDownload {
    fn new(dst_path: PathBuf, mtime: FileTime) -> Self {
        Self { dst_path, mtime }
    }

    async fn create_file(&self) -> io::Result<(File, PathBuf)> {
        let dl_path = {
            let mut dl_path = self.dst_path.clone();
            dl_path.push_file_name_suffix(".tmp");
            dl_path
        };

        if let Some(parent) = dl_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let file = File::create(&dl_path).await?;

        Ok((file, dl_path))
    }

    async fn finish(&mut self, file: &mut File, dl_path: PathBuf) -> io::Result<()> {
        file.shutdown().await?;
        let mtime = self.mtime;
        let dl_path = task::spawn_blocking(move || {
            filetime::set_file_times(&dl_path, mtime, mtime)?;
            Result::<_, io::Error>::Ok(dl_path)
        })
        .await??;
        fs::rename(dl_path, &self.dst_path).await?;
        Ok(())
    }
}

async fn cmp_mtime(path: &Path, mtime: &FileTime) -> io::Result<Ordering> {
    fs::metadata(path).await.map(|metadata| {
        let file_mtime = FileTime::from_last_modification_time(&metadata);
        file_mtime.cmp(mtime)
    })
}

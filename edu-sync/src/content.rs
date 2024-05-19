use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    time::SystemTime,
};

use edu_ws::{
    response::content::{Content as WsContent, Type},
    token::Token,
};
use reqwest::Url;
use tokio::{
    fs::{self, File},
    io::{self, AsyncBufReadExt, AsyncSeekExt, AsyncWriteExt, BufReader},
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
    UpToDate(PathBuf),
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

    fn mtime(&self) -> SystemTime {
        self.ws_content.modified.into()
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
        let latest_path = latest_path(self.path.clone()).await.unwrap();
        match cmp_mtime(&latest_path, &self.mtime()).await.ok() {
            None | Some(Ordering::Less) | Some(Ordering::Greater) => self.download(),
            Some(Ordering::Equal) => SyncStatus::UpToDate(latest_path),
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
        self.common.finish(file, path).await?;
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
        self.common.finish(file, path).await?;
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
        self.common.finish(file, path).await?;
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
    mtime: SystemTime,
}

impl CommonDownload {
    fn new(dst_path: PathBuf, mtime: SystemTime) -> Self {
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
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&dl_path)
            .await?;

        Ok((file, dl_path))
    }

    async fn finish(&mut self, mut file: File, dl_path: PathBuf) -> io::Result<()> {
        let latest_path = latest_path(self.dst_path.clone()).await?;
        match cmp_mtime(&latest_path, &self.mtime).await.ok() {
            Some(Ordering::Equal) => unreachable!(),
            Some(Ordering::Less) | Some(Ordering::Greater) => {
                let mut dst_file = File::open(&latest_path).await?;
                if file_eq(&mut file, &mut dst_file).await? {
                    file_set_modified(dst_file, self.mtime).await?;
                    fs::remove_file(&dl_path).await?;
                    return Ok(());
                } else {
                    self.dst_path = next_path(self.dst_path.clone()).await?;
                }
            }
            None => {}
        }

        file_set_modified(file, self.mtime).await?;
        fs::rename(dl_path, &self.dst_path).await?;
        Ok(())
    }
}

async fn cmp_mtime(path: &Path, mtime: &SystemTime) -> io::Result<Ordering> {
    fs::metadata(path).await.map(|metadata| {
        let file_mtime = metadata.modified().unwrap();
        file_mtime.cmp(mtime)
    })
}

async fn file_eq(file_a: &mut File, file_b: &mut File) -> io::Result<bool> {
    let (metadata_a, metadata_b) = tokio::join!(file_a.metadata(), file_b.metadata());

    if metadata_a?.len() != metadata_b?.len() {
        return Ok(false);
    }

    let (seek_a, seek_b) = tokio::join!(file_a.rewind(), file_b.rewind());
    seek_a?;
    seek_b?;

    let mut reader_a = BufReader::new(file_a);
    let mut reader_b = BufReader::new(file_b);

    loop {
        let (buf_a, buf_b) = tokio::join!(reader_a.fill_buf(), reader_b.fill_buf());
        let (buf_a, buf_b) = (buf_a?, buf_b?);
        if buf_a != buf_b {
            return Ok(false);
        }
        if buf_a.is_empty() {
            return Ok(buf_b.is_empty());
        }
        let (len_a, len_b) = (buf_a.len(), buf_b.len());
        reader_a.consume(len_a);
        reader_b.consume(len_b);
    }
}

async fn file_set_modified(file: File, mtime: SystemTime) -> io::Result<()> {
    let file = file.into_std().await;
    task::spawn_blocking(move || file.set_modified(mtime)).await??;
    Ok(())
}

fn alt_paths(path: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    (0..).map(|i| {
        let mut path = path.to_path_buf();
        path.push_file_prefix_suffix(format!("_new-{i}"));
        path
    })
}

async fn latest_path(path: PathBuf) -> io::Result<PathBuf> {
    let mut latest_path = path.clone();
    for path in alt_paths(&path) {
        if fs::try_exists(&path).await? {
            latest_path = path;
        } else {
            break;
        }
    }
    Ok(latest_path)
}

async fn next_path(path: PathBuf) -> io::Result<PathBuf> {
    for path in alt_paths(&path) {
        if !fs::try_exists(&path).await? {
            return Ok(path);
        }
    }
    unreachable!()
}

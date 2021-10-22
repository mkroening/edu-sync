use std::{
    borrow::Cow,
    future::Future,
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use dialoguer::{
    console::{self, Alignment},
    Confirm,
};
use edu_sync::{
    account::{Account, Token},
    config::{AccountConfig, Config},
    content::{Content, Download, FileDownload, SyncStatus},
};
use futures_util::{
    future,
    stream::{self, FuturesOrdered, FuturesUnordered},
    StreamExt,
};
use indicatif::{BinaryBytes, MultiProgress, ProgressBar, ProgressStyle};
use log::{info, trace};
use structopt::StructOpt;
use tokio::{task, time};

use crate::util;

/// Synchronizes available content from the configured courses.
#[derive(Debug, StructOpt)]
pub struct Subcommand {
    /// Bypass any and all “Are you sure?” messages. It’s not a good idea to do
    /// this unless you want to run edu-sync-cli from a script.
    #[structopt(long)]
    no_confirm: bool,
}

impl Subcommand {
    pub async fn run(self) -> anyhow::Result<()> {
        let config = Config::read().await?;

        if util::check_active_courses(&config) {
            let syncer = Syncer::from(config).await;
            syncer.sync(self.no_confirm).await?;
        }

        Ok(())
    }
}

struct Syncer {
    parallel_downloads: usize,
    outdated_courses: Vec<CourseStatus>,
}

impl Syncer {
    async fn from(config: Config) -> Self {
        println!("Requesting content databases...");
        let parallel_downloads = config.parallel_downloads;
        let outdated_courses = config
            .accounts
            .into_iter()
            .filter_map(|(account_name, account_config)| {
                let AccountConfig {
                    id, path, courses, ..
                } = account_config;
                Account::try_from(id)
                    .map(|account| (account, path, courses.0))
                    .map_err(|err| println!("Could not get password for {}: {}", account_name, err))
                    .ok()
            })
            .flat_map(|(account, account_path, courses)| {
                let account = Arc::new(account);
                courses
                    .into_iter()
                    .rev()
                    .filter(|(_, course_config)| course_config.sync)
                    .map(move |(course_id, course_config)| {
                        let course_path =
                            account_path.join(course_config.name_as_path_component().as_ref());
                        (account.clone(), course_id, course_config.name, course_path)
                    })
            })
            .map(|(account, course_id, course_name, course_path)| {
                tokio::spawn(async move {
                    let contents = account.get_contents(course_id, course_path).await;
                    CourseStatus::from_contents(contents, account.token(), course_name).await
                })
            })
            .collect::<FuturesOrdered<_>>()
            .filter_map(|res| future::ready(res.map_err(|err| println!("{}", err)).ok()))
            .filter(|course_status| future::ready(!course_status.downloads.is_empty()))
            .collect::<Vec<_>>()
            .await;
        Self {
            parallel_downloads,
            outdated_courses,
        }
    }

    async fn sync(self, no_confirm: bool) -> io::Result<()> {
        if self.outdated_courses.is_empty() {
            println!("All resources are up to date.");
        } else {
            println!();

            let size_width = 9;
            let pad_course_name = |course_name| {
                let width = 80 - size_width - 4 - 19;
                console::pad_str(course_name, width, Alignment::Left, Some("..."))
            };
            let pad_size = |size| {
                let message = if size > 0 {
                    Cow::from(BinaryBytes(size).to_string())
                } else {
                    Cow::from("N/A")
                };
                console::pad_str(message.as_ref(), size_width, Alignment::Right, None).into_owned()
            };

            let (count, size) = self
                .outdated_courses
                .iter()
                .map(|course| {
                    let count = course.downloads.len();
                    let size = course.downloads.iter().map(Download::size).sum();
                    let name = &course.name;
                    (count, size, name)
                })
                .inspect(|(count, size, name)| {
                    println!(
                        "{} {:>4} items, totalling {}",
                        pad_course_name(name),
                        count,
                        pad_size(*size)
                    );
                })
                .map(|(count, size, _name)| (count, size))
                .reduce(|(count_a, size_a), (count_b, size_b)| (count_a + count_b, size_a + size_b))
                .unwrap();

            let size = if size > 0 {
                Cow::from(BinaryBytes(size).to_string())
            } else {
                Cow::from("N/A")
            };

            println!();
            println!("Total: {} items, totalling {}", count, size);
            println!();

            let proceed = no_confirm
                || task::spawn_blocking(|| {
                    Confirm::new()
                        .with_prompt("Proceed with synchronization?")
                        .default(true)
                        .interact()
                })
                .await??;

            if proceed {
                println!("Downloading missing files...");
                self.download().await?;
            }
        }

        Ok(())
    }

    async fn download(self) -> io::Result<()> {
        let multi_progress = Arc::new(MultiProgress::new());
        let content_progress_style =
            ProgressStyle::default_bar().template("[{pos}/{len}] {wide_msg}");
        let size_progress_style = ProgressStyle::default_bar()
            .template(
                "└──── {binary_bytes:>9} / {binary_total_bytes:>9} [{bar:25}] \
                 {binary_bytes_per_sec:>11} in {elapsed:>3} ETA: {eta:>3}",
            )
            .progress_chars("=> ");

        let multi_progress_clone = multi_progress.clone();
        let download_tasks = self
            .outdated_courses
            .into_iter()
            .map(
                |CourseStatus {
                     token,
                     name,
                     downloads,
                 }| {
                    let multi_progress = multi_progress_clone.clone();
                    let content_progress_style = content_progress_style.clone();
                    let size_progress_style = size_progress_style.clone();
                    let content_progress = multi_progress.add(
                        ProgressBar::new(0)
                            .with_style(content_progress_style)
                            .with_message(name),
                    );
                    let size_progress =
                        multi_progress.add(ProgressBar::new(0).with_style(size_progress_style));
                    tokio::spawn(async move {
                        CourseDownload {
                            downloads,
                            token,
                            content_progress,
                            size_progress,
                        }
                        .run()
                        .await
                    })
                },
            )
            .collect::<FuturesUnordered<_>>();

        let total_bar = multi_progress.add(
            ProgressBar::new(0).with_style(
                ProgressStyle::default_bar()
                    .template(
                        "Total {binary_bytes:>9} / {binary_total_bytes:>9} [{bar:25}] \
                         {binary_bytes_per_sec:>11} in {elapsed:>3} ETA: {eta:>3}",
                    )
                    .progress_chars("=> "),
            ),
        );

        let joiner = task::spawn_blocking(move || multi_progress.join());
        let (file_downloads, content_downloads, size_progress, content_progress, size) =
            download_tasks
                .filter_map(|res| future::ready(res.map_err(|err| println!("{}", err)).ok()))
                .filter_map(|res| future::ready(res.map_err(|err| println!("{}", err)).ok()))
                .fold(
                    (Vec::new(), Vec::new(), Vec::new(), Vec::new(), 0),
                    |(
                        mut file_downloads,
                        mut content_downloads,
                        mut size_progress,
                        mut content_progress,
                        size,
                    ),
                     mut download| async move {
                        file_downloads.append(&mut download.file_downloads);
                        content_downloads.append(&mut download.content_downloads);
                        size_progress.push((download.download_progresses, download.size_progress));
                        content_progress.push(download.content_progress);
                        (
                            file_downloads,
                            content_downloads,
                            size_progress,
                            content_progress,
                            size + download.size,
                        )
                    },
                )
                .await;

        total_bar.set_length(size);

        let size_progresses = size_progress
            .iter()
            .map(|(_, bar)| bar)
            .cloned()
            .collect::<Vec<_>>();

        let file_downloads = stream::iter(file_downloads)
            .map(tokio::spawn)
            .buffer_unordered(self.parallel_downloads)
            .collect::<Vec<_>>();

        let total_bar_clone = total_bar.clone();
        let size = tokio::spawn(async move {
            let mut timer = time::interval(Duration::from_millis(200));
            loop {
                let mut total = 0;
                for (progresses, size_progress) in &size_progress {
                    let progress = progresses
                        .iter()
                        .map(|progress| progress.load(Ordering::Relaxed))
                        .sum();
                    size_progress.set_position(progress);
                    total += progress;
                }
                total_bar_clone.set_position(total);
                timer.tick().await;
            }
        });

        let content_downloads = content_downloads
            .into_iter()
            .map(tokio::spawn)
            .collect::<Vec<_>>();
        for file_download in file_downloads.await {
            file_download?;
        }
        for content_download in content_downloads {
            content_download.await?;
        }

        size.abort();
        for size_progress in size_progresses {
            size_progress.finish();
        }
        for content_progress in content_progress {
            content_progress.finish();
        }
        total_bar.finish();

        joiner.await??;

        Ok(())
    }
}

struct CourseStatus {
    token: Token,
    name: String,
    downloads: Vec<Download>,
}

impl CourseStatus {
    async fn from_contents(
        contents: impl Iterator<Item = Content> + Send,
        token: Token,
        name: String,
    ) -> Self {
        let downloads = contents
            .map(|content| {
                tokio::spawn(async move {
                    match content.sync().await {
                        SyncStatus::Downloadable(download) => Some(download),
                        SyncStatus::NotSupported(content_type, path) => {
                            info!(
                                "Not supported: ContentType::{:?} at {}",
                                content_type,
                                path.display()
                            );
                            None
                        }
                        SyncStatus::Outdated(path) => {
                            println!("Outdated: {}", path.display());
                            None
                        }
                        SyncStatus::UpToDate(path) => {
                            trace!("Up to date: {}", path.display());
                            None
                        }
                        SyncStatus::Modified(path) => {
                            println!("Modified: {}", path.display());
                            None
                        }
                    }
                })
            })
            .collect::<FuturesUnordered<_>>()
            .filter_map(|res| future::ready(res.map_err(|err| println!("{}", err)).ok()))
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        Self {
            token,
            name,
            downloads,
        }
    }
}

struct CourseDownload {
    downloads: Vec<Download>,
    token: Token,
    content_progress: ProgressBar,
    size_progress: ProgressBar,
}

struct CourseDownloads<F, C> {
    file_downloads: Vec<F>,
    content_downloads: Vec<C>,
    download_progresses: Vec<Arc<AtomicU64>>,
    size_progress: ProgressBar,
    size: u64,
    content_progress: ProgressBar,
}

impl CourseDownload {
    async fn run(
        self,
    ) -> io::Result<CourseDownloads<impl Future<Output = ()>, impl Future<Output = ()>>> {
        let Self {
            downloads,
            token,
            content_progress,
            size_progress,
        } = self;

        content_progress.set_length(downloads.len() as u64);

        let (file_downloads, content_downloads) = downloads
            .into_iter()
            .partition::<Vec<_>, _>(|download| matches!(download, Download::File(_)));

        let file_downloads = file_downloads
            .into_iter()
            .map(|file_download| match file_download {
                Download::File(file_download) => file_download,
                _ => unreachable!(),
            })
            .collect::<Vec<FileDownload>>();

        let download_size = file_downloads.iter().map(FileDownload::size).sum();
        size_progress.set_length(download_size);

        let progresses = file_downloads
            .iter()
            .map(|_| Arc::new(AtomicU64::new(0)))
            .collect::<Vec<_>>();
        let content_progress_clone = content_progress.clone();
        let file_downloads = file_downloads
            .into_iter()
            .zip(progresses.iter().cloned())
            .map(|(mut file_download, progress)| {
                let content_progress = content_progress_clone.clone();
                async move {
                    file_download
                        .run(&token, |val| progress.store(val, Ordering::Relaxed))
                        .await
                        .unwrap();
                    content_progress.inc(1);
                    let path = file_download.path().display().to_string();
                    content_progress.println(&path);
                }
            })
            .collect::<Vec<_>>();

        let content_progress_clone = content_progress.clone();
        let content_downloads = content_downloads
            .into_iter()
            .map(|download| {
                let content_progress = content_progress_clone.clone();
                async move {
                    match download {
                        Download::File(_) => unreachable!(),
                        Download::Url(mut url_download) => {
                            url_download.run().await.unwrap();
                            content_progress.inc(1);
                            let path = url_download.path().display().to_string();
                            content_progress.println(&path);
                        }
                        Download::Content(mut content_download) => {
                            content_download.run().await.unwrap();
                            content_progress.inc(1);
                            let path = content_download.path().display().to_string();
                            content_progress.println(&path);
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

        Ok(CourseDownloads {
            file_downloads,
            content_downloads,
            download_progresses: progresses,
            size_progress,
            size: download_size,
            content_progress,
        })
    }
}

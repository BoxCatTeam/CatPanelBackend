use std::future::Future;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use chrono::{Datelike, Local};
use smallvec::SmallVec;
use tokio::fs::{create_dir_all, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

const LOG_DIR: &str = "logs";

pub fn init_tracing_subscriber() -> impl Future<Output = anyhow::Result<()>> {
    let mut layers = Vec::with_capacity(2);

    layers.push(
        tracing_subscriber::fmt::layer()
            .compact()
            .with_ansi(true)
            .with_filter(
                std::env::var("LOG_LEVEL")
                    .ok()
                    .and_then(|var| LevelFilter::from_str(&var).ok())
                    .unwrap_or(LevelFilter::INFO),
            )
            .boxed(),
    );

    let (w, tx, notify) = MakeNonBlockingLogFileWriter::new();

    layers.push(
        tracing_subscriber::fmt::layer()
            .json()
            .flatten_event(true)
            .with_ansi(false)
            .with_file(true)
            .with_line_number(true)
            .with_thread_names(true)
            .with_thread_ids(true)
            .with_current_span(true)
            .with_writer(w)
            .with_filter(
                std::env::var("LOG_FILE_LEVEL")
                    .ok()
                    .and_then(|var| LevelFilter::from_str(&var).ok())
                    .unwrap_or(LevelFilter::DEBUG),
            )
            .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                metadata.target() != "log_file_writer"
            }))
            .boxed(),
    );

    tracing_subscriber::registry().with(layers).init();

    async move {
        tx.send(Msg::Shutdown).await.unwrap();
        notify.notified().await;
        Ok(())
    }
}

struct MakeNonBlockingLogFileWriter {
    sender: Sender<Msg>,
}

#[derive(Debug)]
enum Msg {
    Buf(SmallVec<[u8; 128]>),
    Shutdown,
}

struct NonBlockingLogFileWriter {
    sender: Sender<Msg>,
}

struct LogFileWriter {
    file: File,
    date_time: chrono::DateTime<Local>,
}

impl LogFileWriter {
    async fn new() -> anyhow::Result<Self> {
        create_dir_all(LOG_DIR)
            .await
            .with_context(|| "创建日志文件夹失败")?;

        let now = Local::now();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(Path::new(LOG_DIR).join(format!("{}.log", now.format("%Y-%m-%d"))))
            .await
            .with_context(|| "打开日志文件失败")?;

        Ok(LogFileWriter {
            file,
            date_time: now,
        })
    }

    async fn check_date(&mut self) -> anyhow::Result<()> {
        let now = Local::now();
        if (
            self.date_time.year(),
            self.date_time.month(),
            self.date_time.day(),
        ) != (now.year(), now.month(), now.day())
        {
            *self = Self::new().await?
        }
        Ok(())
    }

    async fn write(&mut self, buf: &[u8]) -> anyhow::Result<()> {
        self.check_date().await?;
        self.file.write_all(buf).await?;
        Ok(())
    }
}

impl std::io::Write for NonBlockingLogFileWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // 如果在tokio的工作线程中
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let sender = self.sender.clone();
            let owned_buf = SmallVec::from_slice(buf);
            handle.spawn(async move {
                if let Err(SendError(Msg::Buf(err))) = sender
                    .send(Msg::Buf(owned_buf))
                    .await {
                    tracing::error!(target: "log_file_writer", "日志文件写入器已经关闭但仍然试图写入: {:?}", std::str::from_utf8(&err));
                }
            });
        } else if let Err(SendError(Msg::Buf(err))) = self
            .sender
            .blocking_send(Msg::Buf(SmallVec::from_slice(buf)))
        {
            tracing::error!(target: "log_file_writer", "日志文件写入器已经关闭但仍然试图写入: {:?}", std::str::from_utf8(&err));
        }

        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.write(buf).map(|_| ())
    }
}

impl MakeNonBlockingLogFileWriter {
    pub fn new() -> (Self, Sender<Msg>, Arc<Notify>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let notify = Arc::new(Notify::new());
        let notify2 = notify.clone();

        tokio::task::spawn(async move {
            let mut writer = match LogFileWriter::new().await {
                Ok(w) => w,
                Err(err) => {
                    tracing::error!(target: "log_file_writer", "日志文件写入器失败: {}", err);
                    panic!("{}", err)
                }
            };

            while let Some(msg) = rx.recv().await {
                match msg {
                    Msg::Buf(buf) => {
                        if let Err(err) = writer.write(&buf).await {
                            tracing::error!(target: "log_file_writer", "写入日志文件时发生错误: {}", err);
                        }
                    }
                    Msg::Shutdown => {
                        rx.close();
                    }
                }
            }
            notify2.notify_one();
        });

        (
            MakeNonBlockingLogFileWriter { sender: tx.clone() },
            tx,
            notify,
        )
    }
}

impl<'a> MakeWriter<'a> for MakeNonBlockingLogFileWriter {
    type Writer = NonBlockingLogFileWriter;

    fn make_writer(&'a self) -> Self::Writer {
        NonBlockingLogFileWriter {
            sender: self.sender.clone(),
        }
    }
}

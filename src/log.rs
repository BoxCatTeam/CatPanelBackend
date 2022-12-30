use std::fmt::Arguments;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::IoSlice;
use std::path::Path;

use chrono::{Datelike, Local};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

const LOG_DIR: &str = "logs";

#[allow(clippy::vec_init_then_push)]
pub fn init_tracing_subscriber() {
    let mut layers = Vec::with_capacity(2);

    layers.push(
        tracing_subscriber::fmt::layer()
            .compact()
            .with_ansi(true)
            .with_filter(LevelFilter::INFO)
            .boxed(),
    );

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
            .with_writer(MakeLogFileWriter)
            .with_filter(LevelFilter::DEBUG)
            .boxed(),
    );

    tracing_subscriber::registry().with(layers).init();
}

struct MakeLogFileWriter;

struct LogFileWriter {
    file: File,
    date_time: chrono::DateTime<Local>,
}

impl LogFileWriter {
    fn new() -> Self {
        create_dir_all(LOG_DIR).expect("创建日志文件夹失败");

        let now = Local::now();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(Path::new(LOG_DIR).join(format!("{}.log", now.format("%Y-%m-%d"))))
            .expect("打开日志文件失败");

        LogFileWriter {
            file,
            date_time: now,
        }
    }

    fn check_date(&mut self) -> std::io::Result<()> {
        let now = Local::now();
        if (
            self.date_time.year(),
            self.date_time.month(),
            self.date_time.day(),
        ) != (now.year(), now.month(), now.day())
        {
            *self = Self::new()
        }
        Ok(())
    }
}

impl std::io::Write for LogFileWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.check_date()?;
        self.file.write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> std::io::Result<usize> {
        self.check_date()?;
        self.file.write_vectored(bufs)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.check_date()?;
        self.file.flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.check_date()?;
        self.file.write_all(buf)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: Arguments<'_>) -> std::io::Result<()> {
        self.check_date()?;
        self.file.write_fmt(fmt)
    }

    #[inline]
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

impl<'a> MakeWriter<'a> for MakeLogFileWriter {
    type Writer = LogFileWriter;

    fn make_writer(&'a self) -> Self::Writer {
        Self::Writer::new()
    }
}

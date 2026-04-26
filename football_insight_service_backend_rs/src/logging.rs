use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use std::sync::{Arc, Mutex};

const LOG_DIR_NAME: &str = "logs";
const LOG_FILE_PREFIX: &str = "app";
const LOG_FILE_SUFFIX: &str = "log";
const LOG_FILE_MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;
const LOG_FILE_MAX_ROTATED_FILES: usize = 100;

pub fn init_tracing() {
    let log_dir = Path::new(LOG_DIR_NAME);
    if !log_dir.exists() {
        fs::create_dir_all(log_dir).expect("failed to create logs directory");
    }

    let file_appender = RollingFileAppender::new(
        log_dir.to_path_buf(),
        LOG_FILE_PREFIX.to_string(),
        LOG_FILE_SUFFIX.to_string(),
        LOG_FILE_MAX_SIZE_BYTES,
        LOG_FILE_MAX_ROTATED_FILES,
    );
    let shared_file_appender = Arc::new(Mutex::new(file_appender));
    let file_writer = {
        let shared_file_appender = Arc::clone(&shared_file_appender);
        move || SharedLogWriter::new(Arc::clone(&shared_file_appender))
    };

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(false)
        .init();

    tracing::info!(
        log_path = %log_dir.join(format!("{LOG_FILE_PREFIX}.{LOG_FILE_SUFFIX}")).display(),
        "tracing initialized with file logging"
    );
}

fn current_log_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string()
}

struct SharedLogWriter {
    appender: Arc<Mutex<RollingFileAppender>>,
}

impl SharedLogWriter {
    fn new(appender: Arc<Mutex<RollingFileAppender>>) -> Self {
        Self { appender }
    }
}

impl Write for SharedLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        std::io::stdout().write_all(buf)?;
        let written = self
            .appender
            .lock()
            .expect("log appender mutex poisoned")
            .write(buf)?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        std::io::stdout().flush()?;
        self.appender
            .lock()
            .expect("log appender mutex poisoned")
            .flush()
    }
}

struct RollingFileAppender {
    log_dir: PathBuf,
    prefix: String,
    suffix: String,
    max_size: u64,
    max_files: usize,
    current_file: File,
    current_size: u64,
}

impl RollingFileAppender {
    fn new(
        log_dir: PathBuf,
        prefix: String,
        suffix: String,
        max_size: u64,
        max_files: usize,
    ) -> Self {
        let current_path = log_dir.join(format!("{prefix}.{suffix}"));
        let current_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&current_path)
            .expect("failed to open log file");

        let current_size = current_file.metadata().map(|meta| meta.len()).unwrap_or(0);

        Self {
            log_dir,
            prefix,
            suffix,
            max_size,
            max_files,
            current_file,
            current_size,
        }
    }

    fn rotate(&mut self) {
        let timestamp = current_log_timestamp();
        let rotated_name = format!("{}.{}.{}", self.prefix, timestamp, self.suffix);
        let rotated_path = self.log_dir.join(rotated_name);

        drop(std::mem::replace(
            &mut self.current_file,
            File::open("/dev/null").expect("failed to open /dev/null during log rotation"),
        ));

        let current_path = self
            .log_dir
            .join(format!("{}.{}", self.prefix, self.suffix));

        if let Err(error) = fs::rename(&current_path, &rotated_path) {
            eprintln!("failed to rotate log file: {error}");
            return;
        }

        self.current_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&current_path)
            .expect("failed to create new log file after rotation");
        self.current_size = 0;
        self.cleanup_old_files();
    }

    fn cleanup_old_files(&mut self) {
        let Ok(entries) = fs::read_dir(&self.log_dir) else {
            return;
        };

        let mut files: Vec<(String, std::time::SystemTime)> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                name.starts_with(&format!("{}.", self.prefix))
                    && name.ends_with(&format!(".{}", self.suffix))
                    && name != format!("{}.{}", self.prefix, self.suffix)
            })
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                let modified_at = entry.metadata().ok()?.modified().ok()?;
                Some((name, modified_at))
            })
            .collect();

        files.sort_by(|a, b| b.1.cmp(&a.1));

        for (name, _) in files.into_iter().skip(self.max_files) {
            let path = self.log_dir.join(name);
            let _ = fs::remove_file(path);
        }
    }
}

impl Write for RollingFileAppender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.current_size >= self.max_size {
            self.rotate();
        }

        let written = self.current_file.write(buf)?;
        self.current_size += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.current_file.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::{EnvFilter, layer::SubscriberExt};
    use uuid::Uuid;

    fn temp_log_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("football-insight-log-{name}-{}", Uuid::new_v4()))
    }

    #[test]
    fn rolling_file_appender_writes_to_current_log_file() {
        let log_dir = temp_log_dir("write");
        fs::create_dir_all(&log_dir).expect("create temp log dir");

        let mut appender = RollingFileAppender::new(
            log_dir.clone(),
            "app".to_string(),
            "log".to_string(),
            1024,
            10,
        );

        appender.write_all(b"hello log").expect("write log");
        appender.flush().expect("flush log");

        let content = fs::read_to_string(log_dir.join("app.log")).expect("read app.log");
        assert_eq!(content, "hello log");

        fs::remove_dir_all(log_dir).ok();
    }

    #[test]
    fn rolling_file_appender_rotates_when_size_limit_is_reached() {
        let log_dir = temp_log_dir("rotate");
        fs::create_dir_all(&log_dir).expect("create temp log dir");

        let mut appender =
            RollingFileAppender::new(log_dir.clone(), "app".to_string(), "log".to_string(), 5, 10);

        appender.write_all(b"12345").expect("write first payload");
        appender.write_all(b"6789").expect("write second payload");
        appender.flush().expect("flush log");

        let entries = fs::read_dir(&log_dir)
            .expect("read log dir")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert!(
            entries
                .iter()
                .filter(|name| name.starts_with("app.") && name.ends_with(".log"))
                .count()
                >= 2,
            "expected current log and rotated log, got {entries:?}"
        );

        fs::remove_dir_all(log_dir).ok();
    }

    #[test]
    fn runtime_style_dual_layers_still_write_to_file() {
        let log_dir = temp_log_dir("dual-layers");
        fs::create_dir_all(&log_dir).expect("create temp log dir");

        let file_appender = Arc::new(Mutex::new(RollingFileAppender::new(
            log_dir.clone(),
            "app".to_string(),
            "log".to_string(),
            1024,
            10,
        )));
        let file_writer = {
            let file_appender = Arc::clone(&file_appender);
            move || SharedLogWriter::new(Arc::clone(&file_appender))
        };

        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new("info"))
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file_writer)
                    .with_ansi(false)
                    .with_target(false),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::sink)
                    .with_target(false),
            );

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("runtime style dual layer test");
        });

        let content = fs::read_to_string(log_dir.join("app.log")).expect("read app.log");
        assert!(
            content.contains("runtime style dual layer test"),
            "expected file log, got {content:?}"
        );

        fs::remove_dir_all(log_dir).ok();
    }
}

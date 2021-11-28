pub mod warp;

use std::{error::Error as StdError, fmt, io, process::Stdio, time::Duration};

use futures::future::try_join3;
use log::debug;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::Semaphore,
    time::timeout,
};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Dot(String),
    Timeout,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::Dot(s) => s.splitn(2, "<stdin>: ").try_for_each(|s| s.fmt(f)),
            Self::Timeout => write!(f, "operation timed out"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Dot(_) | Self::Timeout => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub struct ProcessCount {
    count: usize,
}

impl ProcessCount {
    pub const fn new(count: usize) -> Self {
        ProcessCount { count }
    }
}

pub trait Output: From<Vec<u8>> {
    fn type_name() -> &'static str;
}

struct Graphviz {
    timeout: Duration,
    semaphore: Semaphore,
}

impl Graphviz {
    pub const fn new(timeout: Duration, limit: ProcessCount) -> Self {
        Self {
            timeout,
            semaphore: Semaphore::const_new(limit.count),
        }
    }

    pub async fn dot<T: Output>(&self, input: &[u8]) -> Result<T, Error> {
        self.command("dot", T::type_name(), input)
            .await
            .map(Into::into)
    }

    pub async fn dot_with_format(
        &self,
        input: &[u8],
        format: Format,
    ) -> Result<OutputFormat, Error> {
        match format {
            Format::Gif => Ok(OutputFormat::Gif(self.dot(input).await?)),
            Format::Jpg => Ok(OutputFormat::Jpg(self.dot(input).await?)),
            Format::Pdf => Ok(OutputFormat::Pdf(self.dot(input).await?)),
            Format::Png => Ok(OutputFormat::Png(self.dot(input).await?)),
            Format::Postscript => Ok(OutputFormat::Postscript(self.dot(input).await?)),
            Format::Svg => Ok(OutputFormat::Svg(self.dot(input).await?)),
        }
    }

    pub fn is_rate_limited(&self) -> bool {
        self.semaphore.available_permits() == 0
    }

    async fn command(&self, name: &str, type_name: &str, input: &[u8]) -> Result<Vec<u8>, Error> {
        let mut command = Command::new(name);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-T")
            .arg(type_name);

        // need to start the child process in the timeout block, but we need to
        // kill it if the timeout is reached, so need some way of keeping ahold
        // of it after the timeout block. Also, to properly enforce the process
        // limit the permit can't be dropped until after the child process is
        // finished/killed
        let mut state = None;

        let result = timeout(self.timeout, async {
            let permit = self
                .semaphore
                .acquire()
                .await
                .expect("semaphore isn't ever closed");

            debug!("acquired permit {:?}", permit);

            state = Some((permit, command.spawn()?));
            let dot = match state {
                Some((_, ref mut d)) => d,
                None => unreachable!(),
            };

            dot.stdin.take().unwrap().write(input).await?;

            let mut stdout = Vec::new();
            let mut stdout_io = dot.stdout.take().unwrap();
            let stdout_f = stdout_io.read_to_end(&mut stdout);

            let mut stderr = Vec::new();
            let mut stderr_io = dot.stderr.take().unwrap();
            let stderr_f = stderr_io.read_to_end(&mut stderr);

            debug!("waiting for process {:?}", dot.id());
            let (status, _, _) = try_join3(dot.wait(), stdout_f, stderr_f).await?;

            if status.success() {
                Ok(stdout)
            } else {
                Err(Error::Dot(String::from_utf8_lossy(&stderr).into_owned()))
            }
        })
        .await;

        match result {
            Ok(v) => v,
            Err(_) => {
                if let Some((_, mut dot)) = state {
                    let _ = dot.kill().await;
                }
                Err(Error::Timeout)
            }
        }
    }
}

static DEFAULT: Graphviz = Graphviz::new(Duration::from_secs(4), ProcessCount::new(64));

pub async fn dot_with_format(input: &[u8], format: Format) -> Result<OutputFormat, Error> {
    DEFAULT.dot_with_format(input, format).await
}

pub fn is_rate_limited() -> bool {
    DEFAULT.is_rate_limited()
}

macro_rules! output_type {
    ($name:ident, $id:literal) => {
        pub struct $name(Vec<u8>);

        impl Output for $name {
            fn type_name() -> &'static str {
                $id
            }
        }

        impl From<Vec<u8>> for $name {
            fn from(val: Vec<u8>) -> Self {
                Self(val)
            }
        }
    };
}

output_type!(Gif, "gif");
output_type!(Jpg, "jpg");
output_type!(Pdf, "pdf");
output_type!(Png, "png");
output_type!(Postscript, "ps");
output_type!(Svg, "svg");

#[derive(Clone, Copy)]
pub enum Format {
    Gif,
    Jpg,
    Pdf,
    Png,
    Postscript,
    Svg,
}

impl Format {
    pub fn from_content_type(s: &str) -> Option<Self> {
        match s {
            "image/gif" => Some(Format::Gif),
            "image/jpeg" => Some(Format::Jpg),
            "application/pdf" => Some(Format::Pdf),
            "image/png" => Some(Format::Png),
            "application/postscript" => Some(Format::Postscript),
            "image/svg+xml" => Some(Format::Svg),
            _ => None,
        }
    }
}

pub enum OutputFormat {
    Gif(Gif),
    Jpg(Jpg),
    Pdf(Pdf),
    Png(Png),
    Postscript(Postscript),
    Svg(Svg),
}

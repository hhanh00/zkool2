use std::sync::Mutex;

use flutter_rust_bridge::frb;
use tracing_subscriber::{
    fmt::{
        self,
        format::{self, FmtSpan, Format},
        MakeWriter,
    },
    layer::SubscriberExt as _,
    util::SubscriberInitExt as _,
    EnvFilter, Layer, Registry,
};

use crate::frb_generated::StreamSink;

#[frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
    let _ = env_logger::builder().try_init();
    let _ = Registry::default()
        .with(default_layer())
        .with(env_layer())
        .with(frb_layer())
        .try_init();
    let _ = rustls::crypto::ring::default_provider().install_default();
}

type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;

fn default_layer<S>() -> BoxedLayer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fmt::layer()
        .with_ansi(false)
        .with_span_events(FmtSpan::ACTIVE)
        .compact()
        .boxed()
}

fn env_layer<S>() -> BoxedLayer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    EnvFilter::from_default_env().boxed()
}

struct FrbWriter {}

impl FrbWriter {
    fn new() -> Self {
        FrbWriter {}
    }
}

impl std::io::Write for FrbWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message = String::from_utf8_lossy(buf);
        let sink = LOG_SINK.lock().unwrap();
        if let Some(sink) = sink.as_ref() {
            let message = message.to_string();
            let log_message = LogMessage { level: 0, message };
            let _ = sink.add(log_message);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct FrbMakeWriter {}

impl<'a> MakeWriter<'a> for FrbMakeWriter {
    type Writer = FrbWriter;

    fn make_writer(&'a self) -> Self::Writer {
        FrbWriter::new()
    }
}

fn frb_layer<S>(
) -> fmt::Layer<S, format::DefaultFields, format::Format<format::Full, ()>, FrbMakeWriter>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fmt::Layer::new()
        .event_format(
            Format::default()
                .with_level(false)
                .with_ansi(false)
                .without_time(),
        )
        .with_writer(FrbMakeWriter {})
}

#[frb(dart_metadata = ("freezed"))]
pub struct LogMessage {
    pub level: u8,
    pub message: String,
}

#[frb(sync)]
pub fn set_log_stream(s: StreamSink<LogMessage>) {
    let mut sink = LOG_SINK.lock().unwrap();
    *sink = Some(s);
}

lazy_static::lazy_static! {
    static ref LOG_SINK: Mutex<Option<StreamSink<LogMessage>>> = Mutex::new(None);
}

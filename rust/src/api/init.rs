use std::sync::{Mutex, OnceLock};

use flutter_rust_bridge::frb;
use tracing::{level_filters::LevelFilter, Event, Level, Subscriber};
use tracing_subscriber::{
    field::MakeVisitor,
    fmt::{
        self,
        format::{FmtSpan, Writer},
    },
    layer::{Context, SubscriberExt as _},
    registry::LookupSpan,
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
    tracing::info!("Rust logging initialized");
}

pub type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;

pub fn default_layer<S>() -> BoxedLayer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fmt::layer()
        .with_ansi(false)
        .with_span_events(FmtSpan::ACTIVE)
        .compact()
        .boxed()
}

pub fn env_layer<S>() -> BoxedLayer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .boxed()
}

fn frb_layer<S>() -> BoxedLayer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    FrbLogger {}.boxed()
}

struct FrbLogger;

impl<S> Layer<S> for FrbLogger
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event, ctx: Context<S>) {
        let mut message = String::new();
        let writer = Writer::new(&mut message);
        let mut visitor =
            tracing_subscriber::fmt::format::DefaultFields::default().make_visitor(writer);
        event.record(&mut visitor);
        let level: u8 = match *event.metadata().level() {
            Level::ERROR => 4,
            Level::WARN => 3,
            Level::INFO => 2,
            Level::DEBUG => 1,
            Level::TRACE => 0,
        };
        let span = ctx.lookup_current().map(|s| s.name().to_string());
        let log = LogMessage {
            level,
            message,
            span,
        };
        if let Some(sink) = LOG_SINK.get() {
            let _ = sink.lock().unwrap().add(log);
        }
    }
}

#[frb(dart_metadata = ("freezed"))]
pub struct LogMessage {
    pub level: u8,
    pub message: String,
    pub span: Option<String>,
}

#[frb(sync)]
pub fn set_log_stream(s: StreamSink<LogMessage>) {
    println!("Setting log stream");
    let _ = LOG_SINK.set(Mutex::new(s));
}

static LOG_SINK: OnceLock<Mutex<StreamSink<LogMessage>>> = OnceLock::new();

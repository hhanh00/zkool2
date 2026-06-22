use std::sync::{Mutex, OnceLock};

use tracing::{level_filters::LevelFilter, Event, Level, Subscriber};
use tracing_subscriber::{
    field::MakeVisitor,
    fmt::{
        self,
        format::{FmtSpan, Writer},
    },
    layer::{Context, Layered, SubscriberExt as _},
    registry::LookupSpan,
    reload,
    util::SubscriberInitExt as _,
    EnvFilter, Layer, Registry,
};

#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

/// The subscriber type at the point where the reload filter layer is applied
/// (after `default_layer` but before `frb_layer`).
type FilterSub = Layered<BoxedLayer<Registry>, Registry>;

#[cfg(feature = "flutter")]
static FILTER_HANDLE: OnceLock<reload::Handle<EnvFilter, FilterSub>> = OnceLock::new();

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb(init))]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
    let _ = env_logger::builder().try_init();

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let (filter_layer, reload_handle) = reload::Layer::new(env_filter);
    FILTER_HANDLE.set(reload_handle).ok();

    let _ = Registry::default()
        .with(default_layer())
        .with(filter_layer)
        .with(frb_layer())
        .try_init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    tracing::info!("Rust logging initialized");
}

/// Enable expert mode, which lowers the log filter to allow
/// sync, mempool, and memo target debug messages
/// while keeping everything else at `info`.
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn set_expert_mode(enabled: bool) {
    if let Some(handle) = FILTER_HANDLE.get() {
        let filter = if enabled {
            EnvFilter::new("warp=debug,rlz=debug,info")
        } else {
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
        };
        let _ = handle.modify(|f| *f = filter);
    }
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

#[cfg(feature = "flutter")]
fn frb_layer<S>() -> BoxedLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    FrbLogger {}.boxed()
}

struct FrbLogger;

#[cfg(feature = "flutter")]
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

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct LogMessage {
    pub level: u8,
    pub message: String,
    pub span: Option<String>,
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn set_log_stream(s: StreamSink<LogMessage>) {
    println!("Setting log stream");
    let _ = LOG_SINK.set(Mutex::new(s));
}

#[cfg(feature = "flutter")]
static LOG_SINK: OnceLock<Mutex<StreamSink<LogMessage>>> = OnceLock::new();

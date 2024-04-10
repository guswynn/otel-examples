use clap::Parser;
use tracing::Instrument;
use tracing::{debug_span, info_span};

use ext::OpenTelemetryContext;

mod ext;
mod setup;

#[derive(clap::Parser)]
struct Args {
    #[clap(short, long)]
    fmt_debug: bool,
    #[clap(short, long)]
    otel_debug: bool,
    #[clap(short, long)]
    weird_case: bool,
    #[clap(short, long, default_value = "")]
    name: String,
}

async fn wait_a_bit() {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
}

async fn parent_child_overlaps() {
    wait_a_bit().await;
    let p = info_span!("parent_overlaps");

    let ctx = p.in_scope(|| OpenTelemetryContext::obtain());

    wait_a_bit().await;

    let c = info_span!("child_overlaps");

    wait_a_bit().await;

    c.in_scope(|| ctx.attach_as_parent());

    wait_a_bit().await;

    drop(p);

    wait_a_bit().await;
}

async fn parent_child_before() {
    wait_a_bit().await;
    let p = info_span!("parent_before");

    let ctx = p.in_scope(|| OpenTelemetryContext::obtain());
    wait_a_bit().await;
    drop(p);

    wait_a_bit().await;

    let c = info_span!("child_after");
    c.in_scope(|| ctx.attach_as_parent());

    wait_a_bit().await;
}

async fn follows_from_overlaps() {
    wait_a_bit().await;
    let p = info_span!("follow_from_overlaps");

    let ctx = p.in_scope(|| OpenTelemetryContext::obtain());

    wait_a_bit().await;

    let c = info_span!("follows_from_child_overlaps");

    wait_a_bit().await;

    c.in_scope(|| ctx.attach_as_link());

    wait_a_bit().await;

    drop(p);

    wait_a_bit().await;
}

async fn follows_from_before() {
    wait_a_bit().await;
    let p = info_span!("follows_from_before");

    let ctx = p.in_scope(|| OpenTelemetryContext::obtain());
    wait_a_bit().await;
    drop(p);

    wait_a_bit().await;

    // Note this is in the same trace tree!
    let c = info_span!("follows_from_after");
    c.in_scope(|| ctx.attach_as_link());

    wait_a_bit().await;
}

async fn follows_from_root() {
    wait_a_bit().await;
    let p = info_span!("follows_from_root");

    let ctx = p.in_scope(|| OpenTelemetryContext::obtain());
    wait_a_bit().await;
    drop(p);

    wait_a_bit().await;

    // Note this is in the same trace tree!
    let c = info_span!(parent: None,  "follows_from_child_root");
    c.in_scope(|| ctx.attach_as_link());

    wait_a_bit().await;
}

#[tracing::instrument]
async fn instrument_parent() {
    wait_a_bit().await;

    #[tracing::instrument]
    async fn child_func() {
        wait_a_bit().await;
    }

    let jh = tokio::task::spawn(child_func().instrument(tracing::Span::current()));

    let _ = jh.await;
    wait_a_bit().await;
}

async fn enter_exit() {
    wait_a_bit().await;

    let p = info_span!("enter_exit");

    let one = p.enter();
    wait_a_bit().await;
    drop(one);

    wait_a_bit().await;

    let two = p.enter();
    wait_a_bit().await;
    drop(two);
}

async fn weird_case(name: String) {
    wait_a_bit().await;

    let p = info_span!("info_parent", %name);

    let _p = p.entered();
    wait_a_bit().await;
    let c = debug_span!("debug_child", %name);
    let _c = c.entered();
    wait_a_bit().await;

    let oc = OpenTelemetryContext::obtain();
    let dc = info_span!(parent: None, "disconnected_child", %name);
    dc.in_scope(|| {
        oc.attach_as_parent();
    });
    drop(_c);

    wait_a_bit().await;
    drop(dc);
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    setup::configure(args.fmt_debug, args.otel_debug);

    if args.weird_case {
        weird_case(args.name).await;
        tokio::time::sleep(std::time::Duration::from_secs(100)).await;
        return;
    }

    let guard = info_span!("top_level").entered();

    tokio::join!(parent_child_overlaps(), parent_child_before());
    wait_a_bit().await;
    tokio::join!(follows_from_overlaps(), follows_from_before());
    wait_a_bit().await;
    follows_from_root().await;
    wait_a_bit().await;
    instrument_parent().await;
    wait_a_bit().await;
    enter_exit().await;

    drop(guard);

    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
}

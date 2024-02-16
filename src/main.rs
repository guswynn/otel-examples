use tracing::info_span;

use ext::OpenTelemetryContext;

mod ext;
mod setup;

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

#[tokio::main]
async fn main() {
    setup::configure();

    let guard = info_span!("top_level").entered();

    tokio::join!(parent_child_overlaps(), parent_child_before());
    wait_a_bit().await;
    tokio::join!(follows_from_overlaps(), follows_from_before());
    wait_a_bit().await;
    follows_from_root().await;

    drop(guard);

    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
}

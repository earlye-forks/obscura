/// Drains the global DOM-mutation telemetry stream (feature-011) from a
/// background thread while the browser drives a page, showing how a
/// companion automation process can observe DOM changes without blocking
/// or slowing down Obscura's own engine loop.
use obscura::{Browser, TelemetryDomEvent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Subscribe before driving any page: the channel is single-consumer, so
    // `subscribe_dom_changes()` returns `None` on every call after the first.
    let mut events = obscura::subscribe_dom_changes().expect("dom telemetry already subscribed");

    // `blocking_recv` parks this thread, not the tokio runtime or the V8
    // engine loop, so it never competes with page execution.
    std::thread::spawn(move || {
        while let Some(event) = events.blocking_recv() {
            match event {
                TelemetryDomEvent::NodeInserted { node_id, parent_id } => {
                    println!("[dom] node {node_id} inserted under {parent_id}");
                }
                TelemetryDomEvent::NodeRemoved { node_id } => {
                    println!("[dom] node {node_id} removed");
                }
                TelemetryDomEvent::AttributeChanged { node_id, name } => {
                    println!("[dom] node {node_id} attribute `{name}` changed");
                }
            }
        }
    });

    let browser = Browser::builder().build()?;
    let mut page = browser.new_page().await?;
    page.goto("https://example.com").await?;
    page.evaluate("document.body.appendChild(document.createElement('div'))");
    page.settle(200).await;

    Ok(())
}

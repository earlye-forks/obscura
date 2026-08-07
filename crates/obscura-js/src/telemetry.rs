//! Global DOM-mutation event stream.
//!
//! `op_dom`'s mutation commands run on V8's thread-local, `!Send` engine
//! loop, so they can't hold a direct reference to a consumer living on
//! another thread. Instead they push lightweight [`TelemetryDomEvent`]
//! values into a process-wide `tokio::sync::mpsc` queue that an external
//! automation thread drains via [`subscribe_dom_changes`]. `send` on an
//! unbounded sender never blocks and never locks, so this is safe to call
//! from inside a `#[op2]` body.

use std::sync::{Mutex, OnceLock};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// A DOM structural mutation, reported with only the identifiers needed to
/// look the change back up — never the node's content — so emitting one
/// inside the V8 execution path stays cheap regardless of subtree size.
#[derive(Debug, Clone)]
pub enum TelemetryDomEvent {
    NodeInserted { node_id: u32, parent_id: u32 },
    NodeRemoved { node_id: u32 },
    AttributeChanged { node_id: u32, name: String },
}

struct Channel {
    sender: UnboundedSender<TelemetryDomEvent>,
    receiver: Mutex<Option<UnboundedReceiver<TelemetryDomEvent>>>,
}

static CHANNEL: OnceLock<Channel> = OnceLock::new();

fn channel() -> &'static Channel {
    CHANNEL.get_or_init(|| {
        let (sender, receiver) = unbounded_channel();
        Channel { sender, receiver: Mutex::new(Some(receiver)) }
    })
}

/// Fire-and-forget dispatch used by `op_dom`'s mutation commands. Never
/// blocks; silently drops the event if nobody is subscribed yet (an
/// unbounded sender only errs once every receiver has been dropped, which
/// can't happen here since the global `Channel` holds one for the process
/// lifetime).
pub fn emit_dom_event(event: TelemetryDomEvent) {
    let _ = channel().sender.send(event);
}

/// Take the process-wide DOM-mutation receiver. The underlying channel is
/// single-consumer, so this returns `Some` on the first call and `None`
/// afterwards — hand the receiver to whichever task or thread owns
/// consuming the stream, then keep it there.
pub fn subscribe_dom_changes() -> Option<UnboundedReceiver<TelemetryDomEvent>> {
    channel().receiver.lock().unwrap().take()
}

//! A dumb TCP listener used by the proxy integration tests.
//!
//! It is not a real proxy — it accepts a connection, records that fact,
//! reads/discards a handful of bytes, then closes. That is enough for a
//! `cargo-generate` subprocess to prove that its clone attempt was routed
//! through the listener's address rather than going direct, which is the
//! only claim these tests make.
//!
//! Works uniformly for `http://`, `https://` and `socks5://` proxy URLs
//! because the assertion happens at the TCP-accept layer, below any of the
//! respective handshakes.

use std::io::Read;
use std::net::{SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

pub struct FakeProxy {
    addr: SocketAddr,
    hit: Arc<AtomicBool>,
    // Keeps the accept loop alive for the lifetime of this handle; joined on drop.
    _thread: Option<JoinHandle<()>>,
}

impl FakeProxy {
    pub fn spawn() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake proxy");
        let addr = listener.local_addr().expect("local addr");
        let hit = Arc::new(AtomicBool::new(false));
        let hit_c = hit.clone();

        let thread = std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                hit_c.store(true, Ordering::SeqCst);
                let mut buf = [0u8; 128];
                let _ = stream.read(&mut buf);
                // Drop closes the connection; the client sees EOF.
            }
        });

        Self {
            addr,
            hit,
            _thread: Some(thread),
        }
    }

    pub fn was_hit(&self) -> bool {
        self.hit.load(Ordering::SeqCst)
    }

    /// A `host:port` string suitable for embedding in a proxy URL.
    pub fn authority(&self) -> String {
        self.addr.to_string()
    }
}

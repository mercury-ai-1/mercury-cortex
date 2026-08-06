//! Tests for the platform IPC endpoint derivation (`ipc::endpoint_for`).
//!
//! On Windows the endpoint is a TCP loopback address derived from the socket
//! path. The derivation must be spelling-insensitive: `PathBuf::push` performs
//! raw byte concatenation and preserves embedded `/` separators, so a daemon
//! built with chained `.join()` and a client built with a forward-slash literal
//! must still agree on the address, or they silently talk to different
//! listeners.

#[cfg(windows)]
use std::path::Path;

#[cfg(windows)]
#[test]
fn same_path_spelled_differently_maps_to_same_endpoint() {
    let backslash =
        Path::new(r"C:\Users\runneradmin\AppData\Local\Temp\abc123\.mercury\cortex\runtime.sock");
    let mixed =
        Path::new("C:/Users/runneradmin/AppData/Local/Temp/abc123/.mercury/cortex/runtime.sock");
    assert_eq!(
        mercury_cortex::ipc::endpoint_for(backslash),
        mercury_cortex::ipc::endpoint_for(mixed)
    );
}

#[cfg(windows)]
#[test]
fn different_paths_map_to_different_endpoints() {
    let a = Path::new(r"C:\Users\a\.mercury\cortex\runtime.sock");
    let b = Path::new(r"C:\Users\b\.mercury\cortex\runtime.sock");
    assert_ne!(
        mercury_cortex::ipc::endpoint_for(a),
        mercury_cortex::ipc::endpoint_for(b)
    );
}

#[cfg(windows)]
#[test]
fn derived_endpoint_is_loopback_tcp() {
    let path = Path::new(r"C:\Users\a\.mercury\cortex\runtime.sock");
    let endpoint = mercury_cortex::ipc::endpoint_for(path);
    let addr: std::net::SocketAddr = endpoint.parse().unwrap();
    assert_eq!(addr.ip(), std::net::Ipv4Addr::LOCALHOST);
    assert!(addr.port() >= 1024);
}

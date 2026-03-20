// Shared library for the video-rtp-latency project.

use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_net as gst_net;

#[path = "hdr-ext.rs"]
pub mod hdr_ext;

/// Try to create a PTP clock. If any step fails (init, creation, or sync),
/// fall back to the system monotonic clock with a warning.
///
/// Reads the `PTP_IFACE` environment variable to restrict PTP to a specific
/// network interface (e.g. `PTP_IFACE=veth1`). When unset, all interfaces
/// are used.
pub fn setup_clock(name: &str) -> gst::Clock {
    let ptp_result: Result<gst::Clock, String> = (|| {
        let ifaces: Vec<String> = std::env::var("PTP_IFACE")
            .ok()
            .map(|s| vec![s])
            .unwrap_or_default();
        let iface_refs: Vec<&str> = ifaces.iter().map(String::as_str).collect();

        gst_net::PtpClock::init(None, &iface_refs).map_err(|e| format!("init: {e}"))?;
        let ptp = gst_net::PtpClock::new(Some(name), 0).map_err(|e| format!("create: {e}"))?;
        // PTP sync needs several announce/sync/delay_req round-trips; 30 s
        // gives plenty of headroom even on slow networks.
        ptp.wait_for_sync(gst::ClockTime::from_seconds(30))
            .map_err(|e| format!("sync: {e}"))?;
        println!("Using PTP clock (domain 0).");
        Ok(ptp.upcast())
    })();

    ptp_result.unwrap_or_else(|err| {
        eprintln!("PTP unavailable ({err}), using system clock.");
        eprintln!("Latency measurement still works for same-host testing.");
        gst::SystemClock::obtain().upcast()
    })
}

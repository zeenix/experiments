// Shared library for the video-rtp-latency project.

use gstreamer as gst;
use gstreamer::prelude::*;

#[path = "hdr-ext.rs"]
pub mod hdr_ext;

/// Create a real-time system clock for the pipeline.
///
/// When both hosts have their system clocks synchronized via PTP (e.g.
/// `ptp4l` + `phc2sys`, or `chrony` with PTP support), the real-time
/// (wall-clock) time is already identical on both machines. We just need
/// a GStreamer clock that reads it.
///
/// We instantiate a *new* `SystemClock` (not the global singleton) and set
/// its `clock-type` to `Realtime` so it reads `CLOCK_REALTIME` instead of
/// the default `CLOCK_MONOTONIC`.
pub fn setup_clock() -> gst::Clock {
    gst::glib::Object::builder::<gst::SystemClock>()
        .property("clock-type", gst::ClockType::Realtime)
        .build()
        .upcast()
}

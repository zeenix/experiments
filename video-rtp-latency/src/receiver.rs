// Receiver: Receives raw video frames over RTP/UDP and measures per-frame
// latency using a custom RTP header extension.
//
// Pipeline: udpsrc ! rtpvrawdepay ! fakesink
//
// The sender's RTP header extension embeds the sender PTS (== sender clock
// time) in every RTP packet. The depayloader's header extension reader
// converts it into a GstReferenceTimestampMeta on the output buffer. A pad
// probe on rtpvrawdepay's src reads this meta and compares with the current
// clock time:
//
//   latency = clock_now - sender_pts
//
// Clock and timing: same setup as sender (start_time=None, base_time=0,
// PTP-or-system clock).

use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

use anyhow::{Context, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_rtp::prelude::*;

use video_rtp_latency::{hdr_ext, setup_clock};

fn main() -> Result<()> {
    gst::init().context("Failed to initialize GStreamer")?;

    let clock = setup_clock("receiver-ptp");

    let pipeline = gst::Pipeline::new();
    pipeline.use_clock(Some(&clock));

    // Mirror the sender's time base so both sides share the same epoch.
    pipeline.set_start_time(gst::ClockTime::NONE);
    pipeline.set_base_time(gst::ClockTime::ZERO);

    // --- Build pipeline elements ---

    // RTP caps must match the sender's rtpvrawpay output for I420 320x240.
    let rtp_caps = gst::Caps::builder("application/x-rtp")
        .field("media", "video")
        .field("clock-rate", 90000i32)
        .field("encoding-name", "RAW")
        .field("sampling", "YCbCr-4:2:0")
        .field("depth", "8")
        .field("width", "320")
        .field("height", "240")
        .build();

    let udpsrc = gst::ElementFactory::make("udpsrc")
        .property("port", 5004i32)
        .property("caps", &rtp_caps)
        .build()
        .context("udpsrc")?;

    let rtpvrawdepay = gst::ElementFactory::make("rtpvrawdepay")
        .build()
        .context("rtpvrawdepay")?;

    let fakesink = gst::ElementFactory::make("fakesink")
        .build()
        .context("fakesink")?;

    // --- Attach custom RTP header extension to the depayloader ---
    //
    // The extension reads the sender's clock time from each RTP packet's
    // header extension and stores it as a GstReferenceTimestampMeta on the
    // reassembled output buffer.
    let hdr_ext = hdr_ext::SenderClockTime::default();
    hdr_ext.set_id(1);
    rtpvrawdepay.emit_by_name::<()>("add-extension", &[&hdr_ext]);

    pipeline.add_many([&udpsrc, &rtpvrawdepay, &fakesink])?;
    gst::Element::link_many([&udpsrc, &rtpvrawdepay, &fakesink])?;

    // --- Pad probe: measure latency from ReferenceTimestampMeta ---

    let clock_for_probe = clock.clone();
    let ref_caps = hdr_ext::reference_caps();
    let frame_count = Arc::new(AtomicU64::new(0));
    let fc = frame_count.clone();

    let depay_src = rtpvrawdepay.static_pad("src").unwrap();
    depay_src.add_probe(gst::PadProbeType::BUFFER, move |_pad, info| {
        let Some(gst::PadProbeData::Buffer(ref buffer)) = info.data else {
            return gst::PadProbeReturn::Ok;
        };
        let Some(clock_now) = clock_for_probe.time() else {
            return gst::PadProbeReturn::Ok;
        };

        // Look up the ReferenceTimestampMeta added by the header extension.
        let mut sender_ts = None;
        buffer.foreach_meta(|meta| {
            if let Some(ref_meta) = meta.downcast_ref::<gst::ReferenceTimestampMeta>() {
                if ref_meta.reference().is_subset(&ref_caps) {
                    sender_ts = Some(ref_meta.timestamp());
                    return std::ops::ControlFlow::Break(());
                }
            }
            std::ops::ControlFlow::Continue(())
        });

        if let Some(capture_time) = sender_ts {
            let latency = clock_now.saturating_sub(capture_time);
            let latency_ms = latency.nseconds() as f64 / 1_000_000.0;

            let n = fc.fetch_add(1, Ordering::Relaxed);
            if n.is_multiple_of(30) {
                println!(
                    "Receiver: frame {n}, latency = {latency_ms:.3} ms  \
                     (sender_pts={capture_time}, clock_now={clock_now})"
                );
            }
        }
        gst::PadProbeReturn::Ok
    });

    // --- Run ---

    pipeline
        .set_state(gst::State::Playing)
        .context("Failed to set pipeline to Playing")?;
    println!("Receiver pipeline running. Press Ctrl-C to stop.");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst));

    let bus = pipeline.bus().unwrap();
    while running.load(Ordering::SeqCst) {
        if let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(100)) {
            use gst::MessageView;
            match msg.view() {
                MessageView::Error(err) => {
                    eprintln!(
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    break;
                }
                MessageView::Eos(..) => {
                    println!("End of stream.");
                    break;
                }
                _ => (),
            }
        }
    }

    pipeline.set_state(gst::State::Null)?;
    println!(
        "Receiver stopped. Frames received: {}",
        frame_count.load(Ordering::Relaxed)
    );
    Ok(())
}

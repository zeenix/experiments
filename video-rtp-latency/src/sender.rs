// Sender: Captures raw video frames and sends them over RTP/UDP with a custom
// RTP header extension that carries the sender's PTS (== clock time) so the
// receiver can measure end-to-end latency.
//
// Pipeline: videotestsrc is-live=true ! capsfilter ! rtpvrawpay ! udpsink
//
// To make PTS equal the pipeline clock time we set:
//   start_time = None   (don't auto-adjust base_time on PAUSED→PLAYING)
//   base_time  = 0      (running_time = clock_time - 0 = clock_time)
// A live source sets PTS ≈ running_time, so PTS ≈ clock_time.
//
// Clock: A real-time SystemClock (CLOCK_REALTIME). When both hosts have
// their system clocks PTP-synchronized (e.g. ptp4l + phc2sys, or chrony
// with PTP), the wall-clock time is identical on both machines.

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

    let clock = setup_clock();

    let pipeline = gst::Pipeline::new();
    pipeline.use_clock(Some(&clock));

    // Make PTS == clock_time by anchoring the time base at zero.
    pipeline.set_start_time(gst::ClockTime::NONE);
    pipeline.set_base_time(gst::ClockTime::ZERO);

    // --- Build pipeline elements ---

    let videotestsrc = gst::ElementFactory::make("videotestsrc")
        .property("is-live", true)
        .build()
        .context("videotestsrc")?;

    // Pin the video format so the receiver can hardcode matching RTP caps.
    let capsfilter = gst::ElementFactory::make("capsfilter")
        .property(
            "caps",
            gst::Caps::builder("video/x-raw")
                .field("format", "I420")
                .field("width", 320i32)
                .field("height", 240i32)
                .field("framerate", gst::Fraction::new(30, 1))
                .build(),
        )
        .build()
        .context("capsfilter")?;

    let rtpvrawpay = gst::ElementFactory::make("rtpvrawpay")
        .build()
        .context("rtpvrawpay")?;

    let udpsink = gst::ElementFactory::make("udpsink")
        .property("host", "127.0.0.1")
        .property("port", 5004i32)
        .property("sync", false)
        .build()
        .context("udpsink")?;

    // --- Attach custom RTP header extension to the payloader ---
    //
    // The extension writes the buffer PTS (== clock time) into each RTP
    // packet's header extension space. The payloader calls write() once per
    // outgoing packet; for raw video each frame is split into many packets,
    // but they all carry the same PTS from the original frame buffer.
    let hdr_ext = hdr_ext::SenderClockTime::default();
    hdr_ext.set_id(1);
    // Use emit_by_name because ElementFactory returns gst::Element, not
    // the typed RTPBasePayload.
    rtpvrawpay.emit_by_name::<()>("add-extension", &[&hdr_ext]);

    pipeline.add_many([&videotestsrc, &capsfilter, &rtpvrawpay, &udpsink])?;
    gst::Element::link_many([&videotestsrc, &capsfilter, &rtpvrawpay, &udpsink])?;

    // --- Optional: log frame count ---

    let frame_count = Arc::new(AtomicU64::new(0));
    let fc = frame_count.clone();
    let src_pad = videotestsrc.static_pad("src").unwrap();
    src_pad.add_probe(gst::PadProbeType::BUFFER, move |_pad, info| {
        let Some(gst::PadProbeData::Buffer(ref buffer)) = info.data else {
            return gst::PadProbeReturn::Ok;
        };
        let n = fc.fetch_add(1, Ordering::Relaxed);
        if n.is_multiple_of(90) {
            if let Some(pts) = buffer.pts() {
                println!("Sender: frame {n}, PTS = {pts}");
            }
        }
        gst::PadProbeReturn::Ok
    });

    // --- Run ---

    pipeline
        .set_state(gst::State::Playing)
        .context("Failed to set pipeline to Playing")?;
    println!("Sender pipeline running. Press Ctrl-C to stop.");

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
        "Sender stopped. Frames sent: {}",
        frame_count.load(Ordering::Relaxed)
    );
    Ok(())
}

// Custom RTP header extension that carries the sender's clock time across RTP.
//
// On the payloader side (write): reads the buffer PTS (which equals the
// pipeline clock time when base_time=0 and start_time=None) and writes it as
// an 8-byte big-endian u64 into the RTP header extension space.
//
// On the depayloader side (read): reads the 8 bytes back and attaches a
// GstReferenceTimestampMeta to the output buffer so downstream can retrieve
// the original sender clock time.
//
// The payloader calls write() once per RTP packet. For raw video, one frame
// is split into many packets, but they all share the same input buffer PTS,
// so every packet carries the same timestamp.

use gstreamer as gst;
use gstreamer::subclass::prelude::*;
use gstreamer_rtp as gst_rtp;
use gstreamer_rtp::subclass::prelude::*;

/// Caps used to tag the ReferenceTimestampMeta so the receiver can look it up.
pub fn reference_caps() -> gst::Caps {
    gst::Caps::builder("timestamp/x-sender-clock").build()
}

pub const URI: &str = "urn:x-gst:rtp-hdrext:sender-clock-time";

mod imp {
    use gst::glib;

    use super::*;

    #[derive(Default)]
    pub struct SenderClockTime;

    #[glib::object_subclass]
    impl ObjectSubclass for SenderClockTime {
        const NAME: &'static str = "SenderClockTimeHdrExt";
        type Type = super::SenderClockTime;
        type ParentType = gst_rtp::RTPHeaderExtension;
    }

    impl ObjectImpl for SenderClockTime {}
    impl GstObjectImpl for SenderClockTime {}
    impl ElementImpl for SenderClockTime {
        fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
            static METADATA: std::sync::LazyLock<gst::subclass::ElementMetadata> =
                std::sync::LazyLock::new(|| {
                    gst::subclass::ElementMetadata::new(
                        "Sender Clock Time RTP Header Extension",
                        "Codec/Payloader/Network/RTP",
                        "Carries sender PTS through RTP header extensions \
                         for end-to-end latency measurement",
                        "zeenix",
                    )
                });
            Some(&METADATA)
        }
    }

    impl RTPHeaderExtensionImpl for SenderClockTime {
        const URI: &'static str = super::URI;

        fn supported_flags(&self) -> gst_rtp::RTPHeaderExtensionFlags {
            // One-byte format supports up to 16 bytes of data per extension
            // element; our 8-byte timestamp fits comfortably.
            gst_rtp::RTPHeaderExtensionFlags::ONE_BYTE | gst_rtp::RTPHeaderExtensionFlags::TWO_BYTE
        }

        fn max_size(&self, _input: &gst::BufferRef) -> usize {
            // u64 nanosecond timestamp = 8 bytes.
            std::mem::size_of::<u64>()
        }

        /// Called by the payloader for each outgoing RTP packet. `input` is
        /// the original media buffer (the full video frame) whose PTS we want
        /// to transmit.
        fn write(
            &self,
            input: &gst::BufferRef,
            _write_flags: gst_rtp::RTPHeaderExtensionFlags,
            _output: &gst::BufferRef,
            output_data: &mut [u8],
        ) -> Result<usize, gst::LoggableError> {
            let Some(pts) = input.pts() else {
                return Ok(0);
            };
            output_data[..8].copy_from_slice(&pts.nseconds().to_be_bytes());
            Ok(8)
        }

        /// Called by the depayloader for each incoming RTP packet. `output` is
        /// the (possibly partially assembled) media buffer that will be pushed
        /// downstream.
        fn read(
            &self,
            _read_flags: gst_rtp::RTPHeaderExtensionFlags,
            input_data: &[u8],
            output: &mut gst::BufferRef,
        ) -> Result<(), gst::LoggableError> {
            if input_data.len() < 8 {
                return Ok(());
            }
            let pts_ns = u64::from_be_bytes(
                input_data[..8]
                    .try_into()
                    .expect("already checked len >= 8"),
            );
            let timestamp = gst::ClockTime::from_nseconds(pts_ns);

            // Attach a ReferenceTimestampMeta so the receiver's pad probe can
            // retrieve the sender's capture time. The depayloader calls read()
            // once per RTP packet; for chunked frames this means multiple adds
            // of the same value, which is harmless.
            gst::ReferenceTimestampMeta::add(
                output,
                &reference_caps(),
                timestamp,
                gst::ClockTime::NONE,
            );
            Ok(())
        }
    }
}

gst::glib::wrapper! {
    pub struct SenderClockTime(ObjectSubclass<imp::SenderClockTime>)
        @extends gst_rtp::RTPHeaderExtension, gst::Element, gst::Object;
}

impl Default for SenderClockTime {
    fn default() -> Self {
        gst::glib::Object::new()
    }
}

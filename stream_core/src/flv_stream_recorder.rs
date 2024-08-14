use utils::chrono::OutOfRange;
use crate::live::{LiveMonitorTrait, LiveTrait, QualityNumber, RecordingMode, StreamFormat};

pub struct FlvStreamRecorder<Live, Monitor> {
    live: Live,
    live_monitor: Monitor,
    out_dir: String,
    path_template: String,
    stream_format: StreamFormat,
    recording_mode: RecordingMode,
    quality_number: QualityNumber,
    stream_timeout: usize,
    buffer_size: usize,
    read_timeout: Option<usize>,
    disconnection_timeout: Option<usize>,
    filesize_limit: usize,
    duration_limit: usize,
    // stream_param_holder
}

impl<Live: LiveTrait, Monitor: LiveMonitorTrait> FlvStreamRecorder<Live, Monitor> {
    pub fn new(
        live: Live,
        live_monitor: Monitor,
        out_dir: String,
        path_template: String,
        stream_format: StreamFormat,
        recording_mode: RecordingMode,
        quality_number: QualityNumber,
        stream_timeout: usize,
        buffer_size: usize,
        read_timeout: Option<usize>,
        disconnection_timeout: Option<usize>,
        filesize_limit: usize,
        duration_limit: usize,
    ) {

    }
}
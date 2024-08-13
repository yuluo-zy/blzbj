use crate::live::{Live, LiveMonitor, QualityNumber, RecordingMode, StreamFormat};
use utils::BResult;
pub struct StreamRecorder {
    live: Live,
    live_monitor: LiveMonitor,
    out_idr: String,
    path_template: String,
    stream_format: StreamFormat,
    recording_mode: RecordingMode,
    quality_number: QualityNumber,
    stream_timeout: usize,
    buffer_size: Option<usize>,
    read_timeout: Option<usize>,
    disconnection_timeout: Option<usize>,
    filesize_limit: usize,
    duration_limit: usize,
}

impl StreamRecorder {
    pub fn new(live: Live, live_monitor: LiveMonitor) -> Self {
        // 根据 StreamFormat 来启动对应的 StreamRecorderImpl
        Self {
            live,
            live_monitor,
            out_idr: "".to_string(),
            path_template: "".to_string(),
            stream_format: StreamFormat::Flv,
            recording_mode: RecordingMode::Standard,
            quality_number: QualityNumber::P20000,
            stream_timeout: 10,
            buffer_size: None,
            read_timeout: None,
            disconnection_timeout: None,
            filesize_limit: 0,
            duration_limit: 0,
        }
    }

    async fn _do_start(self) -> BResult<()> {
        Ok(())
    }
}
use crate::live::{QualityNumber, StreamFormat};

pub struct StreamParamHolder<Live, Monitor > {
    stream_format: StreamFormat,
    quality_number: QualityNumber,
    stream_url: String,
    stream_host: String,
    use_alternative_stream: bool,
    attempts_for_no_stream: u8,
    live: Live,
    live_monitor: Monitor,
}
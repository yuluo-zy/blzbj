use blake3::Hasher;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use crate::amf::ScriptTagBody;
use crate::h264_nalu::H264Nalu;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagType {
    Audio,
    Video,
    Script,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagHeader {
    pub tag_type: TagType,
    pub data_size: u32,
    pub timestamp: u32,
    pub stream_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub version: u8,
    pub audio: bool,
    pub video: bool,
    pub offset: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagData<'a> {
    Audio(AudioData<'a>),
    Video(VideoData<'a>),
    Script,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Tag<'a> {
    pub header: TagHeader,
    pub data: TagData<'a>,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundFormat {
    PCM_NE, // native endianness...
    ADPCM,
    MP3,
    PCM_LE,
    NELLYMOSER_16KHZ_MONO,
    NELLYMOSER_8KHZ_MONO,
    NELLYMOSER,
    PCM_ALAW,
    PCM_ULAW,
    AAC,
    SPEEX,
    MP3_8KHZ,
    DEVICE_SPECIFIC,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundRate {
    _5_5KHZ,
    _11KHZ,
    _22KHZ,
    _44KHZ,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundSize {
    Snd8bit,
    Snd16bit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundType {
    SndMono,
    SndStereo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AACPacketType {
    SequenceHeader,
    Raw,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AACAudioPacketHeader {
    pub packet_type: AACPacketType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AACAudioPacket<'a> {
    pub packet_type: AACPacketType,
    pub aac_data: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioData<'a> {
    pub sound_format: SoundFormat,
    pub sound_rate: SoundRate,
    pub sound_size: SoundSize,
    pub sound_type: SoundType,
    pub sound_data: &'a [u8],
}

#[derive(Debug, PartialEq, Eq)]
pub struct AudioDataHeader {
    pub sound_format: SoundFormat,
    pub sound_rate: SoundRate,
    pub sound_size: SoundSize,
    pub sound_type: SoundType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameType {
    Key,
    Inter,
    DisposableInter,
    Generated,
    Command,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodecId {
    JPEG,
    SORENSON_H263,
    SCREEN,
    VP6,
    VP6A,
    SCREEN2,
    H264,
    // Not in FLV standard
    H263,
    MPEG4Part2, // MPEG-4 Part 2
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AVCPacketType {
    SequenceHeader,
    NALU,
    EndOfSequence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AVCVideoPacketHeader {
    pub packet_type: AVCPacketType,
    pub composition_time: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AVCVideoPacket<'a> {
    pub packet_type: AVCPacketType,
    pub composition_time: i32,
    pub avc_data: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoData<'a> {
    pub frame_type: FrameType,
    pub codec_id: CodecId,
    pub video_data: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoDataHeader {
    pub frame_type: FrameType,
    pub codec_id: CodecId,
}

#[derive(Debug, PartialEq)]
pub struct ScriptData<'a> {
    pub name: &'a str,
    pub arguments: ScriptDataValue<'a>,
}

#[derive(Debug, PartialEq)]
pub enum ScriptDataValue<'a> {
    Number(f64),
    Boolean(bool),
    String(&'a str),
    Object(Vec<ScriptDataObject<'a>>),
    MovieClip(&'a str),
    Null,
    Undefined,
    Reference(u16),
    ECMAArray(Vec<ScriptDataObject<'a>>),
    StrictArray(Vec<ScriptDataValue<'a>>),
    Date(ScriptDataDate),
    LongString(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct ScriptDataObject<'a> {
    pub name: &'a str,
    pub data: ScriptDataValue<'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScriptDataDate {
    pub date_time: f64,
    pub local_date_time_offset: i16, // SI16
}

#[allow(non_upper_case_globals)]
pub static script_data_name_tag: &[u8] = &[2];


use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use num_enum::IntoPrimitive;
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use crate::error::TagReaderError;

const FLV_HEADER: [u8; 9] = [
    0x46, // 'F'
    0x4c, //'L'
    0x56, //'V'
    0x01, //version
    0x05, //00000101  audio tag  and video tag
    0x00, 0x00, 0x00, 0x09, //flv header size
];

pub const HEADER_LENGTH: u32 = 11;

#[async_trait]
pub trait Unmarshal<'a, T1, T2> {
    async fn unmarshal(reader: &'a T1) -> T2
        where
            Self: Sized;
}

#[async_trait]
pub trait Marshal<T> {
    async fn marshal(&self) -> T;
}

#[derive(Debug, Clone, Serialize, Default)]
pub enum SoundFormat {
    #[default]
    AAC = 10,
    OPUS = 13,
}
/// audio
pub mod aac_packet_type {
    pub const AAC_SEQHDR: u8 = 0;
    pub const AAC_RAW: u8 = 1;
}

/// video
pub mod avc_packet_type {
    pub const AVC_SEQHDR: u8 = 0;
    pub const AVC_NALU: u8 = 1;
    pub const AVC_EOS: u8 = 2;
}

pub mod frame_type {
    /*
        1: keyframe (for AVC, a seekable frame)
        2: inter frame (for AVC, a non- seekable frame)
        3: disposable inter frame (H.263 only)
        4: generated keyframe (reserved for server use only)
        5: video info/command frame
    */
    pub const KEY_FRAME: u8 = 1;
    pub const INTER_FRAME: u8 = 2;
}

#[derive(Debug, Clone, Serialize, Default, IntoPrimitive)]
#[repr(u8)]
pub enum AvcCodecId {
    #[default]
    UNKNOWN = 0,
    H264 = 7,
    HEVC = 12,
}

pub mod tag_type {
    pub const AUDIO: u8 = 8;
    pub const VIDEO: u8 = 9;
    pub const SCRIPT_DATA_AMF: u8 = 18;
}

pub mod h264_nal_type {
    pub const H264_NAL_IDR: u8 = 5;
    pub const H264_NAL_SPS: u8 = 7;
    pub const H264_NAL_PPS: u8 = 8;
    pub const H264_NAL_AUD: u8 = 9;
}

#[derive(Debug, Clone, Serialize, Default, IntoPrimitive)]
#[repr(u8)]
pub enum AacProfile {
    // @see @see ISO_IEC_14496-3-AAC-2001.pdf, page 23
    #[default]
    UNKNOWN = 0,
    LC = 2,
    SSR = 3,
    // AAC HE = LC+SBR
    HE = 5,
    // AAC HEv2 = LC+SBR+PS
    HEV2 = 29,
}

#[derive(Debug, Clone, Serialize, Default, IntoPrimitive)]
#[repr(u8)]
pub enum AvcProfile {
    #[default]
    UNKNOWN = 0,
    // @see ffmpeg, libavcodec/avcodec.h:2713
    Baseline = 66,
    Main = 77,
    Extended = 88,
    High = 100,
}

#[derive(Debug, Clone, Serialize, Default, IntoPrimitive)]
#[repr(u8)]
pub enum AvcLevel {
    #[default]
    UNKNOWN = 0,
    #[serde(rename = "1.0")]
    Level1 = 10,
    #[serde(rename = "1.1")]
    Level11 = 11,
    #[serde(rename = "1.2")]
    Level12 = 12,
    #[serde(rename = "1.3")]
    Level13 = 13,
    #[serde(rename = "2.0")]
    Level2 = 20,
    #[serde(rename = "2.1")]
    Level21 = 21,
    #[serde(rename = "2.2")]
    Level22 = 22,
    #[serde(rename = "3.0")]
    Level3 = 30,
    #[serde(rename = "3.1")]
    Level31 = 31,
    #[serde(rename = "3.2")]
    Level32 = 32,
    #[serde(rename = "4.0")]
    Level4 = 40,
    #[serde(rename = "4.1")]
    Level41 = 41,
    #[serde(rename = "5.0")]
    Level5 = 50,
    #[serde(rename = "5.1")]
    Level51 = 51,
}

pub enum FlvData {
    Video { timestamp: u32, data: BytesMut },
    Audio { timestamp: u32, data: BytesMut },
    MetaData { timestamp: u32, data: BytesMut },
}

#[derive(Clone, Debug)]
pub struct AudioTagHeader {
    //1010 11 1 1
    /*
        SoundFormat: UB[4]
        0 = Linear PCM, platform endian
        1 = ADPCM
        2 = MP3
        3 = Linear PCM, little endian
        4 = Nellymoser 16-kHz mono
        5 = Nellymoser 8-kHz mono
        6 = Nellymoser
        7 = G.711 A-law logarithmic PCM
        8 = G.711 mu-law logarithmic PCM
        9 = reserved
        10 = AAC
        11 = Speex
        14 = MP3 8-Khz
        15 = Device-specific sound
        Formats 7, 8, 14, and 15 are reserved for internal use
        AAC is supported in Flash Player 9,0,115,0 and higher.
        Speex is supported in Flash Player 10 and higher.
    */
    pub sound_format: u8,
    /*
        SoundRate: UB[2]
        Sampling rate
        0 = 5.5-kHz For AAC: always 3
        1 = 11-kHz
        2 = 22-kHz
        3 = 44-kHz
    */
    pub sound_rate: u8,
    /*
        SoundSize: UB[1]
        0 = snd8Bit
        1 = snd16Bit
        Size of each sample.
        This parameter only pertains to uncompressed formats.
        Compressed formats always decode to 16 bits internally
    */
    pub sound_size: u8,
    /*
        SoundType: UB[1]
        0 = sndMono
        1 = sndStereo
        Mono or stereo sound For Nellymoser: always 0
        For AAC: always 1
    */
    pub sound_type: u8,

    /*
        0: AAC sequence header
        1: AAC raw
    */
    pub aac_packet_type: u8,
}

impl Default for AudioTagHeader {
    fn default() -> Self {
        Self {
            sound_format: 0,
            sound_rate: 0,
            sound_size: 0,
            sound_type: 0,
            aac_packet_type: 0,
        }
    }
}

#[derive(Clone)]
pub struct VideoTagHeader {
    /*
        1: keyframe (for AVC, a seekable frame)
        2: inter frame (for AVC, a non- seekable frame)
        3: disposable inter frame (H.263 only)
        4: generated keyframe (reserved for server use only)
        5: video info/command frame
    */
    pub frame_type: u8,
    /*
        1: JPEG (currently unused)
        2: Sorenson H.263
        3: Screen video
        4: On2 VP6
        5: On2 VP6 with alpha channel
        6: Screen video version 2
        7: AVC
        12: HEVC
    */
    pub codec_id: u8,
    /*
        0: AVC sequence header
        1: AVC NALU
        2: AVC end of sequence (lower level NALU sequence ender is not required or supported)
    */
    pub avc_packet_type: u8,
    pub composition_time: i32,
}

impl Default for VideoTagHeader {
    fn default() -> Self {
        Self {
            frame_type: 0,
            codec_id: 0,
            avc_packet_type: 0,
            composition_time: 0,
        }
    }
}
impl<'a, R> Unmarshal<'a, R, Result<Self, TagReaderError>> for AudioTagHeader where R:AsyncRead + AsyncReadExt + Unpin {
    async fn unmarshal(reader: &'a mut R) -> Result<Self, TagReaderError>
        where
            Self: Sized,
    {
        let mut tag_header = AudioTagHeader::default();

        let flags = reader.read_u8().await?;
        tag_header.sound_format = flags >> 4;
        tag_header.sound_rate = (flags >> 2) & 0x03;
        tag_header.sound_size = (flags >> 1) & 0x01;
        tag_header.sound_type = flags & 0x01;

        if tag_header.sound_format == SoundFormat::AAC.into() {
            tag_header.aac_packet_type = reader.read_u8().await?;
        }

        Ok(tag_header)
    }
}

impl Marshal<Result<Bytes, TagReaderError>> for AudioTagHeader {
    async fn marshal(&self) -> Result<Bytes, TagReaderError> {
        let mut writer = BytesMut::default();

        let byte_1st =
            self.sound_format << 4 | self.sound_rate << 2 | self.sound_size << 1 | self.sound_type;
        writer.put_u8(byte_1st)?;

        if self.sound_format == SoundFormat::AAC as u8 {
            writer.put_u8(self.aac_packet_type)?;
        }

        Ok(writer.freeze())
    }
}
impl<'a, R> Unmarshal<'a, R, Result<Self, TagReaderError>> for VideoTagHeader where R:AsyncRead + AsyncReadExt + Unpin {
    async fn unmarshal(reader: &'a mut R) -> Result<Self, TagReaderError>
        where
            Self: Sized,
    {
        let mut tag_header = VideoTagHeader::default();

        let flags = reader.read_u8().await?;
        tag_header.frame_type = flags >> 4;
        tag_header.codec_id = flags & 0x0f;

        if tag_header.codec_id == AvcCodecId::H264.into()
            || tag_header.codec_id == AvcCodecId::HEVC.into()
        {
            tag_header.avc_packet_type = reader.read_u8().await?;
            tag_header.composition_time = 0;

            //bigend 3bytes
            for _ in 0..3 {
                let time = reader.read_u8().await?;
                //print!("==time0=={}\n", time);
                //print!("==time1=={}\n", self.tag.composition_time);
                tag_header.composition_time = (tag_header.composition_time << 8) + time as i32;
            }
            //transfer to signed i24
            if tag_header.composition_time & (1 << 23) != 0 {
                let sign_extend_mask = 0xff_ff << 23;
                // Sign extend the value
                tag_header.composition_time |= sign_extend_mask
            }
        }

        Ok(tag_header)
    }
}

impl Marshal<Result<Bytes, TagReaderError>> for VideoTagHeader {
    async fn marshal(&self) -> Result<Bytes, TagReaderError> {
        let mut writer = BytesMut::default();

        let byte_1st = self.frame_type << 4 | self.codec_id;
        writer.put_u8(byte_1st)?;

        if self.codec_id == AvcCodecId::H264.into()
            || self.codec_id == AvcCodecId::HEVC.into()
        {
            writer.put_u8(self.avc_packet_type)?;

            let mut cts = self.composition_time;
            for _ in 0..3 {
                writer.put_u8((cts & 0xFF) as u8)?;
                cts >>= 8;
            }
        }

        Ok(writer.freeze())
    }
}
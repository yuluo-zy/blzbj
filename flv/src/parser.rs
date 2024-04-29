// source: https://github.com/rust-av/flavors/blob/master/src/parser.rs
use nom::bits::bits;
use nom::bits::streaming::take;
use nom::bytes::streaming::tag;
use nom::combinator::{flat_map, map, map_res};
use nom::error::{Error, ErrorKind};
use nom::multi::{length_data, many0, many_m_n};
use nom::number::streaming::{be_f64, be_i16, be_i24, be_u16, be_u24, be_u32, be_u8};
use nom::sequence::{pair, terminated, tuple};
use nom::{Err, IResult, Needed};
use serde::Serialize;
use std::str::from_utf8;
use std::time::Duration;
use bytes::{Bytes, BytesMut};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Header {
    pub version: u8,
    pub audio: bool,
    pub video: bool,
    pub offset: u32,
}

pub fn header(input: &[u8]) -> IResult<&[u8], Header> {
    map(
        tuple((tag("FLV"), be_u8, be_u8, be_u32)),
        |(_, version, flags, offset)| Header {
            version,
            audio: flags & 4 == 4,
            video: flags & 1 == 1,
            offset,
        },
    )(input)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum TagType {
    Audio = 8,
    Video = 9,
    Script = 18,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TagHeader {
    pub tag_type: TagType,
    pub data_size: u32,
    pub timestamp: u32,
    pub stream_id: u32,
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

fn tag_type(input: &[u8]) -> IResult<&[u8], TagType> {
    map_res(be_u8, |tag_type| {
        Ok(match tag_type {
            8 => TagType::Audio,
            9 => TagType::Video,
            18 => TagType::Script,
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        })
    })(input)
}

pub fn tag_header(input: &[u8]) -> IResult<&[u8], TagHeader> {
    map(
        tuple((tag_type, be_u24, be_u24, be_u8, be_u24)),
        |(tag_type, data_size, timestamp, timestamp_extended, stream_id)| TagHeader {
            tag_type,
            data_size,
            timestamp: (u32::from(timestamp_extended) << 24) + timestamp,
            stream_id,
        },
    )(input)
}

pub fn complete_tag(input: &[u8]) -> IResult<&[u8], Tag> {
    flat_map(pair(tag_type, be_u24), |(tag_type, data_size)| {
        map(
            tuple((
                be_u24,
                be_u8,
                be_u24,
                tag_data(tag_type, data_size as usize),
            )),
            move |(timestamp, timestamp_extended, stream_id, data)| Tag {
                header: TagHeader {
                    tag_type,
                    data_size,
                    timestamp: (u32::from(timestamp_extended) << 24) + timestamp,
                    stream_id,
                },
                data,
            },
        )
    })(input)
}

pub fn tag_data(tag_type: TagType, size: usize) -> impl Fn(&[u8]) -> IResult<&[u8], TagData> {
    move |input| match tag_type {
        TagType::Video => map(|i| video_data(i, size), TagData::Video)(input),
        TagType::Audio => map(|i| audio_data(i, size), TagData::Audio)(input),
        TagType::Script => Ok((input, TagData::Script)),
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SoundRate {
    _5_5KHZ,
    _11KHZ,
    _22KHZ,
    _44KHZ,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SoundSize {
    Snd8bit,
    Snd16bit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SoundType {
    SndMono,
    SndStereo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum AACPacketType {
    SequenceHeader,
    Raw,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AACAudioPacketHeader {
    pub packet_type: AACPacketType,
}

pub fn aac_audio_packet_header(input: &[u8]) -> IResult<&[u8], AACAudioPacketHeader> {
    map_res(be_u8, |packet_type| {
        Ok(AACAudioPacketHeader {
            packet_type: match packet_type {
                0 => AACPacketType::SequenceHeader,
                1 => AACPacketType::Raw,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            },
        })
    })(input)
}

#[derive(Debug, PartialEq, Eq)]
pub struct AACAudioPacket<'a> {
    pub packet_type: AACPacketType,
    pub aac_data: &'a [u8],
}

pub fn aac_audio_packet(input: &[u8], size: usize) -> IResult<&[u8], AACAudioPacket> {
    if input.len() < size {
        return Err(Err::Incomplete(Needed::new(size)));
    }

    if size < 1 {
        return Err(Err::Incomplete(Needed::new(1)));
    }

    be_u8(input).and_then(|(_, packet_type)| {
        Ok((
            &input[size..],
            AACAudioPacket {
                packet_type: match packet_type {
                    0 => AACPacketType::SequenceHeader,
                    1 => AACPacketType::Raw,
                    _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
                },
                aac_data: &input[1..size],
            },
        ))
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioData<'a> {
    pub sound_format: SoundFormat,
    pub sound_rate: SoundRate,
    pub sound_size: SoundSize,
    pub sound_type: SoundType,
    pub sound_data: &'a [u8],
}

pub fn audio_data(input: &[u8], size: usize) -> IResult<&[u8], AudioData> {
    if input.len() < size {
        return Err(Err::Incomplete(Needed::new(size)));
    }

    if size < 1 {
        return Err(Err::Incomplete(Needed::new(1)));
    }

    let take_bits = tuple((take(4usize), take(2usize), take(1usize), take(1usize)));
    bits::<_, _, Error<_>, _, _>(take_bits)(input).and_then(
        |(_, (sformat, srate, ssize, stype))| {
            let sformat = match sformat {
                0 => SoundFormat::PCM_NE,
                1 => SoundFormat::ADPCM,
                2 => SoundFormat::MP3,
                3 => SoundFormat::PCM_LE,
                4 => SoundFormat::NELLYMOSER_16KHZ_MONO,
                5 => SoundFormat::NELLYMOSER_8KHZ_MONO,
                6 => SoundFormat::NELLYMOSER,
                7 => SoundFormat::PCM_ALAW,
                8 => SoundFormat::PCM_ULAW,
                10 => SoundFormat::AAC,
                11 => SoundFormat::SPEEX,
                14 => SoundFormat::MP3_8KHZ,
                15 => SoundFormat::DEVICE_SPECIFIC,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            };
            let srate = match srate {
                0 => SoundRate::_5_5KHZ,
                1 => SoundRate::_11KHZ,
                2 => SoundRate::_22KHZ,
                3 => SoundRate::_44KHZ,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            };
            let ssize = match ssize {
                0 => SoundSize::Snd8bit,
                1 => SoundSize::Snd16bit,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            };
            let stype = match stype {
                0 => SoundType::SndMono,
                1 => SoundType::SndStereo,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            };

            Ok((
                &input[size..],
                AudioData {
                    sound_format: sformat,
                    sound_rate: srate,
                    sound_size: ssize,
                    sound_type: stype,
                    sound_data: &input[1..size],
                },
            ))
        },
    )
}

#[derive(Debug, PartialEq, Eq)]
pub struct AudioDataHeader {
    pub sound_format: SoundFormat,
    pub sound_rate: SoundRate,
    pub sound_size: SoundSize,
    pub sound_type: SoundType,
}

pub fn audio_data_header(input: &[u8]) -> IResult<&[u8], AudioDataHeader> {
    if input.is_empty() {
        return Err(Err::Incomplete(Needed::new(1)));
    }

    let take_bits = tuple((take(4usize), take(2usize), take(1usize), take(1usize)));
    map_res(
        bits::<_, _, Error<_>, _, _>(take_bits),
        |(sformat, srate, ssize, stype)| {
            let sformat = match sformat {
                0 => SoundFormat::PCM_NE,
                1 => SoundFormat::ADPCM,
                2 => SoundFormat::MP3,
                3 => SoundFormat::PCM_LE,
                4 => SoundFormat::NELLYMOSER_16KHZ_MONO,
                5 => SoundFormat::NELLYMOSER_8KHZ_MONO,
                6 => SoundFormat::NELLYMOSER,
                7 => SoundFormat::PCM_ALAW,
                8 => SoundFormat::PCM_ULAW,
                10 => SoundFormat::AAC,
                11 => SoundFormat::SPEEX,
                14 => SoundFormat::MP3_8KHZ,
                15 => SoundFormat::DEVICE_SPECIFIC,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            };
            let srate = match srate {
                0 => SoundRate::_5_5KHZ,
                1 => SoundRate::_11KHZ,
                2 => SoundRate::_22KHZ,
                3 => SoundRate::_44KHZ,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            };
            let ssize = match ssize {
                0 => SoundSize::Snd8bit,
                1 => SoundSize::Snd16bit,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            };
            let stype = match stype {
                0 => SoundType::SndMono,
                1 => SoundType::SndStereo,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            };

            Ok(AudioDataHeader {
                sound_format: sformat,
                sound_rate: srate,
                sound_size: ssize,
                sound_type: stype,
            })
        },
    )(input)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum FrameType {
    Key,
    Inter,
    DisposableInter,
    Generated,
    Command,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
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

fn packet_type(input: &[u8]) -> IResult<&[u8], AVCPacketType> {
    map_res(be_u8, |packet_type| {
        Ok(match packet_type {
            0 => AVCPacketType::SequenceHeader,
            1 => AVCPacketType::NALU,
            2 => AVCPacketType::EndOfSequence,
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        })
    })(input)
}

pub fn avc_video_packet_header(input: &[u8]) -> IResult<&[u8], AVCVideoPacketHeader> {
    map(
        pair(packet_type, be_i24),
        |(packet_type, composition_time)| AVCVideoPacketHeader {
            packet_type,
            composition_time,
        },
    )(input)
}

#[derive(Debug, PartialEq, Eq)]
pub struct AVCVideoPacket<'a> {
    pub packet_type: AVCPacketType,
    pub composition_time: i32,
    pub avc_data: &'a [u8],
}

pub fn avc_video_packet(input: &[u8], size: usize) -> IResult<&[u8], AVCVideoPacket> {
    if input.len() < size {
        return Err(Err::Incomplete(Needed::new(size)));
    }

    if size < 4 {
        return Err(Err::Incomplete(Needed::new(4)));
    }
    pair(packet_type, be_i24)(input).map(|(_, (packet_type, composition_time))| {
        (
            &input[size..],
            AVCVideoPacket {
                packet_type,
                composition_time,
                avc_data: &input[4..size],
            },
        )
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoData<'a> {
    pub frame_type: FrameType,
    pub codec_id: CodecId,
    pub video_data: &'a [u8],
}

pub fn video_data(input: &[u8], size: usize) -> IResult<&[u8], VideoData> {
    if input.len() < size {
        return Err(Err::Incomplete(Needed::new(size)));
    }

    if size < 1 {
        return Err(Err::Incomplete(Needed::new(1)));
    }

    let take_bits = pair(take(4usize), take(4usize));
    bits::<_, _, Error<_>, _, _>(take_bits)(input).and_then(|(_, (frame_type, codec_id))| {
        let frame_type = match frame_type {
            1 => FrameType::Key,
            2 => FrameType::Inter,
            3 => FrameType::DisposableInter,
            4 => FrameType::Generated,
            5 => FrameType::Command,
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        };
        let codec_id = match codec_id {
            1 => CodecId::JPEG,
            2 => CodecId::SORENSON_H263,
            3 => CodecId::SCREEN,
            4 => CodecId::VP6,
            5 => CodecId::VP6A,
            6 => CodecId::SCREEN2,
            7 => CodecId::H264,
            8 => CodecId::H263,
            9 => CodecId::MPEG4Part2,
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        };

        Ok((
            &input[size..],
            VideoData {
                frame_type,
                codec_id,
                video_data: &input[1..size],
            },
        ))
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoDataHeader {
    pub frame_type: FrameType,
    pub codec_id: CodecId,
}

pub fn video_data_header(input: &[u8]) -> IResult<&[u8], VideoDataHeader> {
    if input.is_empty() {
        return Err(Err::Incomplete(Needed::new(1)));
    }

    let take_bits = pair(take(4usize), take(4usize));
    map_res(
        bits::<_, _, Error<_>, _, _>(take_bits),
        |(frame_type, codec_id)| {
            let frame_type = match frame_type {
                1 => FrameType::Key,
                2 => FrameType::Inter,
                3 => FrameType::DisposableInter,
                4 => FrameType::Generated,
                5 => FrameType::Command,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            };
            let codec_id = match codec_id {
                1 => CodecId::JPEG,
                2 => CodecId::SORENSON_H263,
                3 => CodecId::SCREEN,
                4 => CodecId::VP6,
                5 => CodecId::VP6A,
                6 => CodecId::SCREEN2,
                7 => CodecId::H264,
                8 => CodecId::H263,
                9 => CodecId::MPEG4Part2,
                _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            };

            Ok(VideoDataHeader {
                frame_type,
                codec_id,
            })
        },
    )(input)
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ScriptData<'a> {
    pub name: &'a str,
    pub arguments: ScriptDataValue<'a>,
}

#[derive(Debug, PartialEq, Serialize)]
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

#[derive(Debug, PartialEq, Serialize)]
pub struct ScriptDataObject<'a> {
    pub name: &'a str,
    pub data: ScriptDataValue<'a>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ScriptDataDate {
    pub date_time: f64,
    pub local_date_time_offset: i16, // SI16
}

#[allow(non_upper_case_globals)]
static script_data_name_tag: &[u8] = &[2];

pub fn script_data(input: &[u8]) -> IResult<&[u8], ScriptData> {
    // Must start with a string, i.e. 2
    map(
        tuple((
            tag(script_data_name_tag),
            script_data_string,
            script_data_value,
        )),
        |(_, name, arguments)| ScriptData { name, arguments },
    )(input)
}

pub fn script_data_value(input: &[u8]) -> IResult<&[u8], ScriptDataValue> {
    be_u8(input).and_then(|v| match v {
        (i, 0) => map(be_f64, ScriptDataValue::Number)(i),
        (i, 1) => map(be_u8, |n| ScriptDataValue::Boolean(n != 0))(i),
        (i, 2) => map(script_data_string, ScriptDataValue::String)(i),
        (i, 3) => map(script_data_objects, ScriptDataValue::Object)(i),
        (i, 4) => map(script_data_string, ScriptDataValue::MovieClip)(i),
        (i, 5) => Ok((i, ScriptDataValue::Null)), // to remove
        (i, 6) => Ok((i, ScriptDataValue::Undefined)), // to remove
        (i, 7) => map(be_u16, ScriptDataValue::Reference)(i),
        (i, 8) => map(script_data_ecma_array, ScriptDataValue::ECMAArray)(i),
        (i, 10) => map(script_data_strict_array, ScriptDataValue::StrictArray)(i),
        (i, 11) => map(script_data_date, ScriptDataValue::Date)(i),
        (i, 12) => map(script_data_long_string, ScriptDataValue::LongString)(i),
        _ => Err(Err::Error(Error::new(input, ErrorKind::Alt))),
    })
}

pub fn script_data_objects(input: &[u8]) -> IResult<&[u8], Vec<ScriptDataObject>> {
    terminated(many0(script_data_object), script_data_object_end)(input)
}

pub fn script_data_object(input: &[u8]) -> IResult<&[u8], ScriptDataObject> {
    map(
        pair(script_data_string, script_data_value),
        |(name, data)| ScriptDataObject { name, data },
    )(input)
}

#[allow(non_upper_case_globals)]
static script_data_object_end_terminator: &[u8] = &[0, 0, 9];

pub fn script_data_object_end(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(script_data_object_end_terminator)(input)
}

pub fn script_data_string(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(length_data(be_u16), from_utf8)(input)
}

pub fn script_data_long_string(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(length_data(be_u32), from_utf8)(input)
}

pub fn script_data_date(input: &[u8]) -> IResult<&[u8], ScriptDataDate> {
    map(
        pair(be_f64, be_i16),
        |(date_time, local_date_time_offset)| ScriptDataDate {
            date_time,
            local_date_time_offset,
        },
    )(input)
}

pub fn script_data_ecma_array(input: &[u8]) -> IResult<&[u8], Vec<ScriptDataObject>> {
    map(pair(be_u32, script_data_objects), |(_, data_objects)| {
        data_objects
    })(input)
}

pub fn script_data_strict_array(input: &[u8]) -> IResult<&[u8], Vec<ScriptDataValue>> {
    flat_map(be_u32, |o| many_m_n(1, o as usize, script_data_value))(input)
}


// pub(crate) async fn parse_flv(
//     mut connection: Connection,
//     file: LifecycleFile,
//     mut segment: Segmentable,
// ) -> crate::downloader::error::Result<()>
// {
//     let mut flv_tags_cache: Vec<(TagHeader, Bytes, Bytes)> = Vec::new();
//
//     let _previous_tag_size = connection.read_frame(4).await?;
//
//     let mut out = FlvFile::new(file)?;
//     segment.set_size_position(9 + 4);
//     // let mut downloaded_size = 9 + 4;
//     let mut on_meta_data = None;
//     let mut aac_sequence_header = None;
//     let mut h264_sequence_header: Option<(TagHeader, Bytes, Bytes)> = None;
//     let mut prev_timestamp = 0;
//     let mut create_new = false;
//     loop {
//         let tag_header_bytes = connection.read_frame(11).await?;
//         if tag_header_bytes.is_empty() {
//             // let mut rdr = Cursor::new(tag_header_bytes);
//             // println!("{}", rdr.read_u32::<BigEndian>().unwrap());
//             break;
//         }
//
//         let (_, tag_header) = map_parse_err(tag_header(&tag_header_bytes), "tag header")?;
//         // write_tag_header(&mut out, &tag_header)?;
//
//         let bytes = connection.read_frame(tag_header.data_size as usize).await?;
//         let previous_tag_size = connection.read_frame(4).await?;
//         // out.write(&bytes)?;
//         let (i, flv_tag_data) = map_parse_err(
//             tag_data(tag_header.tag_type, tag_header.data_size as usize)(&bytes),
//             "tag data",
//         )?;
//         let flv_tag = match flv_tag_data {
//             TagData::Audio(audio_data) => {
//                 let packet_type = if audio_data.sound_format == SoundFormat::AAC {
//                     let (_, packet_header) = aac_audio_packet_header(audio_data.sound_data)
//                         .expect("Error in parsing aac audio packet header.");
//                     if packet_header.packet_type == AACPacketType::SequenceHeader {
//                         if aac_sequence_header.is_some() {
//                             warn!("Unexpected aac sequence header tag. {tag_header:?}");
//                             // panic!("Unexpected aac_sequence_header tag.");
//                             // create_new = true;
//                         }
//                         aac_sequence_header =
//                             Some((tag_header, bytes.clone(), previous_tag_size.clone()))
//                     }
//                     Some(packet_header.packet_type)
//                 } else {
//                     None
//                 };
//
//                 FlvTag {
//                     header: tag_header,
//                     data: TagDataHeader::Audio {
//                         sound_format: audio_data.sound_format,
//                         sound_rate: audio_data.sound_rate,
//                         sound_size: audio_data.sound_size,
//                         sound_type: audio_data.sound_type,
//                         packet_type,
//                     },
//                 }
//             }
//             TagData::Video(video_data) => {
//                 let (packet_type, composition_time) = if CodecId::H264 == video_data.codec_id {
//                     let (_, avc_video_header) = avc_video_packet_header(video_data.video_data)
//                         .expect("Error in parsing avc video packet header.");
//                     if avc_video_header.packet_type == AVCPacketType::SequenceHeader {
//                         if let Some((_, binary_data, _)) = &h264_sequence_header {
//                             warn!("Unexpected h264 sequence header tag. {tag_header:?}");
//                             if bytes != binary_data {
//                                 create_new = true;
//                                 warn!("Different h264 sequence header tag. {tag_header:?}");
//                             }
//                         }
//                         h264_sequence_header =
//                             Some((tag_header, bytes.clone(), previous_tag_size.clone()))
//                     }
//                     (
//                         Some(avc_video_header.packet_type),
//                         Some(avc_video_header.composition_time),
//                     )
//                 } else {
//                     (None, None)
//                 };
//
//                 FlvTag {
//                     header: tag_header,
//                     data: TagDataHeader::Video {
//                         frame_type: video_data.frame_type,
//                         codec_id: video_data.codec_id,
//                         packet_type,
//                         composition_time,
//                     },
//                 }
//             }
//             TagData::Script => {
//                 let (_, tag_data) = script_data(i).expect("Error in parsing script tag.");
//                 if on_meta_data.is_some() {
//                     warn!("Unexpected script tag. {tag_header:?}");
//                 }
//                 on_meta_data = Some((tag_header, bytes.clone(), previous_tag_size.clone()));
//
//                 let flv_tag = FlvTag {
//                     header: tag_header,
//                     data: TagDataHeader::Script(tag_data),
//                 };
//                 flv_tag
//             }
//         };
//         match &flv_tag {
//             FlvTag {
//                 data:
//                 TagDataHeader::Video {
//                     frame_type: FrameType::Key,
//                     ..
//                 },
//                 ..
//             } => {
//                 let timestamp = flv_tag.header.timestamp as u64;
//                 segment.set_time_position(Duration::from_millis(timestamp));
//                 for (tag_header, flv_tag_data, previous_tag_size_bytes) in &flv_tags_cache {
//                     if tag_header.timestamp < prev_timestamp {
//                         warn!("Non-monotonous DTS in output stream; previous: {prev_timestamp}, current: {};", tag_header.timestamp);
//                     }
//                     out.write_tag(tag_header, flv_tag_data, previous_tag_size_bytes)?;
//                     segment.increase_size((11 + tag_header.data_size + 4) as u64);
//                     // downloaded_size += (11 + tag_header.data_size + 4) as u64;
//                     prev_timestamp = tag_header.timestamp
//                     // println!("{downloaded_size}");
//                 }
//                 flv_tags_cache.clear();
//
//                 if segment.needed() || create_new {
//                     segment.set_start_time(Duration::from_millis(timestamp));
//                     segment.set_size_position(9 + 4);
//
//                     let (meta_header, meta_bytes, previous_meta_tag_size) =
//                         on_meta_data.as_ref().expect("on_meta_data does not exist");
//                     // onMetaData
//                     flv_tags_cache.push((
//                         *meta_header,
//                         meta_bytes.clone(),
//                         previous_meta_tag_size.clone(),
//                     ));
//                     // AACSequenceHeader
//                     let aac_sequence_header = aac_sequence_header
//                         .as_ref()
//                         .expect("aac_sequence_header does not exist");
//                     flv_tags_cache.push((
//                         aac_sequence_header.0,
//                         aac_sequence_header.1.clone(),
//                         aac_sequence_header.2.clone(),
//                     ));
//                     if !create_new {
//                         // H264SequenceHeader
//                         flv_tags_cache.push(
//                             h264_sequence_header
//                                 .as_ref()
//                                 .expect("h264_sequence_header does not exist")
//                                 .clone(),
//                         );
//                     }
//                     info!("{} splitting.{segment:?}", out.file.file_name);
//                     out.create_new()?;
//                     create_new = false;
//                 }
//                 flv_tags_cache.push((tag_header, bytes.clone(), previous_tag_size.clone()));
//             }
//             _ => {
//                 flv_tags_cache.push((tag_header, bytes.clone(), previous_tag_size.clone()));
//             }
//         }
//     }
//     Ok(())
// }
//
// pub fn map_parse_err<'a, T>(
//     i_result: IResult<&'a [u8], T>,
//     msg: &str,
// ) -> core::result::Result<(&'a [u8], T), crate::downloader::error::Error> {
//     match i_result {
//         Ok((i, res)) => Ok((i, res)),
//         Err(nom::Err::Incomplete(needed)) => Err(crate::downloader::error::Error::NomIncomplete(
//             msg.to_string(),
//             needed,
//         )),
//         Err(Err::Error(e)) => {
//             panic!("parse {msg} err: {e:?}")
//         }
//         Err(Err::Failure(f)) => {
//             panic!("{msg} Failure: {f:?}")
//         }
//     }
// }
//
// pub struct Connection {
//     resp: Response,
//     buffer: BytesMut,
// }
//
// impl Connection {
//     pub fn new(resp: Response) -> Connection {
//         Connection {
//             resp,
//             buffer: BytesMut::with_capacity(8 * 1024),
//         }
//     }
//
//     pub async fn read_frame(&mut self, chunk_size: usize) -> crate::downloader::error::Result<Bytes> {
//         // let mut buf = [0u8; 8 * 1024];
//         loop {
//             if chunk_size <= self.buffer.len() {
//                 let bytes = Bytes::copy_from_slice(&self.buffer[..chunk_size]);
//                 self.buffer.advance(chunk_size);
//                 return Ok(bytes);
//             }
//             // BytesMut::with_capacity(0).deref_mut()
//             // tokio::fs::File::open("").read()
//             // self.resp.chunk()
//             if let Ok(Some(chunk)) = timeout(Duration::from_secs(30), self.resp.chunk()).await? {
//                 // let n = chunk.len();
//                 // println!("Chunk: {:?}", chunk);
//                 self.buffer.put(chunk);
//                 // self.buffer.put_slice(&buf[..n]);
//             } else {
//                 return Ok(self.buffer.split().freeze());
//             }
//             // let n = match self.resp.read(&mut buf).await {
//             //     Ok(n) => n,
//             //     Err(e) if e.kind() == ErrorKind::Interrupted => continue,
//             //     Err(e) => return Err(e),
//             // };
//
//             // if n == 0 {
//             //     return Ok(self.buffer.split().freeze());
//             // }
//             // self.buffer.put_slice(&buf[..n]);
//         }
//     }
// }
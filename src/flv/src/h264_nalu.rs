use tokio::io::{AsyncReadExt, AsyncSeekExt};
use anyhow::Result;
///`H264Nalu` 指的是网络抽象层单元（Network Abstraction Layer Unit）在 H.264/AVC 视频编解码标准中的应用。在 H.264 视频流中，NALU 是视频数据的基本单元。每个 NALU 包含了编码视频的一部分数据，它们一起构成了视频的一个帧或一组帧。
//
// H.264 视频编码使用了一种分层结构，其中最底层是 NALU。NALU 的设计使得视频流可以适应不同的网络层，包括 TCP/IP 和无线网络。这个层次结构的设计意味着相同的视频数据可以在不同的网络条件和协议中传输，而不需要重新编码。
//
// 在编码层面，NALU 包含了一个头部和有效载荷（payload）。头部主要包含了 NALU 的类型信息，而有效载荷包含了实际的编码视频数据。NALU 的类型决定了它包含的数据是视频的一个关键帧（I 帧）、预测帧（P 帧或 B 帧）还是其他类型的数据（比如序列参数集（SPS）或图像参数集（PPS））。
//
// 在你提供的代码中，`H264Nalu` 很可能是一个类或结构体，表示 H.264 编码的视频流中的一个 NALU 单元。它可能包含了如下信息：
//
// - NALU 的大小（可能是 `FullSize` 字段）
// - NALU 的类型（比如是否为关键帧）
// - NALU 的具体编码数据

pub struct H264Nalu {
    start_position: i32,
    full_size: u32,
    type_of: H264NaluType,
    nalu_hash: Option<String>
}

impl H264Nalu {
    pub fn new( start_position: i32,
                full_size: u32,
                type_of: H264NaluType,) -> Self {
        Self {
            start_position,
            full_size,
            type_of,
            nalu_hash: None
        }
    }

    pub async fn parse_nalus<R>(data: &mut R) -> Result<Vec<H264Nalu>>
        where
            R: AsyncReadExt + Unpin,
    {
        let mut h264_nalus = vec![];
        let mut b = [0u8; 4];

        data.seek(5).await?;

        while let Ok(()) = data.read_exact(&mut b).await {
            let size = u32::from_be_bytes(b);
            if let Some(nalu_type) = Self::parse_nalu_type(data.read_u8().await?) {
                let start_position = data.stream_position().await? as usize - 1;
                let nalu = Self::new(start_position, size, nalu_type);
                data.seek(size as i64 - 1).await?;
                h264_nalus.push(nalu);
            } else {
                return anyhow::anyhow!( "Invalid NALU type");
            }
        }

        Ok(h264_nalus)
    }

    pub fn parse_nalu_type(first_byte: u8) -> Option<H264NaluType> {
        if first_byte & 0b10000000 != 0 {
            None
        } else {
           Some( unsafe { std::mem::transmute(first_byte & 0b00011111) })
        }
    }
}

#[repr(u8)]
pub enum H264NaluType {
    Unspecified0 = 0,
    CodedSliceOfANonIdrPicture = 1,
    CodedSliceDataPartitionA = 2,
    CodedSliceDataPartitionB = 3,
    CodedSliceDataPartitionC = 4,
    CodedSliceOfAnIdrPicture = 5,
    Sei = 6,
    Sps = 7,
    Pps = 8,
    AccessUnitDelimiter = 9,
    EndOfSequence = 10,
    EndOfStream = 11,
    FillerData = 12,
    SpsExtension = 13,
    PrefixNalUnit = 14,
    SubsetSps = 15,
    DepthParameterSet = 16,
    Reserved17 = 17,
    Reserved18 = 18,
    SliceLayerWithoutPartitioning = 19,
    SliceLayerExtension20 = 20,
    SliceLayerExtension21 = 21,
    Reserved22 = 22,
    Reserved23 = 23,
    Unspecified24 = 24,
    Unspecified25 = 25,
    Unspecified26 = 26,
    Unspecified27 = 27,
    Unspecified28 = 28,
    Unspecified29 = 29,
    Unspecified30 = 30,
    Unspecified31 = 31,
}

impl Default for H264NaluType {
    fn default() -> Self {
        H264NaluType::Unspecified0
    }
}

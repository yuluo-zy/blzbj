use std::collections::HashMap;
use std::io;
use bytes::{Buf, Bytes};
use tokio::io::{AsyncRead, AsyncReadExt};
use crate::error::AVCError;

type Result<T> =  std::result::Result<T, AVCError>;
/// 高级视频编码（AVC）解码器配置记录。
/// 包含配置AVC解码器所需的信息。
#[derive(Debug, Clone)]
pub struct AVCDecoderConfigurationRecord {
    /// 配置版本。
    configuration_version: u8,
    /// AVC配置文件指示，定义了流的配置文件。
    avc_profile_indication: u8,
    /// 配置文件兼容性，指示对特定配置文件的支持。
    profile_compatibility: u8,
    /// 级别指示，代表流的AVC级别。
    avc_level_indication: u8,
    /// 流中NAL单元大小前面的字节数减一。
    length_size_minus_one: u8,
    /// 序列参数集的数量。
    num_of_sequence_parameter_sets: u8,
    /// 序列参数集列表。
    sequence_parameter_sets: Vec<SequenceParameterSet>,
    /// 图像参数集的数量。
    num_of_picture_parameter_sets: u8,
    /// 图像参数集列表。
    picture_parameter_sets: Vec<PictureParameterSet>,
}
/// 序列参数集（SPS），包含一系列连续编码视频图像的相关参数。
#[derive(Debug, Clone)]
pub struct SequenceParameterSet {
    /// SPS NAL单元的长度。
    sequence_parameter_set_length: u16,
    /// SPS NAL单元的原始字节内容。
    sequence_parameter_set_nal_unit: Vec<u8>,
}

/// 图像参数集（PPS），包含解码一个或多个单独图像的相关参数。
#[derive(Debug, Clone)]
pub struct PictureParameterSet {
    /// PPS NAL单元的长度。
    picture_parameter_set_length: u16,
    /// PPS NAL单元的原始字节内容。
    picture_parameter_set_nal_unit: Vec<u8>,
}
impl AVCDecoderConfigurationRecord {
    pub async fn parse(reader:&mut Bytes) -> Result<Self> {
        let configuration_version = reader.get_u8();
        let avc_profile_indication = reader.get_u8();
        let profile_compatibility = reader.get_u8();
        let avc_level_indication = reader.get_u8();
        let length_size_minus_one = reader.get_u8() & 0b11;
        let num_of_sequence_parameter_sets = reader.get_u8() & 0b11111;
        let mut sequence_parameter_sets = Vec::new();
        for _ in 0..num_of_sequence_parameter_sets {
            let sequence_parameter_set_length = reader.get_u16();
            let mut sequence_parameter_set_nal_unit = vec![0u8; sequence_parameter_set_length as usize];
            if reader.remaining() >= sequence_parameter_set_length as usize {
                // 读取字节到数组中
                reader.copy_to_slice(&mut sequence_parameter_set_nal_unit);
            } else {
               return Err(AVCError::ParameterLength);
            }
            sequence_parameter_sets.push(SequenceParameterSet {
                sequence_parameter_set_length,
                sequence_parameter_set_nal_unit
            });
        }
        let num_of_picture_parameter_sets = reader.get_u8();
        let mut picture_parameter_sets = Vec::new();
        for _ in 0..num_of_picture_parameter_sets {
            let picture_parameter_set_length = reader.get_u16();
            let mut picture_parameter_set_nal_unit = vec![0u8; picture_parameter_set_length as usize];
            if reader.remaining() >= picture_parameter_set_length as usize {
                // 读取字节到数组中
                reader.copy_to_slice(&mut picture_parameter_set_nal_unit);
            } else {
                return Err(AVCError::ParameterLength);
            }
            picture_parameter_sets.push(PictureParameterSet {
                picture_parameter_set_length,
                picture_parameter_set_nal_unit
            });
        }

        Ok(AVCDecoderConfigurationRecord {
            configuration_version,
            avc_profile_indication,
            profile_compatibility,
            avc_level_indication,
            length_size_minus_one,
            num_of_sequence_parameter_sets,
            sequence_parameter_sets,
            num_of_picture_parameter_sets,
            picture_parameter_sets,
        })
    }
}




/// 网络抽象层（NAL）单元。
/// 包含基本的视频编码数据。
#[derive(Debug, Clone)]
pub struct NalUnit {
    /// 禁止为零的比特，必须为0，用于确保NAL单元的同步。
    forbidden_zero_bit: u8,
    /// NAL引用指示（NAL reference indicator），指示NAL单元的重要性。
    nal_ref_idc: u8,
    /// NAL单元类型，定义了NAL单元的负载类型。
    nal_unit_type: u8,
    /// 原始字节序列载荷（Raw Byte Sequence Payload）。
    rbsp_bytes: Vec<u8>,
}

impl NalUnit {
    pub async fn parse(reader: &mut Vec<u8>) -> Result<Self>{


        let byte =  reader.get_u8();
        let forbidden_zero_bit = byte >> 7;
        let nal_ref_idc = (byte >> 5) & 0b0000_0011;
        let nal_unit_type = byte & 0b0001_1111;

        // extensions ignored
        if [14, 20, 21].contains(&nal_unit_type) {
            return Err(AVCError::UnknownNAL(nal_unit_type));
        }
        let mut rbsp_bytes = Vec::new();
        let mut zero_count = 0;
        while reader.has_remaining() {
            let byte = reader.get_u8();
            match byte {
                0 => zero_count += 1,
                3 if zero_count >= 2 => {
                    // Skip the emulation prevention byte
                    zero_count = 0; // Reset the zero count after the emulation byte
                }
                _ => {
                    if zero_count > 0 {
                        rbsp_bytes.extend(vec![0x00; zero_count]);
                        zero_count = 0;
                    }
                    rbsp_bytes.push(byte);
                }
            }
        }

        Ok(NalUnit {
            forbidden_zero_bit,
            nal_ref_idc,
            nal_unit_type,
            rbsp_bytes,
        })
    }
}

/// 其用途是为了从视频流的编码参数中派生出色度采样的子宽度（SubWidthC）和子高度（SubHeightC）的值。
/// 这些参数是对于色度（chroma）分量的采样与亮度（luma）分量采样的水平和垂直分辨率的比率。
const SUB_WIDTH_HEIGHT_MAPPING: HashMap<u8, (u8, u8)> = [
    (1, (2, 2)),
    (2, (2, 1)),
    (3, (1, 1)),
].iter()
    .copied()
    .collect::<HashMap<u8, (u8, u8)>>();


/// 序列参数集数据（Sequence Parameter Set Data）。
/// 包含编码视频序列的关键参数。
#[derive(Debug, Clone)]
pub struct SequenceParameterSetData {
    /// 配置文件标识符。
    profile_idc: u8,
    /// 约束集标志0。
    constraint_set0_flag: u8,
    /// 约束集标志1。
    constraint_set1_flag: u8,
    /// 约束集标志2。
    constraint_set2_flag: u8,
    /// 约束集标志3。
    constraint_set3_flag: u8,
    /// 约束集标志4。
    constraint_set4_flag: u8,
    /// 约束集标志5。
    constraint_set5_flag: u8,
    /// 级别标识符。
    level_idc: u8,
    /// 序列参数集标识符。
    seq_parameter_set_id: u8,
    /// 色彩格式标识符。
    chroma_format_idc: u8,
    /// 独立颜色平面标志。
    separate_colour_plane_flag: u8,
    /// 亮度位深度减8。
    bit_depth_luma_minus8: u8,
    /// 色度位深度减8。
    bit_depth_chroma_minus8: u8,
    /// 变换旁路标志。
    qpprime_y_zero_transform_bypass_flag: u8,
    /// 序列缩放矩阵存在标志。
    seq_scaling_matrix_present_flag: u8,
    /// 序列缩放列表存在标志列表。
    seq_scaling_list_present_flag: Vec<u8>,
    /// 最大帧数减4的对数。
    log2_max_frame_num_minus4: u8,
    /// 图片顺序计数类型。
    pic_order_cnt_type: u8,
    /// 图片顺序计数的最大值减4的对数。
    log2_max_pic_order_cnt_lsb_minus4: u8,
    /// 图片顺序总是零的标志。
    delta_pic_order_always_zero_flag: u8,
    /// 非参考图片的偏移。
    offset_for_non_ref_pic: i32,
    /// 自上而下场的偏移。
    offset_for_top_to_bottom_field: i32,
    /// 图片顺序计数周期中参考框架的数量。
    num_ref_frames_in_pic_order_cnt_cycle: u8,
    /// 参考帧的偏移列表。
    offset_for_ref_frame: Vec<i32>,
    /// 参考帧的最大数量。
    max_num_ref_frames: u8,
    /// 允许帧编号中存在间隙的标志。
    gaps_in_frame_num_value_allowed_flag: u8,
    /// 宏块宽度减1。
    pic_width_in_mbs_minus1: u16,
    /// 地图单位高度减1。
    pic_height_in_map_units_minus1: u16,
    /// 仅帧宏块标志。
    frame_mbs_only_flag: u8,
    /// 宏块自适应帧场标志。
    mb_adaptive_frame_field_flag: u8,
    /// 直接8x8推断标志。
    direct_8x8_inference_flag: u8,
    /// 帧裁剪标志。
    frame_cropping_flag: u8,
    /// 帧裁剪左偏移。
    frame_crop_left_offset: u16,
    /// 帧裁剪右偏移。
    frame_crop_right_offset: u16,
    /// 帧裁剪上偏移。
    frame_crop_top_offset: u16,
    /// 帧裁剪下偏移。
    frame_crop_bottom_offset: u16,
    /// 视频可用性信息存在标志。
    vui_parameters_present_flag: u8,
}

impl SequenceParameterSetData {
    pub fn chroma_array_type(&self) -> u8 {
        if self.separate_colour_plane_flag == 0 {
            self.chroma_format_idc
        } else {
            0
        }
    }

    // 获取 SubWidthC
    pub fn sub_width_c(&self) -> Result<u8> {
       match  SUB_WIDTH_HEIGHT_MAPPING.get(&self.chroma_format_idc){
           None => { Err(AVCError::SubWidthCUndefined) }
           Some(item) => { Ok(item.0) }
       }

    }

    // 获取 SubHeightC
    pub fn sub_height_c(&self) -> Result<u8> {
        match  SUB_WIDTH_HEIGHT_MAPPING.get(&self.chroma_format_idc){
            None => { Err(AVCError::SubHeightCUndefined) }
            Some(item) => { Ok(item.1) }
        }
    }

    // 获取宏块的宽度 C
    pub fn mb_width_c(&self) -> u8 {
        match self.chroma_array_type() {
            0 | 1 => 0,
            _ => 16 / self.sub_width_c().unwrap_or(1),
        }
    }

    // 获取宏块的高度 C
    pub fn mb_height_c(&self) -> u8 {
        match self.chroma_array_type() {
            0 | 1 => 0,
            _ => 16 / self.sub_height_c().unwrap_or(1),
        }
    }

    // 获取图片宽度（以宏块为单位）
    pub fn pic_width_in_mbs(&self) -> usize {
        (self.pic_width_in_mbs_minus1 + 1) as usize
    }

    pub fn pic_width_in_samples_l(&self) -> usize {
        self.pic_width_in_mbs() * 16
    }

    pub fn pic_width_in_samples_c(&self) -> usize {
        self.pic_width_in_mbs() * self.mb_width_c()
    }
    pub fn pic_height_in_map_units(&self) -> usize {
        (self.pic_height_in_map_units_minus1 + 1) as usize
    }

    // 获取图片大小（以地图单元为单位）
    pub fn pic_size_in_map_units(&self) -> usize {
        self.pic_width_in_mbs() * self.pic_height_in_map_units()
    }

    // 获取帧高度（以宏块为单位）
    pub fn frame_height_in_mbs(&self) -> usize {
        (2 - self.frame_mbs_only_flag as usize) * self.pic_height_in_map_units()
    }

    // 获取裁剪单位 X
    pub fn crop_unit_x(&self) -> usize {
        if self.chroma_array_type() == 0 {
            1
        } else {
            self.sub_width_c().unwrap_or(1) as usize
        }
    }

    // 获取裁剪单位 Y
    pub fn crop_unit_y(&self) -> usize {
        if self.chroma_array_type() == 0 {
            2 - self.frame_mbs_only_flag as usize
        } else {
            self.sub_height_c().unwrap_or(1) as usize * (2 - self.frame_mbs_only_flag as usize)
        }
    }

    // 获取帧宽度
    pub fn frame_width(&self) -> usize {
        let x0 = self.crop_unit_x() * self.frame_crop_left_offset;
        let x1 = self.pic_width_in_samples_l() - (self.crop_unit_x() * self.frame_crop_right_offset + 1);
        x1 - x0 + 1 // x1 inclusive
    }

    // 获取帧高度
    pub fn frame_height(&self) -> usize {
        let y0 = self.crop_unit_y() * self.frame_crop_top_offset;
        let y1 = 16 * self.frame_height_in_mbs() - (self.crop_unit_y() * self.frame_crop_bottom_offset + 1);
        y1 - y0 + 1 // y1 inclusive
    }

    pub async fn parse<R>(reader: &mut Vec<u8>) -> Result<Self> {
        let mut bit_reader = BitReader::new(reader);
        let profile_idc = bit_reader.read_bits_as_int(8)? as u8;
        let constraint_set0_flag = bit_reader.read_bits_as_int(1)? as u8;
       let constraint_set1_flag = bit_reader.read_bits_as_int(1)? as u8;
        let constraint_set2_flag = bit_reader.read_bits_as_int(1)? as u8;
        let constraint_set3_flag = bit_reader.read_bits_as_int(1)? as u8;
        let constraint_set4_flag = bit_reader.read_bits_as_int(1)? as u8;
        let constraint_set5_flag = bit_reader.read_bits_as_int(1)? as u8;
        let reserved_zero_2bits = bit_reader.read_bits_as_int(2)? as u8;
        let level_idc = bit_reader.read_bits_as_int(8)? as u8;
        let seq_parameter_set_id = bit_reader.read_ue()?;
        let  chroma_format_idc = 1u8;
        let separate_colour_plane_flag = 0u8;
        let bit_depth_luma_minus8 = 0u8;
        let mut bit_depth_chroma_minus8 = 0u8;
        let mut qpprime_y_zero_transform_bypass_flag = 0u8;
        let mut seq_scaling_matrix_present_flag = 0u8;
        let mut seq_scaling_list_present_flag = Vec::new();
        let mut separate_colour_plane_flag= 0u8;
        let mut bit_depth_luma_minus8 = 0u8;
        if [100, 110, 122, 244, 44, 83, 86, 118, 128, 138, 139, 134, 135].contains(&profile_idc) {
            let chroma_format_idc = bit_reader.read_ue()?;
            if chroma_format_idc == 3{
                separate_colour_plane_flag = bit_reader.read_bits_as_int(1)? as u8;
            }
            bit_depth_luma_minus8 = bit_reader.read_ue()? as u8;
            bit_depth_chroma_minus8 = bit_reader.read_ue()? as u8;
            qpprime_y_zero_transform_bypass_flag = reader.read_bits_as_int(1)? as u8;
            seq_scaling_matrix_present_flag = reader.read_bits_as_int(1)? as u8;
            if seq_scaling_matrix_present_flag != 0{
                let num_scaling_lists = if chroma_format_idc != 3 { 8 } else { 12 };
                for _ in 0..num_scaling_lists {
                    let flag = reader.read_bits_as_int(1)? as u8;
                    seq_scaling_list_present_flag.push(flag);
                    if flag != 0 {
                        // todo 缩放向量
                    }
                }
            }
        }

        let log2_max_frame_num_minus4 = bit_reader.read_ue()?;
        let pic_order_cnt_type = bit_reader.read_ue()?;

        let mut log2_max_pic_order_cnt_lsb_minus4 = 0;
        let mut delta_pic_order_always_zero_flag = 0;
        let mut offset_for_non_ref_pic = 0;
        let mut offset_for_top_to_bottom_field = 0;
        let mut num_ref_frames_in_pic_order_cnt_cycle = 0;
        let mut offset_for_ref_frame = vec![];

        if pic_order_cnt_type == 0 {
            log2_max_pic_order_cnt_lsb_minus4 = bit_reader.read_ue()?;
        } else if pic_order_cnt_type == 1 {
            delta_pic_order_always_zero_flag = bit_reader.read_bits_as_int(1)?;
            offset_for_non_ref_pic = bit_reader.read_se()?;
            offset_for_top_to_bottom_field = bit_reader.read_se()?;
            num_ref_frames_in_pic_order_cnt_cycle = bit_reader.read_ue()?;
            for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
                let offset = bit_reader.read_se()?;
                offset_for_ref_frame.push(offset);
            }
        }

        let max_num_ref_frames = bit_reader.read_ue()?;
        let gaps_in_frame_num_value_allowed_flag = bit_reader.read_bits_as_int(1)?;
        let pic_width_in_mbs_minus1 = bit_reader.read_ue()?;
        let pic_height_in_map_units_minus1 = bit_reader.read_ue()?;
        let frame_mbs_only_flag = bit_reader.read_bits_as_int(1)?;

        let mut mb_adaptive_frame_field_flag = 0;
        if frame_mbs_only_flag == 0 {
            mb_adaptive_frame_field_flag = bit_reader.read_bits_as_int(1)?;
        }

        let direct_8x8_inference_flag = bit_reader.read_bits_as_int(1)?;
        let frame_cropping_flag = bit_reader.read_bits_as_int(1)?;

        let mut frame_crop_left_offset = 0;
        let mut frame_crop_right_offset = 0;
        let mut frame_crop_top_offset = 0;
        let mut frame_crop_bottom_offset = 0;

        if frame_cropping_flag != 0 {
            frame_crop_left_offset = bit_reader.read_ue()?;
            frame_crop_right_offset = bit_reader.read_ue()?;
            frame_crop_top_offset = bit_reader.read_ue()?;
            frame_crop_bottom_offset = bit_reader.read_ue()?;
        }

        let vui_parameters_present_flag = bit_reader.read_bits_as_int(1)?;
        Ok(SequenceParameterSetData{
            profile_idc,
            constraint_set0_flag,
            constraint_set1_flag,
            constraint_set2_flag,
            constraint_set3_flag,
            constraint_set4_flag,
            constraint_set5_flag,
            level_idc,
            seq_parameter_set_id,
            chroma_format_idc,
            separate_colour_plane_flag,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            qpprime_y_zero_transform_bypass_flag,
            seq_scaling_matrix_present_flag,
            seq_scaling_list_present_flag,
            log2_max_frame_num_minus4,
            pic_order_cnt_type,
            log2_max_pic_order_cnt_lsb_minus4,
            delta_pic_order_always_zero_flag,
            offset_for_non_ref_pic,
            offset_for_top_to_bottom_field,
            num_ref_frames_in_pic_order_cnt_cycle,
            offset_for_ref_frame,
            max_num_ref_frames,
            gaps_in_frame_num_value_allowed_flag,
            pic_width_in_mbs_minus1,
            pic_height_in_map_units_minus1,
            frame_mbs_only_flag,
            mb_adaptive_frame_field_flag,
            direct_8x8_inference_flag,
            frame_cropping_flag,
            frame_crop_left_offset,
            frame_crop_right_offset,
            frame_crop_top_offset,
            frame_crop_bottom_offset,
            vui_parameters_present_flag,
        })
    }

    pub fn scaling_list(&mut self, bit_reader: &mut BitReader, list_size: usize) -> Option<Vec<i32>> {
        let mut last_scale = 8;
        let mut next_scale = 8;
        let mut scaling_list = Vec::with_capacity(list_size);

        for _ in 0..list_size {
            if next_scale != 0 {
                let delta_scale = bit_reader.read_se()?;
                next_scale = (last_scale + delta_scale + 256) % 256;
            }
            scaling_list.push(next_scale as i32);
            if next_scale != 0 {
                last_scale = next_scale;
            }
        }

        Some(scaling_list)
    }

}


struct BitReader<'a> {
    bytes: &'a Bytes,
    byte_index: usize,
    bit_index: u8,
}

/// 0阶指数哥伦布编码, 第一个非零比特之后的m+k比特的十进制值为value。
impl BitReader {
    fn new(bytes: &Bytes) -> Self {
        Self {
            bytes,
            byte_index: 0,
            bit_index: 0,
        }
    }

    fn read_bit(&mut self) -> Option<bool> {
        if self.byte_index >= self.bytes.len() {
            return None; // 没有更多的字节来读取比特
        }

        let byte = self.bytes[self.byte_index];
        let bit = byte & (1 << (7 - self.bit_index)) != 0;

        self.bit_index += 1;
        if self.bit_index > 7 {
            self.bit_index = 0;
            self.byte_index += 1;
        }

        Some(bit)
    }

    fn read_bits_as_int(&mut self, n: usize) -> Option<u32> {
        let mut value = 0;
        for _ in 0..n {
            value <<= 1;
            if let Some(bit) = self.read_bit() {
                if bit {
                    value |= 1;
                }
            } else {
                return None; // Not enough bits available
            }
        }
        Some(value)
    }

    fn read_ue(&mut self) -> Option<u32> {
        let mut leading_zero_bits = 0;

        // Count the leading zero bits
        while let Some(false) = self.read_bit() {
            leading_zero_bits += 1;
        }

        // Read the bits following the leading zeros as integer
        let bits_as_int = self.read_bits_as_int(leading_zero_bits)?;

        Some((1 << leading_zero_bits) - 1 + bits_as_int)
    }

    fn read_se(&mut self) -> Option<i32> {
        let code_num = self.read_ue()?;
        let value = if code_num % 2 == 0 {
            (code_num / 2) as i32
        } else {
            -((code_num + 1) / 2) as i32
        };
        Some(value)
    }

}

pub async fn extract_resolution(packet: &mut Bytes) -> Result<(usize, usize)> {
    let mut record = AVCDecoderConfigurationRecord::parse(packet).await?;
    let mut sps = record.sequence_parameter_sets[0].sequence_parameter_set_nal_unit.clone();
    let nal_unit = NalUnit::parse(&mut sps).await?;
    let mut nal_rbsp = nal_unit.rbsp_bytes.clone();
    let sps_data = SequenceParameterSetData::parse(&mut nal_rbsp).await?;
    Ok((sps_data.frame_width(), sps_data.frame_height()))
}

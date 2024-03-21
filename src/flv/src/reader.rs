use anyhow::Result;
use blake3::IncrementCounter::No;
use bytes::{Buf, BufMut, BytesMut};
use nom::{IResult, Needed};
use tokio::io::{AsyncRead, AsyncReadExt};
use crate::parser::{complete_tag, header};

use crate::tag::{Header, Tag, TagType};

pub struct FlvTagPipeReader{
    buffer: BytesMut,
    file_header: bool
}

impl FlvTagPipeReader {
    pub async fn new() -> Result<Self> {
        Ok(FlvTagPipeReader{
            buffer : BytesMut::with_capacity(4096),
            file_header: false
        } )
    }

    pub async fn read_next_tag<R: AsyncRead + Unpin>(&mut self,  reader:&mut R) -> Result<Option<Tag>> {

        loop {
            // 为新数据预留空间
            if self.buffer.remaining_mut() < 512 {
                self.buffer.reserve(4096);
            }

            let n = match reader.read_buf(&mut self.buffer).await {
                Ok(0) => {
                    // 读取到 0 字节表示连接关闭
                    return Ok(None);
                }
                Ok(n) => n,
                Err(e) => {
                    return Err(anyhow::anyhow!("failed to read from socket; err = {:?}", e));
                }
            };

            // 只要 buffer 有内容就尝试解析
            loop {
                match self.parse_data() {
                    Ok((remaining, parsed_data)) => {
                        // 从 buffer 中移除已经解析的数据
                        let consumed = self.buffer.len() - remaining.len();
                        self.buffer.advance(consumed);
                        return Ok(Some(parsed_data));
                    }
                    Err(nom::Err::Incomplete(_)) => {
                        break;
                    }
                    Err(_) => {
                        // 解析错误，可以选择如何处理
                        // ...
                        break;
                    }
                }
            }
        }
    }

    pub fn parse_data(&mut self,) ->  IResult<&[u8], Tag>{
        if !self.file_header {
            if self.buffer.remaining() <9 {
                return Err(nom::Err::Incomplete(Needed::Unknown))
            }
            match header(&self.buffer[..9]) {
                Ok(_) => {
                    self.file_header = true;
                    self.buffer.advance(9);
                }
                Err(_) => {
                    return Err(nom::Err::Incomplete(Needed::Unknown))
                }
            }
        }
        if self.buffer.remaining() < 4  {
            return Err(nom::Err::Incomplete(Needed::Unknown))
        }
        
        return  complete_tag(&self.buffer[4..])
    }
}
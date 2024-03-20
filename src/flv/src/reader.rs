use std::sync::Arc;
use anyhow::Result;
use blake3::IncrementCounter::No;
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use crate::tag::Tag;

struct FlvTagPipeReader {
    reader: TcpStream,
    semaphore: Arc<Semaphore>,
    tag_index: i32,
    skip_data: bool,
    leave_open: bool,
    peek: bool,
    file_header: bool,
    peek_tag: Option<Tag>,
}

impl FlvTagPipeReader {
    pub async fn new(reader: TcpStream) -> Result<Self> {
        Ok(FlvTagPipeReader {
            reader,
            semaphore: Arc::new(Semaphore::new(1)),
            tag_index: 0,
            skip_data: false,
            leave_open: false,
            peek: false,
            file_header: false,
            peek_tag: None,
        })
    }

    pub async fn read_next_tag(&mut self) -> Result<Option<Tag>> {
        // 处理 FLV 标签的异步读取逻辑...
        let mut buffer = BytesMut::new();

        loop {
            let num = self.reader.read_buf(&mut buffer).await?;

            if num == 0 && buffer.is_empty() {
                return Ok(None);
            }

            // 尝试分析文件头

            if !self.file_header {
                if Self::parse_file_header(&mut buffer).await? {
                    self.file_header = true;
                } else {
                    // 如果没有足够的数据来解析文件头，继续读取。
                    continue;
                }
            }
        }


        Ok(None) // 返回解析到的 Tag 或 None
    }

    pub async fn parse_file_header(buffer: &mut BytesMut) -> Result<bool> {
        if buffer.remaining() < 9 {
            return Ok(false);
        }
        let mut header = buffer.split_to(9);
        let data = header.as_ref();

        if data[0] != b'F' || data[1] != b'L' || data[2] != b'V' || data[3] != 1 {
            return Err(anyhow::anyhow!("Data is not FLV."));
        }

        if data[5] != 0 || data[6] != 0 || data[7] != 0 || data[8] != 9 {
            return Err(anyhow::anyhow!("Not Supported FLV format."));
        }
        Ok(true)
    }

    pub async fn parse_tag_data(buffer: &mut BytesMut) -> Result<bool> {
        if buffer.remaining() < 11 + 4 {
            return Ok(false);
        };
        let tag_header_slice = buffer.slice(4..4 + 11);

        Ok(false)
    }

    pub async fn peek_tag(&mut self) -> Result<Option<Tag>> {
        let _permit = self.semaphore.acquire().await.unwrap();
        // 调用 read_next_tag，但不实际消耗数据...
        // ...

        Ok(None) // 返回解析到的 Tag 或 None
    }

    pub async fn read_tag(&mut self) -> Result<Option<Tag>> {
        let _permit = self.semaphore.acquire().await.unwrap();
        // 调用 read_next_tag 并消耗数据...
        // ...

        Ok(None) // 返回解析到的 Tag 或 None
    }
}
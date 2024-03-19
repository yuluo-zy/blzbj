use anyhow::Result;
use crate::tag::Tag;

#[async_trait::async_trait]
pub trait FlvTagReader {
    // 异步预览下一个标签，但不从流中消费它。
    async fn peek_tag_async(&mut self) -> Result<Option<Tag>>;

    // 异步读取下一个标签。
    async fn read_tag_async(&mut self) -> Result<Option<Tag>>;
}
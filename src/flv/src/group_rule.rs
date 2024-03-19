use crate::tag::Tag;

trait GroupingRule {
    // 判断一个标签是否可以作为一个新的组的起始
    fn can_start_with(&self, tag: &Tag) -> bool;

    // 判断一个标签是否可以被附加到现有标签列表中
    fn can_append_with(&self, tag: &Tag, tags: &[Tag]) -> bool;

    // 基于一组标签创建一个管道操作
    fn create_pipeline_action(&self, tags: &[Tag]) -> PipelineAction;
}

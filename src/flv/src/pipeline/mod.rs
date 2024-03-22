// use num_enum::TryFromPrimitive;
// use serde::Serialize;
//
// mod actions;
// mod rules;
// mod pipeline_builder;
// mod processing_comment;
// mod processing_rule;
// mod processing_context;
//
//
// #[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, Serialize)]
// #[repr(u8)]
// pub enum CommentType {
//     Other = 0,
//     Logging,
//     Unrepairable,
//     TimestampJump,
//     TimestampOffset,
//     DecodingHeader,
//     RepeatingData,
//     OnMetaData,
// }
//
//
// pub struct PipelineSettings {
//     split: bool
// }
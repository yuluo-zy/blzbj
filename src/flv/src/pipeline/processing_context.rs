use std::collections::HashMap;
use crate::pipeline::processing_comment::ProcessingComment;

pub struct State {
    // actions: Vec<PipelineAction>,
    session_items: HashMap<String, Option<String>>,
    local_items: HashMap<String, Option<String>>,
    comments: Vec<ProcessingComment>,
}

impl State {
    pub fn add_comment(&mut self,value: ProcessingComment) {
        self.comments.push(value)
    }
    //
    // pub fn per_action_run<F>(&mut self,  mut func: F)  -> bool
    //     where
    //         F: FnMut(&State, &PipelineAction) -> Option<Vec<PipelineAction>>,
    // {
    //     let mut success = true;
    //     let mut result = Vec::new();
    //
    //     for action in &self.actions {
    //         if let Some(outputs) = func(self, action) {
    //             result.extend(outputs);
    //         } else {
    //             success = false;
    //             break;
    //         }
    //     }
    //
    //     self.actions = result;
    //     success
    // }
}
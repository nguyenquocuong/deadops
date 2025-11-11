use anyhow::Result;
use rig::{agent::Agent, providers::huggingface::completion::CompletionModel, tool::Tool};

use crate::{
    common::{huggingface_agent_builder, openai_agent_builder},
    jenkins::tools::{BuildJobTool, SearchJobTool},
};

pub async fn create_jenkins_agent(preamble: Option<String>) -> Result<Agent<CompletionModel>> {
    let preamble = preamble.unwrap_or(format!(
        "{} {}",
        &format!(
            "You are an assistant here to help the user perform Jenkins tasks.
            Follow these instructions closely.
            1. Use the {} tool to search for the job name.
            2. Use the {} tool to build the job with the found job name.
            ",
            SearchJobTool::NAME,
            BuildJobTool::NAME,
        ),
        ""
    ));

    Ok(huggingface_agent_builder()
        .preamble(&preamble)
        .max_tokens(1024)
        .tool(SearchJobTool)
        .tool(BuildJobTool)
        .build())
}

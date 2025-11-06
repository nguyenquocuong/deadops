use anyhow::Result;
use rig::agent::Agent;

use crate::{
    common::{huggingface_agent_builder, openai_agent_builder},
    BuildJob, SearchJob,
};

pub async fn create_jenkins_agent(
    preamble: Option<String>,
) -> Result<Agent<impl rig::completion::CompletionModel>> {
    let preamble = preamble.unwrap_or(format!(
        "{} {}",
        "You are an assistant here to help the user perform Jenkins tasks.", ""
    ));

    Ok(huggingface_agent_builder()
        .preamble(&preamble)
        .max_tokens(1024)
        .tool(SearchJob)
        .tool(BuildJob)
        .build())
}

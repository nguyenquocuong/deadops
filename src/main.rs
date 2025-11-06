use rig::integrations::cli_chatbot::ChatBotBuilder;

use crate::{
    common::huggingface_agent_builder,
    jenkins::{
        agent::create_jenkins_agent,
        tools::{BuildJob, JenkinsTool, SearchJob},
    },
};

mod common;
mod jenkins;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let jenkins_agent = create_jenkins_agent(Some(
        "You are an assistant here to help the user perform Jenkins tasks.
        Follow these instructions closely:
        1. Use the search_job tool to search for the job name.
        2. Use the build_job tool to build the job with the found job name.
        "
        .to_string(),
    ))
    .await?;

    let jenkins_tool = JenkinsTool(jenkins_agent);

    let multi_agent_system = huggingface_agent_builder()
        .preamble("You are a helpful DevSecOps assistant here to help the user perform daily automation tasks.")
        .tool(jenkins_tool)
        .build();

    let chatbot = ChatBotBuilder::new()
        .agent(multi_agent_system)
        // .multi_turn_depth(10)
        .build();

    chatbot.run().await?;

    Ok(())
}

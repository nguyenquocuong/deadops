use rig::{integrations::cli_chatbot::ChatBotBuilder, tool::Tool};

use crate::{
    common::{huggingface_agent_builder, openai_agent_builder},
    jenkins::{
        agent::create_jenkins_agent,
        tools::{BuildJob, JenkinsTool, SearchJob},
    },
};

mod common;
mod jenkins;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // tracing_subscriber::fmt().init();

    // let client = providers::ollama::Client::new();
    // let model = "gpt-oss:20b";

    // let client = providers::openai::Client::new("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");

    let jenkins_agent = create_jenkins_agent(Some(format!(
        "You are an assistant here to help the user perform Jenkins tasks.
        Follow these instructions closely:
        1. Use the {} tool to search for the job name.
        2. Use the {} tool to build the job with the found job name.
        ",
        SearchJob::NAME,
        BuildJob::NAME
    )))
    .await?;

    let jenkins_tool = JenkinsTool(jenkins_agent);

    let multi_agent_system = huggingface_agent_builder()
        .preamble("You are a helpful DevSecOps assistant here to help the user perform daily automation tasks.")
        .tool(jenkins_tool)
        .build();

    let chatbot = ChatBotBuilder::new()
        .agent(multi_agent_system)
        .multi_turn_depth(10)
        .build();

    chatbot.run().await?;

    Ok(())
}

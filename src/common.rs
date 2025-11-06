use rig::agent::AgentBuilder;
use rig::prelude::*;

pub fn huggingface_agent_builder() -> AgentBuilder<impl rig::completion::CompletionModel> {
    rig::providers::huggingface::ClientBuilder::<reqwest::Client>::new("")
        .base_url("http://localhost:8081")
        .build()
        .unwrap()
        .agent("qwen3")
}

pub fn openai_agent_builder() -> AgentBuilder<impl rig::completion::CompletionModel> {
    rig::providers::openai::Client::from_env().agent("gpt-4")
}

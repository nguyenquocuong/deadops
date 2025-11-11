use rig::agent::AgentBuilder;
use rig::prelude::*;
use rig::providers::huggingface::completion::CompletionModel as HuggingfaceCompletionModel;
use rig::providers::openai::responses_api::ResponsesCompletionModel as OpenAICompletionModel;

pub fn huggingface_agent_builder() -> AgentBuilder<HuggingfaceCompletionModel> {
    rig::providers::huggingface::ClientBuilder::<reqwest::Client>::new("")
        .base_url("http://localhost:8081")
        .build()
        .unwrap()
        .agent("qwen3")
}

pub fn openai_agent_builder() -> AgentBuilder<OpenAICompletionModel> {
    rig::providers::openai::Client::from_env().agent("gpt-4")
}

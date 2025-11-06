use anyhow::Result;
use rig::{
    agent::Agent,
    completion::{CompletionModel, Prompt, ToolDefinition},
    tool::Tool,
};
use rig_tool_macro::tool;
use serde::Deserialize;
use serde_json::json;

// Define a wrapper around an agent so that it can be provided to another agent
// as a tool
pub struct JenkinsTool<M: CompletionModel>(pub Agent<M>);

// The input that will be sent to the translator agent from the main agent
#[derive(Deserialize)]
pub struct JenkinsArgs {
    prompt: String,
}

impl<M: CompletionModel> Tool for JenkinsTool<M> {
    const NAME: &'static str = "jenkins";

    type Args = JenkinsArgs;
    type Error = rig::completion::PromptError;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Retrieve, build Jenkins jobs.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "The task need to execute."
                    },
                },
                "required": ["prompt"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match self.0.prompt(args.prompt).multi_turn(10).await {
            Ok(response) => {
                println!("[{}] prompt: {response}", Self::NAME);
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("SearchJob error: {0}")]
pub struct SearchJobError(String);

#[tool(description = "
Search job from Jenkins Server
")]
pub async fn search_job(job_name: String) -> Result<String> {
    println!("[{}] Searching job \"search_job\"...", job_name);

    #[derive(Debug, Clone, Deserialize)]
    struct Job {
        _class: String,
        name: String,
    }

    #[derive(Debug, Deserialize)]
    struct Response {
        _class: String,
        jobs: Vec<Job>,
    }

    let client = reqwest::Client::new();

    let res = client
        .get("http://localhost:8080/api/json?tree=jobs[name]")
        .basic_auth("admin", Some("110ae742f68ae5a9d557281f932906a9a1"))
        .send()
        .await
        .expect("Request failed")
        .json::<Response>()
        .await
        .expect("Parse json failed");

    let match_jobs: Vec<Job> = res
        .jobs
        .iter()
        .filter(|job| job.name.contains(&job_name))
        .cloned()
        .collect();

    let first_job = match_jobs
        .first()
        .ok_or(SearchJobError("Job not found".to_string()))?;

    println!("Found job \"{}\"", first_job.name);

    Ok(first_job.name.clone())
}

#[tool(description = "
Build job from Jenkins Server
")]
pub async fn build_job(job_name: String) -> Result<bool> {
    println!("[{}] Building job \"build_job\"...", job_name);

    let client = reqwest::Client::new();

    let _ = client
        .post(format!("http://localhost:8080/job/{}/build", job_name))
        .basic_auth("admin", Some("110ae742f68ae5a9d557281f932906a9a1"))
        .send()
        .await
        .expect("Request failed")
        // .json::<Response>()
        .text()
        .await
        .expect("Parse json failed");

    Ok(true)
}

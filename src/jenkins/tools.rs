use rig::{
    agent::Agent,
    completion::{CompletionModel, Prompt, ToolDefinition},
    tool::{Tool, ToolError},
    tool_macro,
};
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

#[tool_macro(
    description = "Search job from Jenkins Server",
    params(job_name = "Job name"),
    required(job_name)
)]
pub async fn search_job(job_name: String) -> Result<String, ToolError> {
    println!("[search_job] Searching job \"{}\"...", job_name);

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
        .or(Err(ToolError::ToolCallError("Request failed".into())))?
        .json::<Response>()
        .await
        .or(Err(ToolError::ToolCallError("Parse json failed".into())))?;

    let match_jobs: Vec<Job> = res
        .jobs
        .iter()
        .filter(|job| job.name.contains(&job_name))
        .cloned()
        .collect();

    let first_job = match_jobs
        .first()
        .ok_or(ToolError::ToolCallError("Job not found".into()))?;

    println!("Found job \"{}\"", first_job.name);

    Ok(first_job.name.clone())
}

#[tool_macro(description = "Build job from Jenkins Server")]
pub async fn build_job(job_name: String) -> Result<bool, ToolError> {
    println!("[build_job] Building job \"{}\"...", job_name);

    let client = reqwest::Client::new();

    let _ = client
        .post(format!("http://localhost:8080/job/{}/build", job_name))
        .basic_auth("admin", Some("110ae742f68ae5a9d557281f932906a9a1"))
        .send()
        .await
        .or(Err(ToolError::ToolCallError("Request failed".into())))?
        // .json::<Response>()
        .text()
        .await
        .or(Err(ToolError::ToolCallError("Parse json failed".into())))?;

    Ok(true)
}

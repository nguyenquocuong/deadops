use rig::{
    agent::Agent,
    client::completion::CompletionClient,
    completion::{Chat, CompletionModel, PromptError, ToolDefinition},
    integrations::cli_chatbot::ChatBotBuilder,
    providers,
    tool::Tool,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let client = providers::ollama::Client::new();

    let jenkins_agent = client
        .agent("llama3.2")
        .preamble(&format!(
            "You are an assistant here to help the user to perform Jenkins tasks.
            Follow these instructions closely.
            1. Using {} tool to search the job name.
            2. Using {} tool to build the job with the founded job name.
            ",
            SearchJob.name(),
            BuildJob.name()
        ))
        .tool(SearchJob)
        .tool(BuildJob)
        .name("jenkins_agent")
        .build();

    // let jenkins_tool = JenkinsTool(jenkins_agent);
    //
    // let multi_agent_system = client.agent("llama3.2").preamble(
    //     "You are a helpful DevSecOps assistant here to help the user perform daily automation tasks.",
    // )
    //     .tool(jenkins_tool)
    //     .build();

    let chatbot = ChatBotBuilder::new()
        .agent(jenkins_agent)
        .multi_turn_depth(10)
        .build();

    chatbot.run().await?;

    Ok(())
}

// Define a wrapper around an agent so that it can be provided to another agent
// as a tool
struct JenkinsTool<M: CompletionModel>(Agent<M>);

// The input that will be sent to the translator agent from the main agent
#[derive(Deserialize)]
struct JenkinsArgs {
    prompt: String,
}

impl<M: CompletionModel> Tool for JenkinsTool<M> {
    const NAME: &'static str = "jenkins";

    type Args = JenkinsArgs;
    type Error = PromptError;
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
        match self.0.chat(&args.prompt, vec![]).await {
            Ok(response) => {
                println!("[{}] prompt: {response}", Self::NAME);
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct SearchJob;

#[derive(Deserialize)]
struct SearchJobArgs {
    job_name: String,
}

#[derive(Debug, thiserror::Error)]
#[error("SearchJob error: {0}")]
struct SearchJobError(String);

impl Tool for SearchJob {
    const NAME: &'static str = "search_job";

    type Args = SearchJobArgs;
    type Error = SearchJobError;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search job from Jenkins Server".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "job_name": {
                        "type": "string",
                        "description": "The job name need to be search."
                    }
                },
                "required": ["job_name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[{}] Searching job \"{}\"...", Self::NAME, args.job_name);

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
            .basic_auth("admin", Some("112b9374cf35299340c683c64861200e8c"))
            .send()
            .await
            .expect("Request failed")
            .json::<Response>()
            .await
            .expect("Parse json failed");

        let match_jobs: Vec<Job> = res
            .jobs
            .iter()
            .filter(|job| job.name.contains(&args.job_name))
            .cloned()
            .collect();

        let first_job = match_jobs
            .first()
            .ok_or(SearchJobError("Job not found".to_string()))?;

        println!("Found job \"{}\"", first_job.name);

        Ok(first_job.name.clone())
    }
}

#[derive(Deserialize, Serialize)]
struct BuildJob;

#[derive(Deserialize)]
struct BuildJobArgs {
    job_name: String,
}

#[derive(Debug, thiserror::Error)]
#[error("BuildJob error: {0}")]
struct BuildJobError(String);

impl Tool for BuildJob {
    const NAME: &'static str = "build_job";

    type Args = BuildJobArgs;
    type Error = BuildJobError;
    type Output = bool;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Build job from Jenkins Server".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "job_name": {
                        "type": "string",
                        "description": "The job name need to be build."
                    }
                },
                "required": ["job_name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[{}] Building job \"{}\"...", Self::NAME, args.job_name);

        let client = reqwest::Client::new();

        let res = client
            .post(format!("http://localhost:8080/job/{}/build", args.job_name))
            .basic_auth("admin", Some("112b9374cf35299340c683c64861200e8c"))
            .send()
            .await
            .expect("Request failed")
            // .json::<Response>()
            .text()
            .await
            .expect("Parse json failed");

        println!("{:?}", res);

        Ok(true)
    }
}

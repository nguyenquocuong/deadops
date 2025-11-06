use rig::{
    agent::Agent,
    completion::{Chat, CompletionModel, ToolDefinition},
    tool::Tool,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

// Define a wrapper around an agent so that it can be provided to another agent
// as a tool
pub struct JenkinsTool<M: CompletionModel>(pub Agent<M>);

// The input that will be sent to the translator agent from the main agent
#[derive(Deserialize)]
pub struct JenkinsArgs {
    prompt: String,
}

impl<M: CompletionModel> rig::tool::Tool for JenkinsTool<M> {
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
pub struct SearchJob;

#[derive(Deserialize)]
pub struct SearchJobArgs {
    job_name: String,
}

#[derive(Debug, thiserror::Error)]
#[error("SearchJob error: {0}")]
pub struct SearchJobError(String);

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
pub struct BuildJob;

#[derive(Deserialize)]
pub struct BuildJobArgs {
    job_name: String,
}

#[derive(Debug, thiserror::Error)]
#[error("BuildJob error: {0}")]
pub struct BuildJobError(String);

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
            .basic_auth("admin", Some("110ae742f68ae5a9d557281f932906a9a1"))
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

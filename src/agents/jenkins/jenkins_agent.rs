//! Jenkins Agent Implementation
//!
//! This agent interacts with Jenkins CI/CD pipelines to manage builds, jobs, and automation.

use crate::agents::{Agent, AgentMessage, AgentResponse, AgentStatus, MessageType};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JenkinsConfig {
    pub base_url: String,
    pub username: Option<String>,
    pub api_token: Option<String>,
    pub poll_interval: u64,
    pub enabled_jobs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JenkinsJob {
    pub name: String,
    pub url: String,
    pub color: String,
    pub last_build: Option<JenkinsBuild>,
    pub next_build_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JenkinsBuild {
    pub number: u32,
    pub url: String,
    pub result: Option<String>,
    pub timestamp: u64,
    pub duration: u64,
    pub building: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JenkinsPipeline {
    pub name: String,
    pub status: String,
    pub stages: Vec<PipelineStage>,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub name: String,
    pub status: String,
    pub duration: u64,
    pub logs: Option<String>,
}

pub struct JenkinsAgent {
    status: AgentStatus,
    config: JenkinsConfig,
    client: Client,
    jobs: HashMap<String, JenkinsJob>,
    pipelines: HashMap<String, JenkinsPipeline>,
}

impl JenkinsAgent {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = JenkinsConfig {
            base_url: std::env::var("JENKINS_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            username: std::env::var("JENKINS_USERNAME").ok(),
            api_token: std::env::var("JENKINS_API_TOKEN").ok(),
            poll_interval: 30, // 30 seconds
            enabled_jobs: vec![],
        };

        let mut client = Client::new();
        
        // Add basic auth if credentials are provided
        if let (Some(username), Some(token)) = (&config.username, &config.api_token) {
            let auth = format!("{}:{}", username, token);
            let encoded = base64::encode(auth);
            client = client.header("Authorization", format!("Basic {}", encoded));
        }

        Ok(Self {
            status: AgentStatus::Starting,
            config,
            client,
            jobs: HashMap::new(),
            pipelines: HashMap::new(),
        })
    }

    pub async fn fetch_jobs(&mut self) -> Result<Vec<JenkinsJob>, Box<dyn std::error::Error>> {
        let url = format!("{}/api/json", self.config.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch jobs: {}", response.status()).into());
        }

        let json: serde_json::Value = response.json().await?;
        let mut jobs = Vec::new();

        if let Some(jobs_array) = json.get("jobs").and_then(|j| j.as_array()) {
            for job_data in jobs_array {
                if let Ok(job) = serde_json::from_value::<JenkinsJob>(job_data.clone()) {
                    jobs.push(job);
                }
            }
        }

        Ok(jobs)
    }

    pub async fn trigger_build(&self, job_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/job/{}/build", self.config.base_url, job_name);
        let response = self.client.post(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to trigger build for job {}: {}", job_name, response.status()).into());
        }

        Ok(())
    }

    pub async fn get_build_status(&self, job_name: &str, build_number: u32) -> Result<JenkinsBuild, Box<dyn std::error::Error>> {
        let url = format!("{}/job/{}/{}/api/json", self.config.base_url, job_name, build_number);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get build status: {}", response.status()).into());
        }

        let build: JenkinsBuild = response.json().await?;
        Ok(build)
    }

    pub async fn get_job_logs(&self, job_name: &str, build_number: u32) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/job/{}/{}/consoleText", self.config.base_url, job_name, build_number);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get job logs: {}", response.status()).into());
        }

        let logs = response.text().await?;
        Ok(logs)
    }

    pub async fn create_pipeline(&self, name: &str, pipeline_script: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/createItem?name={}", self.config.base_url, name);
        
        let mut form = HashMap::new();
        form.insert("name", name);
        form.insert("mode", "hudson.model.FreeStyleProject");
        
        let response = self.client.post(&url)
            .form(&form)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to create pipeline: {}", response.status()).into());
        }

        Ok(())
    }

    pub async fn monitor_jobs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let jobs = self.fetch_jobs().await?;
        
        for job in jobs {
            self.jobs.insert(job.name.clone(), job);
        }

        // Check for job status changes and trigger alerts
        for (job_name, job) in &self.jobs {
            if let Some(last_build) = &job.last_build {
                if last_build.building {
                    println!("Job {} is currently building (Build #{})", job_name, last_build.number);
                } else if let Some(result) = &last_build.result {
                    match result.as_str() {
                        "SUCCESS" => println!("Job {} completed successfully (Build #{})", job_name, last_build.number),
                        "FAILURE" => println!("Job {} failed (Build #{})", job_name, last_build.number),
                        "UNSTABLE" => println!("Job {} is unstable (Build #{})", job_name, last_build.number),
                        _ => println!("Job {} has unknown result: {} (Build #{})", job_name, result, last_build.number),
                    }
                }
            }
        }

        Ok(())
    }
}

impl Agent for JenkinsAgent {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = AgentStatus::Running;
        println!("Jenkins Agent started and connected to: {}", self.config.base_url);

        // Start monitoring loop
        let mut agent = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(agent.config.poll_interval));
            loop {
                interval.tick().await;
                if let Err(e) = agent.monitor_jobs().await {
                    eprintln!("Jenkins monitoring error: {}", e);
                }
            }
        });

        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = AgentStatus::Stopping;
        println!("Stopping Jenkins Agent...");
        self.status = AgentStatus::Stopped;
        Ok(())
    }

    fn status(&self) -> AgentStatus {
        self.status.clone()
    }

    fn process_message(
        &mut self,
        message: AgentMessage,
    ) -> Result<AgentResponse, Box<dyn std::error::Error>> {
        match message.message_type {
            MessageType::DeploymentRequest => {
                // Handle deployment requests
                if let Some(job_name) = message.payload.get("job_name").and_then(|v| v.as_str()) {
                    let rt = tokio::runtime::Runtime::new()?;
                    rt.block_on(self.trigger_build(job_name))?;
                    
                    Ok(AgentResponse {
                        success: true,
                        message: format!("Build triggered for job: {}", job_name),
                        data: Some(serde_json::json!({"job_name": job_name})),
                        timestamp: chrono::Utc::now(),
                    })
                } else {
                    Ok(AgentResponse {
                        success: false,
                        message: "Missing job_name in deployment request".to_string(),
                        data: None,
                        timestamp: chrono::Utc::now(),
                    })
                }
            }
            MessageType::WorkflowStart => {
                // Handle workflow start requests
                if let Some(pipeline_name) = message.payload.get("pipeline_name").and_then(|v| v.as_str()) {
                    let rt = tokio::runtime::Runtime::new()?;
                    rt.block_on(self.trigger_build(pipeline_name))?;
                    
                    Ok(AgentResponse {
                        success: true,
                        message: format!("Pipeline triggered: {}", pipeline_name),
                        data: Some(serde_json::json!({"pipeline_name": pipeline_name})),
                        timestamp: chrono::Utc::now(),
                    })
                } else {
                    Ok(AgentResponse {
                        success: false,
                        message: "Missing pipeline_name in workflow request".to_string(),
                        data: None,
                        timestamp: chrono::Utc::now(),
                    })
                }
            }
            _ => Ok(AgentResponse {
                success: false,
                message: "Unsupported message type for Jenkins Agent".to_string(),
                data: None,
                timestamp: chrono::Utc::now(),
            }),
        }
    }
}

impl Clone for JenkinsAgent {
    fn clone(&self) -> Self {
        Self {
            status: self.status.clone(),
            config: self.config.clone(),
            client: self.client.clone(),
            jobs: self.jobs.clone(),
            pipelines: self.pipelines.clone(),
        }
    }
}
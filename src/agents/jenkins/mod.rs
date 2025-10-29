//! Jenkins Agent Module
//!
//! This module contains the Jenkins agent that can interact with Jenkins CI/CD pipelines,
//! manage builds, jobs, and provide automation capabilities.

pub mod jenkins_agent;

pub use jenkins_agent::JenkinsAgent;
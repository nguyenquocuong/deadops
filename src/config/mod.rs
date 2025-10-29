//! Configuration management for the DevSecOps system

// pub mod agents;
// pub mod workflows;
// pub mod policies;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub agents: HashMap<String, AgentConfig>,
    pub workflows: Vec<WorkflowConfig>,
    pub policies: Vec<PolicyConfig>,
    pub logging: LoggingConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub enabled: bool,
    pub interval: u64,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub name: String,
    pub triggers: Vec<String>,
    pub steps: Vec<WorkflowStep>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub agent: String,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub name: String,
    pub rules: Vec<PolicyRule>,
    pub enforcement: EnforcementLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub condition: String,
    pub action: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementLevel {
    Advisory,
    Warning,
    Blocking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub health_checks: bool,
    pub alerting: bool,
}

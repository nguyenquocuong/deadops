//! Agent modules for the DevSecOps Multi-Agent System
//!
//! This module contains all specialized agents organized by their functional domains.

pub mod jenkins;
pub mod security;

use serde::{Deserialize, Serialize};

/// Common agent trait that all specialized agents must implement
pub trait Agent: Send + Sync {
    /// Start the agent
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Stop the agent
    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Get agent status
    fn status(&self) -> AgentStatus;

    /// Process a message
    fn process_message(
        &mut self,
        message: AgentMessage,
    ) -> Result<AgentResponse, Box<dyn std::error::Error>>;
}

/// Agent status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

/// Common message structure for inter-agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from: String,
    pub to: String,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Message types for agent communication
#[derive(Eq, Hash, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    // Security messages
    VulnerabilityDetected,
    ThreatAlert,
    AccessRequest,

    // Compliance messages
    PolicyViolation,
    AuditRequest,
    ComplianceCheck,

    // Monitoring messages
    MetricsUpdate,
    LogEvent,
    AlertTriggered,

    // Remediation messages
    IncidentReport,
    PatchRequired,
    RollbackRequest,

    // Orchestration messages
    WorkflowStart,
    DeploymentRequest,
    ResourceAllocation,
}

/// Common response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

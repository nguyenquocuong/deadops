//! System state management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub agents: HashMap<String, AgentState>,
    pub workflows: HashMap<String, WorkflowState>,
    pub incidents: Vec<Incident>,
    pub metrics: SystemMetrics,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub status: String,
    pub last_heartbeat: DateTime<Utc>,
    pub metrics: AgentMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub id: String,
    pub status: String,
    pub current_step: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub severity: String,
    pub description: String,
    pub status: IncidentStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_agents: usize,
    pub active_agents: usize,
    pub total_workflows: usize,
    pub active_workflows: usize,
    pub open_incidents: usize,
    pub system_health: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub messages_processed: u64,
    pub errors: u64,
    pub uptime_seconds: u64,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            agents: HashMap::new(),
            workflows: HashMap::new(),
            incidents: Vec::new(),
            metrics: SystemMetrics {
                total_agents: 0,
                active_agents: 0,
                total_workflows: 0,
                active_workflows: 0,
                open_incidents: 0,
                system_health: 0.0,
            },
            last_updated: Utc::now(),
        }
    }
}

impl SystemState {
    pub fn update_agent_state(&mut self, agent_id: String, state: AgentState) {
        self.agents.insert(agent_id, state);
        self.update_metrics();
        self.last_updated = Utc::now();
    }

    pub fn add_incident(&mut self, incident: Incident) {
        self.incidents.push(incident);
        self.update_metrics();
        self.last_updated = Utc::now();
    }

    fn update_metrics(&mut self) {
        self.metrics.total_agents = self.agents.len();
        self.metrics.active_agents = self
            .agents
            .values()
            .filter(|agent| agent.status == "Running")
            .count();
        self.metrics.total_workflows = self.workflows.len();
        self.metrics.active_workflows = self
            .workflows
            .values()
            .filter(|workflow| workflow.status == "Running")
            .count();
        self.metrics.open_incidents = self
            .incidents
            .iter()
            .filter(|incident| {
                matches!(
                    incident.status,
                    IncidentStatus::Open | IncidentStatus::InProgress
                )
            })
            .count();

        if self.metrics.total_agents > 0 {
            self.metrics.system_health =
                (self.metrics.active_agents as f64) / (self.metrics.total_agents as f64);
        }
    }
}

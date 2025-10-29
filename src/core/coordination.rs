//! System coordination and orchestration

use crate::agents::{Agent, AgentMessage, AgentResponse, AgentStatus};
use crate::core::communication::MessageBus;
use std::collections::HashMap;

pub struct SystemCoordinator {
    agents: HashMap<String, Box<dyn Agent>>,
    message_bus: MessageBus,
    workflows: Vec<Workflow>,
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub steps: Vec<WorkflowStep>,
    pub status: WorkflowStatus,
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub agent_id: String,
    pub action: String,
    pub parameters: HashMap<String, String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl SystemCoordinator {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            agents: HashMap::new(),
            message_bus: MessageBus::new(),
            workflows: Vec::new(),
        })
    }

    pub fn register_agent(&mut self, id: String, agent: Box<dyn Agent>) {
        self.agents.insert(id, agent);
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Start all registered agents
        for (id, agent) in &mut self.agents {
            agent.start()?;
            println!("Started agent: {}", id);
        }

        // Start message processing
        let mut message_bus = self.message_bus.clone();
        tokio::spawn(async move {
            message_bus.process_messages().await;
        });

        Ok(())
    }

    pub fn execute_workflow(
        &mut self,
        workflow: Workflow,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.workflows.push(workflow);
        // Workflow execution logic would go here
        Ok(())
    }

    pub fn get_system_status(&self) -> SystemStatus {
        let mut agent_statuses = HashMap::new();

        for (id, agent) in &self.agents {
            agent_statuses.insert(id.clone(), agent.status());
        }

        SystemStatus {
            agents: agent_statuses,
            active_workflows: self.workflows.len(),
            system_health: self.calculate_system_health(),
        }
    }

    fn calculate_system_health(&self) -> f64 {
        let total_agents = self.agents.len();
        if total_agents == 0 {
            return 0.0;
        }

        let running_agents = self
            .agents
            .values()
            .filter(|agent| matches!(agent.status(), AgentStatus::Running))
            .count();

        (running_agents as f64) / (total_agents as f64)
    }
}

#[derive(Debug)]
pub struct SystemStatus {
    pub agents: HashMap<String, AgentStatus>,
    pub active_workflows: usize,
    pub system_health: f64,
}

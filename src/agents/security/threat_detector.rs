//! Threat Detection Agent
//!
//! Monitors for security threats and suspicious activities

use crate::agents::{Agent, AgentMessage, AgentResponse, AgentStatus, MessageType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    pub id: String,
    pub threat_type: ThreatType,
    pub severity: crate::agents::security::vulnerability_scanner::Severity,
    pub description: String,
    pub source_ip: Option<String>,
    pub target_resource: String,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub status: ThreatStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    BruteForce,
    DDoS,
    Malware,
    Phishing,
    UnauthorizedAccess,
    DataExfiltration,
    InsiderThreat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatStatus {
    Active,
    Mitigated,
    FalsePositive,
    Resolved,
}

pub struct ThreatDetector {
    status: AgentStatus,
    active_threats: HashMap<String, Threat>,
    detection_rules: Vec<DetectionRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub threshold: f64,
    pub enabled: bool,
}

impl ThreatDetector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            status: AgentStatus::Starting,
            active_threats: HashMap::new(),
            detection_rules: vec![
                DetectionRule {
                    id: "brute_force".to_string(),
                    name: "Brute Force Detection".to_string(),
                    pattern: "failed_login".to_string(),
                    threshold: 5.0,
                    enabled: true,
                },
                DetectionRule {
                    id: "ddos".to_string(),
                    name: "DDoS Detection".to_string(),
                    pattern: "high_request_rate".to_string(),
                    threshold: 1000.0,
                    enabled: true,
                },
            ],
        })
    }

    pub fn analyze_logs(
        &mut self,
        logs: Vec<String>,
    ) -> Result<Vec<Threat>, Box<dyn std::error::Error>> {
        let mut threats = Vec::new();

        for log in logs {
            for rule in &self.detection_rules {
                if rule.enabled && self.matches_pattern(&log, &rule.pattern) {
                    let threat = Threat {
                        id: uuid::Uuid::new_v4().to_string(),
                        threat_type: self.get_threat_type(&rule.id),
                        severity: crate::agents::security::vulnerability_scanner::Severity::High,
                        description: format!("Detected pattern: {}", rule.name),
                        source_ip: self.extract_ip(&log),
                        target_resource: "unknown".to_string(),
                        detected_at: chrono::Utc::now(),
                        status: ThreatStatus::Active,
                    };
                    threats.push(threat);
                }
            }
        }

        Ok(threats)
    }

    fn matches_pattern(&self, log: &str, pattern: &str) -> bool {
        log.contains(pattern)
    }

    fn get_threat_type(&self, rule_id: &str) -> ThreatType {
        match rule_id {
            "brute_force" => ThreatType::BruteForce,
            "ddos" => ThreatType::DDoS,
            _ => ThreatType::UnauthorizedAccess,
        }
    }

    fn extract_ip(&self, log: &str) -> Option<String> {
        // Simple IP extraction - in real implementation, use proper regex
        if let Some(ip_start) = log.find("IP:") {
            let ip_part = &log[ip_start + 3..];
            if let Some(space) = ip_part.find(' ') {
                Some(ip_part[..space].to_string())
            } else {
                Some(ip_part.to_string())
            }
        } else {
            None
        }
    }
}

impl Agent for ThreatDetector {
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = AgentStatus::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = AgentStatus::Stopping;
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
            MessageType::ThreatAlert => Ok(AgentResponse {
                success: true,
                message: "Threat analysis completed".to_string(),
                data: None,
                timestamp: chrono::Utc::now(),
            }),
            _ => Ok(AgentResponse {
                success: false,
                message: "Unsupported message type".to_string(),
                data: None,
                timestamp: chrono::Utc::now(),
            }),
        }
    }
}

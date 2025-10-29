//! Inter-agent communication system using message bus

use crate::agents::{AgentMessage, AgentResponse, MessageType};
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct MessageBus {
    subscribers: HashMap<MessageType, Vec<mpsc::UnboundedSender<AgentMessage>>>,
    message_tx: mpsc::UnboundedSender<AgentMessage>,
    message_rx: mpsc::UnboundedReceiver<AgentMessage>,
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBus {
    pub fn new() -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        Self {
            subscribers: HashMap::new(),
            message_tx,
            message_rx,
        }
    }

    pub fn subscribe(
        &mut self,
        message_type: MessageType,
        sender: mpsc::UnboundedSender<AgentMessage>,
    ) {
        self.subscribers
            .entry(message_type)
            .or_default()
            .push(sender);
    }

    pub fn publish(&self, message: AgentMessage) -> Result<(), Box<dyn std::error::Error>> {
        self.message_tx.send(message)?;
        Ok(())
    }

    pub async fn process_messages(&mut self) {
        while let Some(message) = self.message_rx.recv().await {
            if let Some(subscribers) = self.subscribers.get(&message.message_type) {
                for subscriber in subscribers {
                    let _ = subscriber.send(message.clone());
                }
            }
        }
    }
}

impl Clone for MessageBus {
    fn clone(&self) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        Self {
            subscribers: self.subscribers.clone(),
            message_tx,
            message_rx,
        }
    }
}

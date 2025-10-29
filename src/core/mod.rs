//! Core system modules for communication, coordination, and state management

pub mod communication;
pub mod coordination;
pub mod state;

pub use communication::MessageBus;
pub use coordination::SystemCoordinator;
pub use state::SystemState;

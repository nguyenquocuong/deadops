//! DevSecOps Multi-Agent System - Main Entry Point
//! Currently focused on Jenkins CI/CD integration

use deadops::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Starting DevSecOps Multi-Agent System...");

    // Run the multi-agent system
    run().await?;

    println!("DevSecOps Multi-Agent System started successfully!");

    // Keep the main thread alive
    tokio::signal::ctrl_c().await?;

    println!("Shutting down DevSecOps Multi-Agent System...");
    Ok(())
}

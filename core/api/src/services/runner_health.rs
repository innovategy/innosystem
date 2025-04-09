use std::sync::Arc;
use uuid::Uuid;
use anyhow::{Result, Context};
use chrono::{Utc, Duration};
use tracing::{info, error};

use innosystem_common::models::runner::RunnerStatus;
use innosystem_common::models::job::JobStatus;
use innosystem_common::repositories::{JobRepository, JobTypeRepository, RunnerRepository};

/// Defines the health status of a runner
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerHealthStatus {
    /// Runner is healthy and responding to heartbeats
    Healthy,
    /// Runner is alive but has warning signs (e.g., slow response)
    Warning,
    /// Runner is not responding or has critical issues
    Critical,
    /// Runner status cannot be determined
    Unknown,
}

impl RunnerHealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RunnerHealthStatus::Healthy => "healthy",
            RunnerHealthStatus::Warning => "warning",
            RunnerHealthStatus::Critical => "critical",
            RunnerHealthStatus::Unknown => "unknown",
        }
    }
}

/// Configuration for the runner health service
#[derive(Debug, Clone)]
pub struct RunnerHealthConfig {
    /// Maximum duration between heartbeats (in seconds) for a runner to be considered healthy
    pub healthy_heartbeat_interval_secs: i64,
    /// Maximum duration between heartbeats (in seconds) for a runner to be considered in warning state
    pub warning_heartbeat_interval_secs: i64,
}

impl Default for RunnerHealthConfig {
    fn default() -> Self {
        Self {
            healthy_heartbeat_interval_secs: 60,  // 1 minute
            warning_heartbeat_interval_secs: 180, // 3 minutes
        }
    }
}

/// Service for monitoring runner health and compatibility
pub struct RunnerHealthService {
    job_repo: Arc<dyn JobRepository>,
    job_type_repo: Arc<dyn JobTypeRepository>,
    runner_repo: Arc<dyn RunnerRepository>,
    config: RunnerHealthConfig,
}

impl RunnerHealthService {
    /// Create a new RunnerHealthService
    pub fn new(
        job_repo: Arc<dyn JobRepository>,
        job_type_repo: Arc<dyn JobTypeRepository>,
        runner_repo: Arc<dyn RunnerRepository>,
        config: Option<RunnerHealthConfig>,
    ) -> Self {
        Self {
            job_repo,
            job_type_repo,
            runner_repo,
            config: config.unwrap_or_default(),
        }
    }
    
    /// Check if a runner is healthy
    pub async fn check_runner_health(&self, runner_id: Uuid) -> Result<RunnerHealthStatus> {
        // Get the runner
        let runner = self.runner_repo.find_by_id(runner_id)
            .await
            .context("Failed to find runner for health check")?;
        
        // If the runner is marked as inactive or maintenance, return unknown
        if runner.status != RunnerStatus::Active {
            return Ok(RunnerHealthStatus::Unknown);
        }
        
        // If no heartbeat, return critical
        let last_heartbeat = match runner.last_heartbeat {
            Some(heartbeat) => heartbeat,
            None => return Ok(RunnerHealthStatus::Critical),
        };
        
        // Calculate the duration since the last heartbeat
        let now = Utc::now().naive_utc();
        let duration = now.signed_duration_since(last_heartbeat);
        
        // Check against thresholds
        if duration.num_seconds() <= self.config.healthy_heartbeat_interval_secs {
            Ok(RunnerHealthStatus::Healthy)
        } else if duration.num_seconds() <= self.config.warning_heartbeat_interval_secs {
            Ok(RunnerHealthStatus::Warning)
        } else {
            Ok(RunnerHealthStatus::Critical)
        }
    }
    
    /// Check runner compatibility with a job type
    pub async fn is_compatible_with_job_type(&self, runner_id: Uuid, job_type_id: Uuid) -> Result<bool> {
        // Get the runner
        let runner = self.runner_repo.find_by_id(runner_id)
            .await
            .context("Failed to find runner for compatibility check")?;
        
        // Get the job type
        let job_type = self.job_type_repo.find_by_id(job_type_id)
            .await
            .context("Failed to find job type for compatibility check")?;
        
        // Check if the job type name is in the runner's compatible job types
        let is_compatible = runner.compatible_job_types.contains(&job_type.name);
        
        Ok(is_compatible)
    }
    
    /// Find compatible runners for a job type, sorted by health status
    pub async fn find_compatible_runners(&self, job_type_id: Uuid) -> Result<Vec<(Uuid, RunnerHealthStatus)>> {
        // Get the job type
        let job_type = self.job_type_repo.find_by_id(job_type_id)
            .await
            .context("Failed to find job type")?;
        
        // Get all active runners
        let since = (Utc::now() - Duration::minutes(5)).naive_utc();
        let runners = self.runner_repo.list_active(since)
            .await
            .context("Failed to list active runners")?;
        
        // Filter runners that are compatible with the job type
        let mut compatible_runners = Vec::new();
        for runner in runners {
            if runner.compatible_job_types.contains(&job_type.name) {
                // Check the health status
                let health_status = self.check_runner_health(runner.id).await?;
                compatible_runners.push((runner.id, health_status));
            }
        }
        
        // Sort by health status (Healthy > Warning > Critical > Unknown)
        compatible_runners.sort_by(|a, b| {
            let order_a = match a.1 {
                RunnerHealthStatus::Healthy => 0,
                RunnerHealthStatus::Warning => 1,
                RunnerHealthStatus::Critical => 2,
                RunnerHealthStatus::Unknown => 3,
            };
            
            let order_b = match b.1 {
                RunnerHealthStatus::Healthy => 0,
                RunnerHealthStatus::Warning => 1,
                RunnerHealthStatus::Critical => 2,
                RunnerHealthStatus::Unknown => 3,
            };
            
            order_a.cmp(&order_b)
        });
        
        Ok(compatible_runners)
    }
    
    /// Update runner status based on health status
    pub async fn update_status_based_on_health(&self, runner_id: Uuid) -> Result<()> {
        // Check the health status
        let health_status = self.check_runner_health(runner_id).await?;
        
        // Only update if health status is Critical
        if health_status == RunnerHealthStatus::Critical {
            // Get the runner
            let runner = self.runner_repo.find_by_id(runner_id)
                .await
                .context("Failed to find runner for status update")?;
            
            // If the runner is active, set it to inactive
            if runner.status == RunnerStatus::Active {
                info!("Setting runner {} to inactive due to critical health status", runner_id);
                self.runner_repo.set_status(runner_id, false).await?;
            }
        }
        
        Ok(())
    }
    
    /// Check for jobs assigned to unhealthy runners and reassign them
    pub async fn check_and_reassign_jobs(&self) -> Result<u32> {
        // Get all running jobs
        // We'll skip fetching running jobs directly since we're using stalled_jobs instead
        
        let mut reassigned_count = 0;
        
        // We don't need to use in_progress_jobs here, so we'll remove that variable
        // and focus on stalled jobs that need to be reset
        
        // Get jobs that have been in running state too long (stalled)
        let stalled_jobs = self.job_repo.find_stalled_jobs(30) // 30 minutes threshold
            .await
            .context("Failed to find stalled jobs")?;
        
        for job in stalled_jobs {
            // Reset stalled job to pending status
            match self.job_repo.update_status(job.id, JobStatus::Pending).await {
                Ok(_) => {
                    info!("Reset stalled job {} to pending status for reassignment", job.id);
                    reassigned_count += 1;
                },
                Err(e) => {
                    error!("Failed to reset job {} to pending status: {}", job.id, e);
                }
            }
        }
        
        Ok(reassigned_count)
    }
}

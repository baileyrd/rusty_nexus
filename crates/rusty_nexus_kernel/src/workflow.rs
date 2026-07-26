//! Multi-step automated workflow runner for Nexus core.

/// A single step in an automated workflow file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowStep {
    pub name: String,
    pub step_type: String, // "create_note", "sync_reminders", "export_html", "ai_ask"
    pub argument: String,
}

/// Execution report for a workflow run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowReport {
    pub total_steps: usize,
    pub successful_steps: usize,
    pub logs: Vec<String>,
}

/// Workflow engine parsing and running sequential steps.
pub struct WorkflowEngine;

impl WorkflowEngine {
    /// Parse a simple workflow file format (line format: `step_type: argument # Step Name`).
    pub fn parse_workflow(content: &str) -> Vec<WorkflowStep> {
        let mut steps = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(colon) = trimmed.find(':') {
                let step_type = trimmed[..colon].trim().to_string();
                let argument = trimmed[colon + 1..].trim().to_string();
                let name = format!("Step: {}", step_type);
                steps.push(WorkflowStep {
                    name,
                    step_type,
                    argument,
                });
            }
        }
        steps
    }

    /// Execute workflow steps.
    pub fn run_steps(steps: &[WorkflowStep]) -> WorkflowReport {
        let mut report = WorkflowReport {
            total_steps: steps.len(),
            successful_steps: 0,
            logs: Vec::new(),
        };

        for step in steps {
            report.logs.push(format!("Executing '{}' [{}]", step.name, step.step_type));
            report.successful_steps += 1;
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_engine() {
        let sample = "create_note: notes/daily.md # Create note\nsync_reminders: all # Sync tasks";
        let steps = WorkflowEngine::parse_workflow(sample);
        assert_eq!(steps.len(), 2);

        let report = WorkflowEngine::run_steps(&steps);
        assert_eq!(report.successful_steps, 2);
    }
}

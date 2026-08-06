//! Background shell jobs + collect-by-id (Wave C 0.13.0 / spec 50 §1.4).

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct JobSnapshot {
    pub job_id: String,
    pub command: String,
    pub done: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

struct JobInner {
    command: String,
    done: bool,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Process-local background job table.
#[derive(Clone, Default)]
pub struct BgJobStore {
    inner: Arc<Mutex<HashMap<String, JobInner>>>,
}

impl BgJobStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn `sh -c command` in workspace; return job_id immediately.
    pub fn spawn(&self, workspace: &std::path::Path, command: &str) -> Result<String, String> {
        let child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(workspace)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let id = format!("job_{}", Ulid::new());
        {
            let mut map = self.inner.lock().map_err(|e| e.to_string())?;
            map.insert(
                id.clone(),
                JobInner {
                    command: command.to_string(),
                    done: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                },
            );
        }
        let store = self.clone();
        let job_id = id.clone();
        thread::spawn(move || {
            let out = match child.wait_with_output() {
                Ok(o) => o,
                Err(e) => {
                    if let Ok(mut map) = store.inner.lock() {
                        if let Some(job) = map.get_mut(&job_id) {
                            job.done = true;
                            job.stderr = e.to_string();
                        }
                    }
                    return;
                }
            };
            if let Ok(mut map) = store.inner.lock() {
                if let Some(job) = map.get_mut(&job_id) {
                    job.done = true;
                    job.exit_code = out.status.code();
                    job.stdout = String::from_utf8_lossy(&out.stdout).into_owned();
                    job.stderr = String::from_utf8_lossy(&out.stderr).into_owned();
                }
            }
        });
        Ok(id)
    }

    pub fn collect(&self, job_id: &str, wait_ms: u64) -> Result<JobSnapshot, String> {
        let deadline = Instant::now() + Duration::from_millis(wait_ms);
        loop {
            {
                let map = self.inner.lock().map_err(|e| e.to_string())?;
                let job = map
                    .get(job_id)
                    .ok_or_else(|| format!("unknown job_id {job_id}"))?;
                if job.done || wait_ms == 0 {
                    return Ok(JobSnapshot {
                        job_id: job_id.to_string(),
                        command: job.command.clone(),
                        done: job.done,
                        exit_code: job.exit_code,
                        stdout: job.stdout.clone(),
                        stderr: job.stderr.clone(),
                    });
                }
            }
            if Instant::now() >= deadline {
                let map = self.inner.lock().map_err(|e| e.to_string())?;
                let job = map
                    .get(job_id)
                    .ok_or_else(|| format!("unknown job_id {job_id}"))?;
                return Ok(JobSnapshot {
                    job_id: job_id.to_string(),
                    command: job.command.clone(),
                    done: job.done,
                    exit_code: job.exit_code,
                    stdout: job.stdout.clone(),
                    stderr: job.stderr.clone(),
                });
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn snapshot_json(&self, job_id: &str, wait_ms: u64) -> Result<Value, String> {
        let s = self.collect(job_id, wait_ms)?;
        Ok(json!({
            "job_id": s.job_id,
            "command": s.command,
            "done": s.done,
            "exit_code": s.exit_code,
            "stdout": s.stdout,
            "stderr": s.stderr,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn spawn_and_collect_echo() {
        let dir = tempdir().unwrap();
        let store = BgJobStore::new();
        let id = store.spawn(dir.path(), "echo bg-ok").unwrap();
        assert!(id.starts_with("job_"));
        let snap = store.collect(&id, 3000).unwrap();
        assert!(snap.done, "job should finish");
        assert!(snap.stdout.contains("bg-ok"), "stdout={:?}", snap.stdout);
    }
}

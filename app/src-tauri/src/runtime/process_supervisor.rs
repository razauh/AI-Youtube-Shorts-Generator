use std::fmt;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ProcessSpec {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub stdin_bytes: Vec<u8>,
    pub timeout_ms: u64,
}

impl fmt::Debug for ProcessSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcessSpec")
            .field("program", &self.program)
            .field("args", &self.args)
            .field(
                "env_keys",
                &self.env.iter().map(|(key, _)| key).collect::<Vec<_>>(),
            )
            .field("stdin_bytes_len", &self.stdin_bytes.len())
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ProcessOutput {
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

#[derive(Debug, Clone)]
pub enum ProcessError {
    Spawn(String),
    Wait(String),
}

fn read_pipe(
    mut pipe: impl Read + Send + 'static,
    sink: Arc<Mutex<Vec<u8>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut local = Vec::new();
        let _ = pipe.read_to_end(&mut local);
        if let Ok(mut g) = sink.lock() {
            *g = local;
        }
    })
}

pub fn run_supervised(spec: ProcessSpec) -> Result<ProcessOutput, ProcessError> {
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (key, value) in &spec.env {
        command.env(key, value);
    }

    let mut child = command
        .spawn()
        .map_err(|e| ProcessError::Spawn(e.to_string()))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(&spec.stdin_bytes);
        let _ = stdin.flush();
    }

    let stdout_buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    let stderr_buf = Arc::new(Mutex::new(Vec::<u8>::new()));

    let out_handle = child
        .stdout
        .take()
        .map(|p| read_pipe(p, Arc::clone(&stdout_buf)));
    let err_handle = child
        .stderr
        .take()
        .map(|p| read_pipe(p, Arc::clone(&stderr_buf)));

    let deadline = Instant::now() + Duration::from_millis(spec.timeout_ms);
    let mut timed_out = false;

    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => {
                if Instant::now() >= deadline {
                    timed_out = true;
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(ProcessError::Wait(e.to_string())),
        }
    }

    if let Some(h) = out_handle {
        let _ = h.join();
    }
    if let Some(h) = err_handle {
        let _ = h.join();
    }

    let status_code = child
        .try_wait()
        .map_err(|e| ProcessError::Wait(e.to_string()))?
        .and_then(|s| s.code());

    let stdout = String::from_utf8_lossy(&stdout_buf.lock().map(|g| g.clone()).unwrap_or_default())
        .to_string();
    let stderr = String::from_utf8_lossy(&stderr_buf.lock().map(|g| g.clone()).unwrap_or_default())
        .to_string();

    Ok(ProcessOutput {
        status_code,
        stdout,
        stderr,
        timed_out,
    })
}

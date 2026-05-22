use crate::runtime::process_supervisor::{
    run_supervised, ProcessError, ProcessOutput, ProcessSpec,
};
use std::fmt;

#[derive(Clone)]
pub struct PythonInvokeRequest {
    pub python_bin: String,
    pub entry_script: String,
    pub env: Vec<(String, String)>,
    pub stdin_json: Vec<u8>,
    pub timeout_ms: u64,
}

impl fmt::Debug for PythonInvokeRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PythonInvokeRequest")
            .field("python_bin", &self.python_bin)
            .field("entry_script", &self.entry_script)
            .field(
                "env_keys",
                &self.env.iter().map(|(key, _)| key).collect::<Vec<_>>(),
            )
            .field("stdin_json_len", &self.stdin_json.len())
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

pub fn invoke_python(req: PythonInvokeRequest) -> Result<ProcessOutput, ProcessError> {
    run_supervised(ProcessSpec {
        program: req.python_bin,
        args: vec![req.entry_script],
        env: req.env,
        stdin_bytes: req.stdin_json,
        timeout_ms: req.timeout_ms,
    })
}

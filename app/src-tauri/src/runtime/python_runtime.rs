use crate::runtime::process_supervisor::{
    run_supervised, ProcessError, ProcessOutput, ProcessSpec,
};

#[derive(Debug, Clone)]
pub struct PythonInvokeRequest {
    pub python_bin: String,
    pub entry_script: String,
    pub stdin_json: Vec<u8>,
    pub timeout_ms: u64,
}

pub fn invoke_python(req: PythonInvokeRequest) -> Result<ProcessOutput, ProcessError> {
    run_supervised(ProcessSpec {
        program: req.python_bin,
        args: vec![req.entry_script],
        stdin_bytes: req.stdin_json,
        timeout_ms: req.timeout_ms,
    })
}

use serde_json::{json, Value};
use shorts_tauri_app::core::highlights::{
    chunk_transcript, dedupe_highlights, get_highlights, parse_json_loose, HighlightLlm,
    CHUNK_OVERLAP_SECONDS, CHUNK_SIZE_SECONDS, CONTENT_TYPE_PROMPT, GPT_CALL_TIMEOUT_SECONDS,
    HIGHLIGHT_SYSTEM_PROMPT, LONG_VIDEO_THRESHOLD, VIRALITY_CRITERIA,
};

struct StubLlm {
    calls: std::sync::Mutex<Vec<String>>,
    responses: std::sync::Mutex<Vec<String>>,
}

fn normalize_numbers(v: &Value) -> Value {
    match v {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                json!(f)
            } else {
                v.clone()
            }
        }
        Value::Array(arr) => Value::Array(arr.iter().map(normalize_numbers).collect()),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), normalize_numbers(v)))
                .collect(),
        ),
        _ => v.clone(),
    }
}

impl StubLlm {
    fn new(responses: Vec<String>) -> Self {
        Self {
            calls: std::sync::Mutex::new(Vec::new()),
            responses: std::sync::Mutex::new(responses),
        }
    }

    fn take_calls(&self) -> Vec<String> {
        self.calls.lock().expect("calls lock").clone()
    }
}

impl HighlightLlm for StubLlm {
    fn call(&self, prompt: &str) -> Result<String, String> {
        self.calls
            .lock()
            .expect("calls lock")
            .push(prompt.to_string());
        let mut rs = self.responses.lock().expect("responses lock");
        if rs.is_empty() {
            return Err("no stub response".to_string());
        }
        Ok(rs.remove(0))
    }
}

#[test]
fn constants_and_prompt_templates_match_python_contract() {
    assert_eq!(CHUNK_SIZE_SECONDS, 1200.0);
    assert_eq!(LONG_VIDEO_THRESHOLD, 1800.0);
    assert_eq!(CHUNK_OVERLAP_SECONDS, 60.0);
    assert_eq!(GPT_CALL_TIMEOUT_SECONDS, 300.0);
    assert!(CONTENT_TYPE_PROMPT.contains("Respond with JSON only"));
    assert!(VIRALITY_CRITERIA.contains("HOOK MOMENTS"));
    assert!(HIGHLIGHT_SYSTEM_PROMPT.contains("You are an elite short-form video editor"));
}

#[test]
fn parse_json_loose_matches_python_vectors() {
    let raw = include_str!("../../../tests/fixtures/highlights/json_fence_vector.json");
    let vectors: Value = serde_json::from_str(raw).expect("fixture json");
    for case in vectors["cases"].as_array().expect("cases arr") {
        let parsed = parse_json_loose(case["raw"].as_str().expect("raw str")).expect("parse");
        assert_eq!(parsed, case["expected"]);
    }
}

#[test]
fn chunk_transcript_matches_python_vectors() {
    let raw = include_str!("../../../tests/fixtures/highlights/chunk_boundaries_vector.json");
    let vectors: Value = serde_json::from_str(raw).expect("fixture json");
    let chunks = chunk_transcript(vectors["input"].clone());
    let got: Vec<Value> = chunks
        .into_iter()
        .map(|c| {
            let segs = c["segments"].as_array().expect("segments");
            json!({
                "offset": c["_offset"],
                "duration": c["duration"],
                "segment_starts": segs.iter().map(|s| s["start"].clone()).collect::<Vec<_>>(),
                "segment_ends": segs.iter().map(|s| s["end"].clone()).collect::<Vec<_>>()
            })
        })
        .collect();
    let expected = vectors["expected"]
        .as_array()
        .expect("expected array")
        .clone();
    assert_eq!(
        normalize_numbers(&Value::Array(got)),
        normalize_numbers(&Value::Array(expected))
    );
}

#[test]
fn dedupe_matches_python_vectors() {
    let raw = include_str!("../../../tests/fixtures/highlights/dedupe_vector.json");
    let vectors: Value = serde_json::from_str(raw).expect("fixture json");
    let deduped = dedupe_highlights(vectors["input"].as_array().expect("input").clone());
    let expected = vectors["expected"]
        .as_array()
        .expect("expected array")
        .clone();
    assert_eq!(deduped, expected);
}

#[test]
fn long_video_chunk_flow_offset_and_dedupe_parity() {
    let transcript = json!({
        "duration": 2500.0,
        "segments": [
            {"start": 0.0, "end": 5.0, "text": "hello"},
            {"start": 1190.0, "end": 1205.0, "text": "near edge"},
            {"start": 1210.0, "end": 1220.0, "text": "second chunk"},
            {"start": 2390.0, "end": 2400.0, "text": "third chunk"}
        ]
    });

    let llm = StubLlm::new(vec![
        "{\"content_type\":\"podcast\",\"density\":\"high\"}".to_string(),
        "{\"highlights\":[{\"title\":\"A\",\"start_time\":10,\"end_time\":50,\"score\":90,\"hook_sentence\":\"h1\",\"virality_reason\":\"r1\"}]}".to_string(),
        "{\"highlights\":[{\"title\":\"B\",\"start_time\":5,\"end_time\":45,\"score\":80,\"hook_sentence\":\"h2\",\"virality_reason\":\"r2\"}]}".to_string(),
        "{\"highlights\":[{\"title\":\"C\",\"start_time\":20,\"end_time\":40,\"score\":70,\"hook_sentence\":\"h3\",\"virality_reason\":\"r3\"}]}".to_string(),
    ]);

    let out = get_highlights(transcript, 3, &llm).expect("highlights");
    let hs = out["highlights"].as_array().expect("highlights arr");
    assert_eq!(hs.len(), 3);
    assert_eq!(hs[0]["title"], "A");
    assert_eq!(hs[0]["start_time"], 10.0);
    assert_eq!(hs[1]["title"], "B");
    assert_eq!(hs[1]["start_time"], 1145.0);
    assert_eq!(hs[2]["title"], "C");
    assert_eq!(hs[2]["start_time"], 2300.0);

    let calls = llm.take_calls();
    assert_eq!(calls.len(), 4);
    assert!(calls[1].contains("Generate at least"));
}

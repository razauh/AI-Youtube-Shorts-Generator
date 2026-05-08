use serde_json::{json, Value};

pub const CONTENT_TYPE_PROMPT: &str = "Analyze this video transcript sample and classify the content type.\nChoose one: podcast, interview, tutorial, lecture, commentary, debate, vlog, other.\nAlso estimate content density: low (mostly filler/chit-chat), medium, or high (dense info/stories).\nRespond with JSON only: {\"content_type\": \"...\", \"density\": \"...\"}";

pub const VIRALITY_CRITERIA: &str = r#"
Virality signals to prioritize (ranked by impact):
1. HOOK MOMENTS — statements that create immediate curiosity ("The secret is...", "Nobody talks about...", "I was completely wrong about...")
2. EMOTIONAL PEAKS — genuine surprise, laughter, anger, vulnerability, excitement; raw unscripted reactions
3. OPINION BOMBS — strong, polarizing or counter-intuitive statements that trigger agree/disagree
4. REVELATION MOMENTS — surprising facts, stats, or confessions that reframe how the viewer thinks
5. CONFLICT/TENSION — disagreement, pushback, or a problem being confronted head-on
6. QUOTABLE ONE-LINERS — a sentence that works as a standalone quote card
7. STORY PEAKS — the climax or twist of an anecdote; the payoff moment
8. PRACTICAL VALUE — a concrete tip, hack, or insight the viewer can immediately apply
"#;

pub const HIGHLIGHT_SYSTEM_PROMPT: &str = "You are an elite short-form video editor who has studied thousands of viral clips on TikTok, Instagram Reels, and YouTube Shorts. You know exactly what makes viewers stop scrolling, watch to the end, and share.\n\n{virality_criteria}\n\nContent type: {content_type} | Density: {density}\n\nYour task: identify the most viral-worthy highlights from the transcript.\n\nRules:\n- Every highlight must open with a strong HOOK — a line that grabs attention within the first 3 seconds\n- Duration sweet spot: 45-90 seconds. Go shorter (20-44s) only for a perfect standalone one-liner. Go longer (91-180s) only when a story arc needs full context to land\n- Never cut mid-sentence or mid-thought — each clip must feel complete and self-contained\n- Clips must not overlap significantly with each other\n- Score 0-100 on viral potential (not general quality)\n- {num_clips_instruction}\n- For each highlight, identify the single best \"hook_sentence\" — the opening line that would make someone stop scrolling\n- Explain in one sentence why this clip is viral (\"virality_reason\")\n\nRespond ONLY with valid JSON (no markdown, no explanation):\n{{\"highlights\":[{{\"title\":\"string\",\"start_time\":float,\"end_time\":float,\"score\":int,\"hook_sentence\":\"string\",\"virality_reason\":\"string\"}}]}}";

pub const CHUNK_SIZE_SECONDS: f64 = 1200.0;
pub const LONG_VIDEO_THRESHOLD: f64 = 1800.0;
pub const CHUNK_OVERLAP_SECONDS: f64 = 60.0;
pub const GPT_CALL_TIMEOUT_SECONDS: f64 = 300.0;

pub trait HighlightLlm {
    fn call(&self, prompt: &str) -> Result<String, String>;
}

pub fn parse_json_loose(raw: &str) -> Result<Value, serde_json::Error> {
    let mut text = raw.trim().to_string();
    if text.starts_with("```") {
        if text.starts_with("```json") {
            text = text[7..].trim_start().to_string();
        } else {
            text = text[3..].trim_start().to_string();
        }
    }
    if text.ends_with("```") {
        text = text[..text.len() - 3].trim_end().to_string();
    }

    match serde_json::from_str::<Value>(&text) {
        Ok(v) => Ok(v),
        Err(_) => {
            let start = text.find('{');
            let end = text.rfind('}');
            if let (Some(s), Some(e)) = (start, end) {
                serde_json::from_str::<Value>(&text[s..=e])
            } else {
                serde_json::from_str::<Value>(&text)
            }
        }
    }
}

pub fn detect_content_type(transcript: &Value, llm: &dyn HighlightLlm) -> Value {
    let sample = transcript
        .get("segments")
        .and_then(Value::as_array)
        .map(|segs| {
            segs.iter()
                .take(25)
                .filter_map(|s| s.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    let sample: String = sample.chars().take(3000).collect();
    let prompt = format!("{CONTENT_TYPE_PROMPT}\n\nTranscript sample:\n{sample}");

    match llm
        .call(&prompt)
        .ok()
        .and_then(|raw| parse_json_loose(&raw).ok())
    {
        Some(v) => v,
        None => json!({"content_type": "other", "density": "medium"}),
    }
}

pub fn build_transcript_text(transcript: &Value) -> String {
    transcript
        .get("segments")
        .and_then(Value::as_array)
        .map(|segs| {
            segs.iter()
                .map(|s| {
                    let start = s.get("start").and_then(Value::as_f64).unwrap_or(0.0);
                    let text = s
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .trim();
                    format!("[{start:.1}s] {text}")
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

pub fn chunk_transcript(transcript: Value) -> Vec<Value> {
    let segments = transcript
        .get("segments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let duration = transcript
        .get("duration")
        .and_then(Value::as_f64)
        .or_else(|| {
            segments
                .last()
                .and_then(|s| s.get("end"))
                .and_then(Value::as_f64)
        })
        .unwrap_or(0.0);

    let mut chunks = Vec::new();
    let mut start = 0.0;
    while start < duration {
        let end = (start + CHUNK_SIZE_SECONDS).min(duration);
        let chunk_segs: Vec<Value> = segments
            .iter()
            .filter(|s| {
                let s_start = s.get("start").and_then(Value::as_f64).unwrap_or(0.0);
                let s_end = s.get("end").and_then(Value::as_f64).unwrap_or(0.0);
                s_start >= start && s_end <= end + CHUNK_OVERLAP_SECONDS
            })
            .cloned()
            .collect();

        if !chunk_segs.is_empty() {
            let mut chunk = transcript.clone();
            chunk["segments"] = Value::Array(chunk_segs);
            chunk["duration"] = json!(end - start);
            chunk["_offset"] = json!(start);
            chunks.push(chunk);
        }
        start += CHUNK_SIZE_SECONDS - CHUNK_OVERLAP_SECONDS;
    }
    chunks
}

fn call_highlight_api(
    transcript_text: &str,
    content_info: &Value,
    duration: f64,
    num_clips: usize,
    is_chunk: bool,
    llm: &dyn HighlightLlm,
) -> Result<Value, String> {
    let target = std::cmp::max(num_clips * 2, 5);
    let natural_max = std::cmp::max(if is_chunk { 2 } else { 3 }, (duration / 90.0) as usize);
    let min_clips = std::cmp::min(std::cmp::min(target, natural_max), 8);

    let system = HIGHLIGHT_SYSTEM_PROMPT
        .replace("{virality_criteria}", VIRALITY_CRITERIA)
        .replace(
            "{content_type}",
            content_info
                .get("content_type")
                .and_then(Value::as_str)
                .unwrap_or("other"),
        )
        .replace(
            "{density}",
            content_info
                .get("density")
                .and_then(Value::as_str)
                .unwrap_or("medium"),
        )
        .replace(
            "{num_clips_instruction}",
            &format!("Generate at least {min_clips} highlights"),
        );

    let full_prompt = format!("{system}\n\nTranscript:\n{transcript_text}");
    let raw = llm.call(&full_prompt)?;
    parse_json_loose(&raw).map_err(|e| e.to_string())
}

pub fn dedupe_highlights(mut highlights: Vec<Value>) -> Vec<Value> {
    highlights.sort_by(|a, b| {
        let sa = a.get("score").and_then(Value::as_i64).unwrap_or(0);
        let sb = b.get("score").and_then(Value::as_i64).unwrap_or(0);
        sb.cmp(&sa)
    });

    let mut kept: Vec<Value> = Vec::new();
    for h in highlights {
        let h_start = h.get("start_time").and_then(Value::as_f64).unwrap_or(0.0);
        let h_end = h.get("end_time").and_then(Value::as_f64).unwrap_or(0.0);
        let h_dur = h_end - h_start;

        let mut overlapping = false;
        for k in &kept {
            let k_start = k.get("start_time").and_then(Value::as_f64).unwrap_or(0.0);
            let k_end = k.get("end_time").and_then(Value::as_f64).unwrap_or(0.0);
            let latest_start = h_start.max(k_start);
            let earliest_end = h_end.min(k_end);
            let overlap = earliest_end - latest_start;
            if overlap > 0.0 && overlap > 0.5 * h_dur {
                overlapping = true;
                break;
            }
        }

        if !overlapping {
            kept.push(h);
        }
    }
    kept
}

pub fn get_highlights(
    transcript: Value,
    num_clips: usize,
    llm: &dyn HighlightLlm,
) -> Result<Value, String> {
    let duration = transcript
        .get("duration")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let content_info = detect_content_type(&transcript, llm);

    let highlights = if duration >= LONG_VIDEO_THRESHOLD {
        let chunks = chunk_transcript(transcript.clone());
        let mut all = Vec::new();
        for chunk in chunks {
            let offset = chunk.get("_offset").and_then(Value::as_f64).unwrap_or(0.0);
            let text = build_transcript_text(&chunk);
            let chunk_duration = chunk.get("duration").and_then(Value::as_f64).unwrap_or(0.0);
            let result =
                call_highlight_api(&text, &content_info, chunk_duration, num_clips, true, llm)?;
            if let Some(arr) = result.get("highlights").and_then(Value::as_array) {
                for h in arr {
                    let mut m = h.clone();
                    let st = m.get("start_time").and_then(Value::as_f64).unwrap_or(0.0) + offset;
                    let et = m.get("end_time").and_then(Value::as_f64).unwrap_or(0.0) + offset;
                    m["start_time"] = json!(st);
                    m["end_time"] = json!(et);
                    all.push(m);
                }
            }
        }
        dedupe_highlights(all)
    } else {
        let text = build_transcript_text(&transcript);
        let result = call_highlight_api(&text, &content_info, duration, num_clips, false, llm)?;
        dedupe_highlights(
            result
                .get("highlights")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
        )
    };

    Ok(json!({ "highlights": highlights }))
}

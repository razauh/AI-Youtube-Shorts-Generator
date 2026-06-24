use serde_json::json;
use shorts_tauri_app::core::pipeline::{
    generate_shorts_with, GenerateShortsRequest, MockPipelineStages,
};

fn base_request(mode: &str) -> GenerateShortsRequest {
    GenerateShortsRequest {
        youtube_url: "https://youtube.com/watch?v=abc".to_string(),
        num_clips: 1,
        aspect_ratio: "9:16".to_string(),
        download_format: "720".to_string(),
        language: None,
        mode: mode.to_string(),
    }
}

#[test]
fn mode_invalid_returns_parity_error() {
    let req = base_request("weird");
    let mut stages = MockPipelineStages::default();

    let err = generate_shorts_with(&req, &mut stages).expect_err("invalid mode must fail");
    assert_eq!(err.error, "Unknown mode: 'weird'. Use 'api'.");
    assert_eq!(err.mode, None);
    assert_eq!(err.source_video_url, None);
}

#[test]
fn mode_api_success_matches_golden_and_stage_order() {
    let req = base_request("api");
    let mut stages = MockPipelineStages::default()
        .with_download(Ok("https://cdn.example.com/video.mp4".to_string()))
        .with_transcript(Ok(json!({
            "duration": 140.0,
            "segments": [
                {"start": 0.0, "end": 4.0, "text": "Most creators miss this."},
                {"start": 4.0, "end": 9.2, "text": "One change doubled retention."}
            ]
        })))
        .with_highlights(Ok(json!({
            "highlights": [
                {
                    "title": "Retention Secret",
                    "start_time": 0.0,
                    "end_time": 54.4,
                    "score": 92,
                    "hook_sentence": "Most creators miss this.",
                    "virality_reason": "Strong hook plus concrete outcome creates immediate curiosity."
                },
                {
                    "title": "One Change",
                    "start_time": 60.0,
                    "end_time": 110.0,
                    "score": 81,
                    "hook_sentence": "One change doubled retention.",
                    "virality_reason": "Clear payoff and actionable tip."
                }
            ]
        })))
        .with_crop(Ok(vec![json!({
            "title": "Retention Secret",
            "start_time": 0.0,
            "end_time": 54.4,
            "score": 92,
            "hook_sentence": "Most creators miss this.",
            "virality_reason": "Strong hook plus concrete outcome creates immediate curiosity.",
            "clip_url": "https://cdn.example.com/short_01.mp4"
        })]));

    let out = generate_shorts_with(&req, &mut stages).expect("api success");

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/golden/v1/api_success.json"
    ))
    .expect("golden fixture");
    let got = serde_json::to_value(out).expect("serialize output");
    assert_eq!(got, expected);

    assert_eq!(
        stages.call_log(),
        vec!["download", "transcribe", "highlights", "crop"]
    );
}

#[test]
fn mode_api_no_segments_maps_hard_failure_envelope() {
    let req = base_request("api");
    let mut stages = MockPipelineStages::default()
        .with_download(Ok("https://cdn.example.com/video.mp4".to_string()))
        .with_transcript(Ok(json!({"duration": 0.0, "segments": []})));

    let err = generate_shorts_with(&req, &mut stages).expect_err("no segments must hard fail");

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/golden/v1/no_segments.json"
    ))
    .expect("golden fixture");
    let got = serde_json::to_value(err).expect("serialize envelope");
    assert_eq!(got, expected);

    assert_eq!(stages.call_log(), vec!["download", "transcribe"]);
}

#[test]
fn mode_api_clip_failure_matches_partial_success_golden() {
    let req = base_request("api");
    let mut stages = MockPipelineStages::default()
        .with_download(Ok("https://cdn.example.com/video.mp4".to_string()))
        .with_transcript(Ok(json!({
            "duration": 140.0,
            "segments": [
                {"start": 0.0, "end": 4.0, "text": "Most creators miss this."},
                {"start": 4.0, "end": 9.2, "text": "One change doubled retention."}
            ]
        })))
        .with_highlights(Ok(json!({
            "highlights": [
                {
                    "title": "Retention Secret",
                    "start_time": 0.0,
                    "end_time": 54.4,
                    "score": 92,
                    "hook_sentence": "Most creators miss this.",
                    "virality_reason": "Strong hook plus concrete outcome creates immediate curiosity."
                }
            ]
        })))
        .with_crop(Ok(vec![json!({
            "title": "Retention Secret",
            "start_time": 0.0,
            "end_time": 54.4,
            "score": 92,
            "hook_sentence": "Most creators miss this.",
            "virality_reason": "Strong hook plus concrete outcome creates immediate curiosity.",
            "clip_url": null,
            "error": "autocrop upstream timeout"
        })]));

    let out = generate_shorts_with(&req, &mut stages)
        .expect("clip failure still returns success envelope");

    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/golden/v1/clip_failure.json"
    ))
    .expect("golden fixture");
    let got = serde_json::to_value(out).expect("serialize output");
    assert_eq!(got, expected);
}

use serde_json::json;
use shorts_tauri_app::core::api_mode::{clipper, downloader, transcriber};

#[test]
fn extract_video_url_fallback_order_and_shapes() {
    let top = json!({"video_url": "https://cdn/top.mp4"});
    assert_eq!(
        downloader::extract_video_url(&top).expect("top-level video_url"),
        "https://cdn/top.mp4"
    );

    let nested = json!({"outputs": {"output_url": "https://cdn/nested.mp4"}});
    assert_eq!(
        downloader::extract_video_url(&nested).expect("nested output_url"),
        "https://cdn/nested.mp4"
    );

    let list = json!({"output": ["https://cdn/list.mp4"]});
    assert_eq!(
        downloader::extract_video_url(&list).expect("list fallback"),
        "https://cdn/list.mp4"
    );
}

#[test]
fn extract_video_url_malformed_payload_errors() {
    let bad = json!({"result": {"url": "not-http"}});
    let err = downloader::extract_video_url(&bad).expect_err("must fail malformed payload");
    assert!(err.contains("Could not find downloaded video URL in MuAPI response"));
}

#[test]
fn extract_verbose_payload_fallback_shapes() {
    let from_output_dict = json!({"output": {"duration": 7, "segments": []}});
    assert_eq!(
        transcriber::extract_verbose_payload(&from_output_dict)
            .expect("output dict")
            .get("duration")
            .and_then(|v| v.as_i64()),
        Some(7)
    );

    let from_result_string = json!({
        "result": "{\"duration\":3.2,\"segments\":[{\"start\":0,\"end\":1,\"text\":\" hi \"}]}"
    });
    assert!(transcriber::extract_verbose_payload(&from_result_string)
        .expect("result json string")
        .get("segments")
        .and_then(|v| v.as_array())
        .is_some());

    let from_outputs_list = json!({
        "outputs": ["{\"duration\":2,\"segments\":[{\"start\":\"1.5\",\"end\":2,\"text\":\"x\"}]}"]
    });
    assert!(transcriber::extract_verbose_payload(&from_outputs_list)
        .expect("outputs list json")
        .get("segments")
        .and_then(|v| v.as_array())
        .is_some());
}

#[test]
fn extract_verbose_payload_malformed_payload_errors() {
    let bad = json!({"output": "not-json"});
    let err =
        transcriber::extract_verbose_payload(&bad).expect_err("must fail when no segments found");
    assert!(err.contains("Could not find Whisper segments in MuAPI response"));
}

#[test]
fn normalize_transcript_segments_float_and_trim_parity() {
    let verbose = json!({
        "duration": null,
        "segments": [
            {"start": "1.25", "end": 2, "text": "  hello  "},
            {"start": 2.5, "end": "4.0", "text": null}
        ]
    });

    let out = transcriber::normalize_transcript(&verbose).expect("normalized transcript");
    assert_eq!(out.duration, 4.0);
    assert_eq!(out.segments.len(), 2);
    assert_eq!(out.segments[0].start, 1.25);
    assert_eq!(out.segments[0].end, 2.0);
    assert_eq!(out.segments[0].text, "hello");
    assert_eq!(out.segments[1].text, "");
}

#[test]
fn crop_highlights_partial_failure_capture() {
    let highlights = vec![
        json!({"title": "ok", "start_time": 0.0, "end_time": 10.0}),
        json!({"title": "bad", "start_time": 10.0, "end_time": 20.0}),
    ];

    let out = clipper::crop_highlights_with_mapper(highlights, |h| {
        if h.get("title").and_then(|v| v.as_str()) == Some("bad") {
            Err("boom".to_string())
        } else {
            Ok("https://cdn/clip.mp4".to_string())
        }
    });

    assert_eq!(out.len(), 2);
    assert_eq!(
        out[0].get("clip_url").and_then(|v| v.as_str()),
        Some("https://cdn/clip.mp4")
    );
    assert_eq!(out[0].get("error"), None);

    assert!(out[1].get("clip_url").is_some());
    assert!(out[1].get("clip_url").unwrap().is_null());
    assert_eq!(out[1].get("error").and_then(|v| v.as_str()), Some("boom"));
}

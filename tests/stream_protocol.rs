use rs_jarvis::client::parse_stream_line;

#[test]
fn parses_text_delta_stream_events() {
    let event =
        parse_stream_line("data: {\"type\":\"response.output_text.delta\",\"delta\":\"hello\"}\r")
            .unwrap()
            .unwrap();

    assert_eq!(event.kind, "response.output_text.delta");
    assert_eq!(event.delta.as_deref(), Some("hello"));
}

#[test]
fn ignores_done_markers_and_non_data_lines() {
    assert!(parse_stream_line("data: [DONE]").unwrap().is_none());
    assert!(
        parse_stream_line("event: response.completed")
            .unwrap()
            .is_none()
    );
    assert!(parse_stream_line("").unwrap().is_none());
}

#[test]
fn rejects_malformed_stream_events() {
    let error = parse_stream_line("data: not-json").unwrap_err();

    assert!(error.to_string().contains("invalid streaming event"));
}

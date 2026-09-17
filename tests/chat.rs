use rs_jarvis::chat::{Input, parse_input};

#[test]
fn parses_supported_commands_after_trimming_whitespace() {
    assert_eq!(parse_input("  /clear\r\n"), Input::Clear);
    assert_eq!(parse_input("/hide\n"), Input::Hide);
    assert_eq!(parse_input(" /exit "), Input::Exit);
    assert_eq!(parse_input("/quit"), Input::Exit);
}

#[test]
fn preserves_regular_messages_and_ignores_blank_input() {
    assert_eq!(
        parse_input("  hello Jarvis  \n"),
        Input::Message("hello Jarvis")
    );
    assert_eq!(parse_input(" \r\n"), Input::Empty);
}

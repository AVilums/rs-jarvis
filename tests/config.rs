use rs_jarvis::config::{Config, Connector, Profile};

const BASE_CONFIG: &str = r#"
    default = "openai"

    [profiles.openai]
    connector = "openai"
    model = "gpt-5.2"
    base_url = "https://api.openai.com/v1"
    api_key_env = "OPENAI_API_KEY"
"#;

#[test]
fn local_configuration_overrides_only_the_selected_values() {
    let local = r#"
        [profiles.openai]
        model = "local-model"
        base_url = "http://localhost:1234/v1"
    "#;

    let config = Config::from_sources(BASE_CONFIG, Some(local)).unwrap();
    let profile = config.selected_profile();

    assert_eq!(profile.model, "local-model");
    assert_eq!(profile.base_url, "http://localhost:1234/v1");
    assert_eq!(profile.api_key_env.as_deref(), Some("OPENAI_API_KEY"));
    assert!(matches!(profile.connector, Connector::Openai));
}

#[test]
fn configuration_rejects_a_missing_default_profile() {
    let source = r#"
        default = "missing"

        [profiles.openai]
        connector = "openai"
        model = "gpt-5.2"
    "#;

    let error = Config::from_sources(source, None).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("default profile \"missing\" does not exist")
    );
}

#[test]
fn profile_rejects_an_api_key_instead_of_an_environment_name() {
    let profile = Profile {
        connector: Connector::Openai,
        model: "gpt-5.2".into(),
        base_url: "https://api.openai.com/v1".into(),
        api_key_env: Some("sk-should-not-be-stored-here".into()),
    };

    let error = profile.api_key().unwrap_err();

    assert!(error.to_string().contains("environment-variable name"));
}

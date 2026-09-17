use std::{
    collections::HashMap,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;
use serde::Deserialize;

const INITIAL_CONFIG: &str = r#"# The environment variable containing your API key.
default = "openai"

[profiles.openai]
connector = "openai"
model = "gpt-5.2"
api_key_env = "OPENAI_API_KEY"
"#;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub default: String,
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub connector: Connector,
    pub model: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    pub api_key_env: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Connector {
    Openai,
    OpenaiCompatible,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("could not create {}", parent.display()))?;
        }

        if !path.exists() {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .with_context(|| format!("could not create {}", path.display()))?;
            file.write_all(INITIAL_CONFIG.as_bytes())
                .with_context(|| format!("could not write {}", path.display()))?;
            eprintln!("Created config at {}", path.display());
        }

        let source = fs::read_to_string(&path)
            .with_context(|| format!("could not read {}", path.display()))?;
        let mut values: toml::Value = toml::from_str(&source)
            .with_context(|| format!("invalid config in {}", path.display()))?;

        let local_path = Path::new(".jarvis.toml");
        if local_path.is_file() {
            let local_source =
                fs::read_to_string(local_path).context("could not read .jarvis.toml")?;
            let local_values: toml::Value =
                toml::from_str(&local_source).context("invalid config in .jarvis.toml")?;
            merge(&mut values, local_values);
        }

        let config: Config = values
            .try_into()
            .context("merged configuration is invalid")?;

        if !config.profiles.contains_key(&config.default) {
            bail!("default profile {:?} does not exist", config.default);
        }

        Ok(config)
    }

    pub fn selected_profile(&self) -> &Profile {
        &self.profiles[&self.default]
    }
}

impl Profile {
    pub fn api_key(&self) -> Result<Option<String>> {
        self.api_key_env
            .as_ref()
            .map(|name| {
                if name.starts_with("sk-") {
                    bail!(
                        "api_key_env must be an environment-variable name, such as OPENAI_API_KEY"
                    );
                }
                env::var(name).with_context(|| format!("environment variable {name} is not set"))
            })
            .transpose()
    }
}

fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "jarvis")
        .context("could not determine the operating system config directory")?;
    Ok(dirs.config_dir().join("config.toml"))
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".into()
}

fn merge(base: &mut toml::Value, override_value: toml::Value) {
    match (base, override_value) {
        (toml::Value::Table(base), toml::Value::Table(overrides)) => {
            for (key, value) in overrides {
                match base.get_mut(&key) {
                    Some(existing) => merge(existing, value),
                    None => {
                        base.insert(key, value);
                    }
                }
            }
        }
        (base, value) => *base = value,
    }
}

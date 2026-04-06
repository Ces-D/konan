use clap::ValueEnum;
use std::fmt::Display;

fn shell_escape(value: &str) -> String {
    value.replace('\'', "'\\''")
}

pub struct PiCommandBuilder {
    parts: Vec<String>,
}

impl PiCommandBuilder {
    pub fn new(subcommand: &str) -> Self {
        Self {
            parts: vec!["pi_cli".to_string(), subcommand.to_string()],
        }
    }

    pub fn positional(mut self, value: &str) -> Self {
        self.parts.push(format!("'{}'", shell_escape(value)));
        self
    }

    pub fn flag(mut self, name: &str, enabled: bool) -> Self {
        if enabled {
            self.parts.push(format!("--{name}"));
        }
        self
    }

    pub fn named<V: Display>(mut self, name: &str, value: Option<V>) -> Self {
        if let Some(v) = value {
            self.parts
                .push(format!("--{name} '{}'", shell_escape(&v.to_string())));
        }
        self
    }

    pub fn named_enum<V: ValueEnum>(mut self, name: &str, value: Option<V>) -> Self {
        if let Some(v) = value {
            let enum_name = v
                .to_possible_value()
                .expect("ValueEnum variant must have a name")
                .get_name()
                .to_string();
            self.parts.push(format!("--{name} {enum_name}"));
        }
        self
    }

    pub fn build(self) -> String {
        self.parts.join(" ")
    }
}

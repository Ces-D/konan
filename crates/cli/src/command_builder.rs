use clap::ValueEnum;
use std::fmt::Display;

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
        self.parts.push(format!("'{value}'"));
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
            self.parts.push(format!("--{name} '{v}'"));
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

#[cfg(test)]
mod tests {
    use super::*;
    use cli_shared::{DateBanner, RemoteFile, TimePeriod};

    #[test]
    fn test_template_box_full() {
        let cmd = PiCommandBuilder::new("template box")
            .named("rows", Some(29u32))
            .flag("lined", true)
            .named_enum("date", Some(DateBanner::Today))
            .named("banner", Some("hello"))
            .flag("no-cut", true)
            .build();
        assert_eq!(
            cmd,
            "pi_cli template box --rows '29' --lined --date today --banner 'hello' --no-cut"
        );
    }

    #[test]
    fn test_template_box_minimal() {
        let cmd = PiCommandBuilder::new("template box")
            .named::<u32>("rows", None)
            .flag("lined", false)
            .named_enum::<DateBanner>("date", None)
            .named::<String>("banner", None)
            .flag("no-cut", false)
            .build();
        assert_eq!(cmd, "pi_cli template box");
    }

    #[test]
    fn test_habit_tracker() {
        let cmd = PiCommandBuilder::new("template habit-tracker")
            .positional("Exercise")
            .named("start-date", Some("2025-01-01"))
            .named_enum("time-period", Some(TimePeriod::TwoWeek))
            .flag("no-cut", true)
            .build();
        assert_eq!(
            cmd,
            "pi_cli template habit-tracker 'Exercise' --start-date '2025-01-01' --time-period two-week --no-cut"
        );
    }

    #[test]
    fn test_file_command() {
        let cmd = PiCommandBuilder::new("file")
            .named_enum("file", Some(RemoteFile::Markdown))
            .named("rows", Some(40u32))
            .flag("no-cut", false)
            .build();
        assert_eq!(cmd, "pi_cli file --file markdown --rows '40'");
    }
}

use anyhow::Result;
use chrono::{TimeZone, Utc};
use cli_shared::PrintJob;
use rrule::{Unvalidated, Validated};
use serde::{Deserialize, Serialize};

pub fn nyc_tz() -> rrule::Tz {
    rrule::Tz::America__New_York
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPulse {
    name: String,
    command: String,
    start_date: i64,
    r_rule: String,
    last_run: i64,
}
impl NewPulse {
    pub fn new(name: String, command: PrintJob, r_rule: rrule::RRule<Unvalidated>) -> Result<Self> {
        let now = Utc::now().with_timezone(&nyc_tz());
        let validated = r_rule.validate(now)?;
        Ok(Self {
            name,
            command: command.into(),
            start_date: now.timestamp(),
            r_rule: validated.to_string(),
            last_run: now.timestamp(),
        })
    }
}

mod de {
    use super::*;
    use serde::Deserializer;

    pub fn print_job<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PrintJob, D::Error> {
        let s = String::deserialize(deserializer)?;
        PrintJob::try_from(s).map_err(serde::de::Error::custom)
    }

    pub fn timestamp<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<chrono::DateTime<Utc>, D::Error> {
        let ts = i64::deserialize(deserializer)?;
        Utc.timestamp_opt(ts, 0)
            .single()
            .ok_or_else(|| serde::de::Error::custom("invalid unix timestamp"))
    }

    pub fn rrule<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<rrule::RRule<Unvalidated>, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse::<rrule::RRule<Unvalidated>>()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
pub struct Pulse {
    pub id: i64,
    pub name: String,
    #[serde(deserialize_with = "de::print_job")]
    pub command: PrintJob,
    #[serde(deserialize_with = "de::timestamp")]
    pub start_date: chrono::DateTime<Utc>,
    #[serde(deserialize_with = "de::rrule")]
    r_rule: rrule::RRule<Unvalidated>,
    #[serde(deserialize_with = "de::timestamp")]
    pub last_run: chrono::DateTime<Utc>,
}

impl Pulse {
    pub fn validated_rrule(&self) -> Result<rrule::RRule<Validated>, rrule::RRuleError> {
        let dt = self.start_date.with_timezone(&nyc_tz());
        self.r_rule.clone().validate(dt)
    }
}

#[derive(Serialize)]
pub struct CompactPulse {
    id: i64,
    pub name: String,
    pub command: String,
    pub next_run: chrono::DateTime<Utc>,
}
impl TryFrom<Pulse> for CompactPulse {
    type Error = anyhow::Error;

    fn try_from(value: Pulse) -> Result<Self> {
        let validated = value.validated_rrule()?;
        let now = Utc::now().with_timezone(&nyc_tz());
        let dt_start = value.start_date.with_timezone(&nyc_tz());

        let rrule_set = rrule::RRuleSet::new(dt_start).rrule(validated);
        let next = rrule_set
            .after(now)
            .all(1)
            .dates
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no future occurrences for pulse '{}'", value.name))?;

        Ok(Self {
            id: value.id,
            name: value.name,
            command: serde_json::to_string(&value.command)?,
            next_run: next.with_timezone(&Utc),
        })
    }
}

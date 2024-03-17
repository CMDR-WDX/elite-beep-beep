use std::str::FromStr;

use anyhow::Result;
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct InteractionRaw {
    #[serde(rename(deserialize = "UserID"))]
    user_id: u64,
    #[serde(rename(deserialize = "CommanderID"))]
    commander_id: u64,
    #[serde(rename(deserialize = "Epoch"))]
    epoch: u64,
    #[serde(rename(deserialize = "Interactions"))]
    interaction: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]

struct InteractionsRaw {
    #[serde(rename(deserialize = "Interactions"))]
    interactions: Vec<InteractionRaw>,
}

#[derive(PartialEq)]
pub(crate) struct MetInteraction {
    pub(crate) commander_id: u64,
    pub(crate) time: DateTime<Utc>,
    pub(crate) iso_time: String,
}

impl MetInteraction {
    pub(crate) fn new(commander_id: u64, time: DateTime<Utc>) -> Self {
        Self {
            commander_id,
            time,
            iso_time: time.to_string(),
        }
    }
}

pub(crate) fn serialize_file_contents(text_data: &str) -> Result<Vec<MetInteraction>> {
    let serialized_raw: InteractionsRaw = serde_json::from_str(text_data)?;

    let return_data: Vec<_> = serialized_raw
        .interactions
        .into_iter()
        .filter_map(|x| {
            if !x.interaction.contains(&"Met".to_string()) {
                return None;
            }
            // for whatever f**ing reason, the time defined in the History File is elapsed seconds relative to… 1601-01-01.
            // why? fuck knows.
            let offset_anker = NaiveDateTime::new(
                NaiveDate::from_str("1601-01-01").unwrap(),
                NaiveTime::from_str("00:00").unwrap(),
            );

            let naive_time = offset_anker + TimeDelta::try_seconds(x.epoch as i64)?;

            return Some(MetInteraction::new(x.commander_id, naive_time.and_utc()));
        })
        .collect();

    return Ok(return_data);
}

///
/// The History File contains entries that happened minutes, hours or days ago.
/// We don't care about those.
/// For now, filter out everything that happened a minute or longer ago
///
pub(crate) fn filter_for_only_relevant_entries(data: Vec<MetInteraction>) -> Vec<MetInteraction> {
    let now = Utc::now();

    let mut return_data: Vec<MetInteraction> = vec![];

    for entry in data {
        if (now - &entry.time) > Duration::try_seconds(60).unwrap() {
            continue;
        }

        return_data.push(entry);
    }

    for entry in &return_data {
        use colored::Colorize;
        eprintln!(
            "{}  :::  {}",
            entry.iso_time.green(),
            entry.commander_id.to_string().green()
        );
    }

    return return_data;
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parse_test() {
        let input = include_str!("./test-src/Commander5788062.cmdrHistory");

        let result = serialize_file_contents(input).unwrap();
        assert_eq!(result.len(), 3);
    }
}

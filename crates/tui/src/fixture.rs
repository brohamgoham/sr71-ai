use std::{collections::HashSet, fs, path::Path};

use thiserror::Error;
use world::Mist;

use crate::{
    model::{AgentId, Fixture},
    observer::{ObserveError, ObservedWorld},
};

#[derive(Debug, Error)]
pub enum FixtureError {
    #[error("cannot read fixture {path}: {source}; check the path and read permissions")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("cannot parse fixture JSON: {0}; use the version 1 demo schema")]
    Json(#[from] serde_json::Error),
    #[error("invalid fixture: {0}; correct the fixture before playback")]
    Invalid(String),
    #[error(transparent)]
    Observe(#[from] ObserveError),
}

pub fn read(path: &Path) -> Result<Fixture, FixtureError> {
    let input = fs::read_to_string(path).map_err(|source| FixtureError::Read {
        path: path.display().to_string(),
        source,
    })?;
    parse(&input)
}

pub fn parse(input: &str) -> Result<Fixture, FixtureError> {
    let fixture: Fixture = serde_json::from_str(input)?;
    validate(&fixture)?;
    Ok(fixture)
}

pub fn story() -> Result<Fixture, FixtureError> {
    parse(include_str!("../fixtures/story.json"))
}

pub fn validate(fixture: &Fixture) -> Result<(), FixtureError> {
    if fixture.schema_version != 1 {
        return invalid("unsupported schema version");
    }
    if fixture.run_id.trim().is_empty()
        || fixture.title.trim().is_empty()
        || fixture.duration_seconds == 0
        || fixture.duration_seconds > 86_400
    {
        return invalid("run identity/title and a duration of 1–86400 seconds are required");
    }
    if fixture.roster.is_empty() || fixture.roster.len() > 64 {
        return invalid("roster must contain 1–64 residents");
    }
    if fixture.floor == Mist::ZERO || usize::from(fixture.focus.0) >= fixture.roster.len() {
        return invalid("floor must be positive and focus must identify a resident");
    }
    let mut names = HashSet::new();
    for (index, resident) in fixture.roster.iter().enumerate() {
        if usize::from(resident.id.0) != index {
            return invalid("agent IDs must match roster indices");
        }
        if resident.name.is_empty()
            || resident.name.len() > 32
            || !resident
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !names.insert(&resident.name)
        {
            return invalid("resident names must be unique, path-safe and 1–32 bytes");
        }
        if resident.origin.trim().is_empty()
            || resident.disposition.trim().is_empty()
            || resident.start < fixture.floor
        {
            return invalid("origin and disposition must not be blank");
        }
    }
    if fixture.brackets.first().map(|b| b.min) != Some(Mist::ZERO)
        || fixture.brackets.windows(2).any(|b| b[0].min >= b[1].min)
    {
        return invalid("wealth brackets must start at zero and strictly increase");
    }
    let mut brackets = HashSet::new();
    if fixture
        .brackets
        .iter()
        .any(|b| b.name.trim().is_empty() || !brackets.insert(&b.name))
    {
        return invalid("bracket names must be nonempty and unique");
    }
    let mut world = ObservedWorld::new(fixture);
    let mut ids = HashSet::new();
    for event in &fixture.events {
        if event.id.0.is_empty() || !ids.insert(&event.id) || event.at > fixture.duration_seconds {
            return invalid("event IDs must be unique/nonempty and timestamps within duration");
        }
        world.apply(event)?;
    }
    if world.agent(AgentId(0)).is_none() {
        return invalid("missing first resident");
    }
    Ok(())
}

fn invalid<T>(reason: &str) -> Result<T, FixtureError> {
    Err(FixtureError::Invalid(reason.to_owned()))
}

use serde::{Deserialize, Serialize};
use world::Mist;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(pub u16);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventId(pub String);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifeState {
    #[default]
    Alive,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    Scheduled,
    Observing,
    Deciding,
    Pending,
    #[default]
    Waiting,
    Failed,
    Unknown,
}

impl Activity {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Observing => "observing",
            Self::Deciding => "deciding",
            Self::Pending => "pending",
            Self::Waiting => "waiting",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Connection {
    #[default]
    Current,
    Disconnected,
    Recovering,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "visibility", rename_all = "snake_case", deny_unknown_fields)]
pub enum DecisionNote {
    Visible { text: String },
    Sealed { record_id: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resident {
    pub id: AgentId,
    pub name: String,
    pub origin: String,
    pub start: Mist,
    pub disposition: String,
    #[serde(default)]
    pub life: LifeState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bracket {
    pub name: String,
    pub min: Mist,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ObserverInput {
    Posted {
        agent: AgentId,
        text: String,
        reply_to: Option<EventId>,
        to: Option<AgentId>,
        transaction: String,
    },
    Gave {
        from: AgentId,
        to: AgentId,
        amount: Mist,
        transaction: String,
    },
    TopUp {
        agent: AgentId,
        amount: Mist,
        transaction: String,
    },
    Epoch {
        number: u32,
    },
    Note {
        agent: AgentId,
        note: DecisionNote,
    },
    Activity {
        agent: AgentId,
        state: Activity,
    },
    Connection {
        state: Connection,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Fixture,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub id: EventId,
    pub at: u64,
    pub source: Provenance,
    pub input: ObserverInput,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub schema_version: u16,
    pub run_id: String,
    pub title: String,
    pub duration_seconds: u64,
    pub focus: AgentId,
    pub floor: Mist,
    pub roster: Vec<Resident>,
    pub brackets: Vec<Bracket>,
    pub events: Vec<Envelope>,
}

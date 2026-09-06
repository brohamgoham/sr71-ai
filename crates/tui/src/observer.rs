use std::collections::{BTreeMap, HashMap};

use thiserror::Error;
use world::Mist;

use crate::model::{
    Activity, AgentId, Bracket, Connection, DecisionNote, Envelope, EventId, Fixture,
    ObserverInput, Resident,
};

#[derive(Debug, Error)]
#[error("cannot observe {event}: {reason}; check the fixture's identities, order and amounts")]
pub struct ObserveError {
    pub event: String,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct AgentCard {
    pub resident: Resident,
    pub balance: Mist,
    pub activity: Activity,
    pub note: Option<DecisionNote>,
    pub posts: BTreeMap<u32, u64>,
    pub given: Mist,
    pub received: Mist,
    pub history: Vec<u64>,
}

#[derive(Clone, Debug)]
pub struct Moment {
    pub at: u64,
    pub title: String,
    pub evidence: EventId,
}

#[derive(Clone, Debug)]
pub struct ObservedWorld {
    pub agents: Vec<AgentCard>,
    pub events: Vec<Envelope>,
    pub moments: Vec<Moment>,
    pub epoch: u32,
    pub connection: Connection,
    pub gifts: u64,
    pub topups: Mist,
    pub floor: Mist,
    pub pairs: BTreeMap<(AgentId, AgentId), (Mist, u64)>,
    pub brackets: Vec<Bracket>,
    pub gift_epochs: BTreeMap<u32, u64>,
    seen: HashMap<EventId, usize>,
    largest: Mist,
}

impl ObservedWorld {
    pub fn new(fixture: &Fixture) -> Self {
        let agents = fixture
            .roster
            .iter()
            .map(|resident| AgentCard {
                resident: resident.clone(),
                balance: resident.start,
                activity: Activity::Waiting,
                note: None,
                posts: BTreeMap::new(),
                given: Mist::ZERO,
                received: Mist::ZERO,
                history: vec![resident.start.value()],
            })
            .collect();
        Self {
            agents,
            events: Vec::new(),
            moments: Vec::new(),
            epoch: 0,
            connection: Connection::Current,
            gifts: 0,
            topups: Mist::ZERO,
            floor: fixture.floor,
            pairs: BTreeMap::new(),
            brackets: fixture.brackets.clone(),
            gift_epochs: BTreeMap::new(),
            seen: HashMap::new(),
            largest: Mist::ZERO,
        }
    }

    pub fn agent(&self, id: AgentId) -> Option<&AgentCard> {
        self.agents
            .get(usize::from(id.0))
            .filter(|a| a.resident.id == id)
    }

    pub fn name(&self, id: AgentId) -> &str {
        self.agent(id)
            .map_or("Unknown", |a| a.resident.name.as_str())
    }

    pub fn bracket(&self, balance: Mist) -> &str {
        self.brackets
            .iter()
            .rev()
            .find(|b| balance >= b.min)
            .map_or("unknown", |b| b.name.as_str())
    }

    pub fn apply(&mut self, event: &Envelope) -> Result<bool, ObserveError> {
        if let Some(index) = self.seen.get(&event.id) {
            if self.events.get(*index) == Some(event) {
                return Ok(false);
            }
            return Err(Self::error(event, "event ID reused with different content"));
        }
        if self.events.last().is_some_and(|last| last.at > event.at) {
            return Err(Self::error(event, "events are out of order"));
        }
        self.apply_input(event)?;
        self.seen.insert(event.id.clone(), self.events.len());
        self.events.push(event.clone());
        Ok(true)
    }

    fn apply_input(&mut self, event: &Envelope) -> Result<(), ObserveError> {
        match &event.input {
            ObserverInput::Posted {
                agent,
                text,
                reply_to,
                to,
                transaction,
            } => {
                self.check_agent(*agent, event)?;
                if let Some(to) = to {
                    self.check_agent(*to, event)?;
                }
                if text.is_empty() || text.len() > 1024 || transaction.is_empty() {
                    return Err(Self::error(
                        event,
                        "post must have 1–1024 bytes and a transaction ID",
                    ));
                }
                if let Some(reply) = reply_to {
                    let original = self
                        .seen
                        .get(reply)
                        .and_then(|index| self.events.get(*index));
                    if !original.is_some_and(|e| matches!(e.input, ObserverInput::Posted { .. })) {
                        return Err(Self::error(event, "reply must refer to an earlier post"));
                    }
                }
                let actor = &mut self.agents[usize::from(agent.0)];
                *actor.posts.entry(self.epoch).or_default() += 1;
                actor.activity = Activity::Waiting;
            }
            ObserverInput::Gave {
                from,
                to,
                amount,
                transaction,
            } => {
                if transaction.is_empty() {
                    return Err(Self::error(event, "missing transaction ID"));
                }
                self.give(event, (*from, *to), *amount)?;
            }
            ObserverInput::TopUp {
                agent,
                amount,
                transaction,
            } => {
                self.check_agent(*agent, event)?;
                if *amount == Mist::ZERO || transaction.is_empty() {
                    return Err(Self::error(
                        event,
                        "top-up requires a positive amount and transaction",
                    ));
                }
                let balance = self.agents[usize::from(agent.0)]
                    .balance
                    .checked_add(*amount)
                    .ok_or_else(|| Self::error(event, "top-up balance overflow"))?;
                let total = self
                    .topups
                    .checked_add(*amount)
                    .ok_or_else(|| Self::error(event, "top-up total overflow"))?;
                if balance != self.floor {
                    return Err(Self::error(event, "top-up must restore exactly the floor"));
                }
                self.topups = total;
                self.agents[usize::from(agent.0)].balance = balance;
            }
            ObserverInput::Epoch { number } => {
                if Some(*number) != self.epoch.checked_add(1) {
                    return Err(Self::error(event, "epoch gap"));
                }
                self.epoch = *number;
                for agent in &mut self.agents {
                    agent.history.push(agent.balance.value());
                }
            }
            ObserverInput::Note { agent, note } => {
                self.check_agent(*agent, event)?;
                self.agents[usize::from(agent.0)].note = Some(note.clone());
            }
            ObserverInput::Activity { agent, state } => {
                self.check_agent(*agent, event)?;
                self.agents[usize::from(agent.0)].activity = *state;
            }
            ObserverInput::Connection { state } => {
                self.connection = *state;
                if *state != Connection::Current {
                    for agent in &mut self.agents {
                        agent.activity = Activity::Unknown;
                    }
                }
            }
        }
        Ok(())
    }

    fn give(
        &mut self,
        event: &Envelope,
        pair: (AgentId, AgentId),
        amount: Mist,
    ) -> Result<(), ObserveError> {
        let (from, to) = pair;
        self.check_agent(from, event)?;
        self.check_agent(to, event)?;
        if from == to || amount == Mist::ZERO {
            return Err(Self::error(
                event,
                "gift must be positive and to another resident",
            ));
        }
        let sender = &self.agents[usize::from(from.0)];
        let recipient = &self.agents[usize::from(to.0)];
        let old_brackets = (
            self.bracket(sender.balance).to_owned(),
            self.bracket(recipient.balance).to_owned(),
        );
        let debit = sender
            .balance
            .checked_sub(amount)
            .ok_or_else(|| Self::error(event, "gift exceeds sender balance"))?;
        let credit = recipient
            .balance
            .checked_add(amount)
            .ok_or_else(|| Self::error(event, "recipient balance overflow"))?;
        let given = sender
            .given
            .checked_add(amount)
            .ok_or_else(|| Self::error(event, "given total overflow"))?;
        let received = recipient
            .received
            .checked_add(amount)
            .ok_or_else(|| Self::error(event, "received total overflow"))?;
        let (previous, count) = self.pairs.get(&pair).copied().unwrap_or((Mist::ZERO, 0));
        let pair_total = previous
            .checked_add(amount)
            .ok_or_else(|| Self::error(event, "pair total overflow"))?;
        self.agents[usize::from(from.0)].balance = debit;
        self.agents[usize::from(from.0)].given = given;
        self.agents[usize::from(to.0)].balance = credit;
        self.agents[usize::from(to.0)].received = received;
        self.agents[usize::from(from.0)].activity = Activity::Waiting;
        let label = if self.gifts == 0 {
            "First gift"
        } else if amount > self.largest {
            "Largest gift so far"
        } else if count == 0 {
            "First gift between this pair"
        } else {
            "Another gift"
        };
        self.moments.push(Moment {
            at: event.at,
            evidence: event.id.clone(),
            title: format!(
                "{label}: {} to {} · {amount} SUI",
                self.name(from),
                self.name(to)
            ),
        });
        for (id, old) in [(from, old_brackets.0), (to, old_brackets.1)] {
            let current = self.bracket(self.agents[usize::from(id.0)].balance);
            if current != old {
                self.moments.push(Moment {
                    at: event.at,
                    evidence: event.id.clone(),
                    title: format!("{} entered {current}", self.name(id)),
                });
            }
        }
        self.largest = self.largest.max(amount);
        self.pairs.insert(pair, (pair_total, count + 1));
        self.gifts += 1;
        *self.gift_epochs.entry(self.epoch).or_default() += 1;
        Ok(())
    }

    fn check_agent(&self, agent: AgentId, event: &Envelope) -> Result<(), ObserveError> {
        self.agent(agent)
            .map(|_| ())
            .ok_or_else(|| Self::error(event, "unknown agent ID"))
    }

    fn error(event: &Envelope, reason: &str) -> ObserveError {
        ObserveError {
            event: event.id.0.clone(),
            reason: reason.to_owned(),
        }
    }

    pub fn metrics(&self) -> (u128, f64, Option<f64>) {
        let total: u128 = self
            .agents
            .iter()
            .map(|a| u128::from(a.balance.value()))
            .sum();
        if total == 0 {
            return (0, 0.0, None);
        }
        let mut differences = 0_u128;
        for a in &self.agents {
            for b in &self.agents {
                differences += u128::from(a.balance.value().abs_diff(b.balance.value()));
            }
        }
        let maximum = self
            .agents
            .iter()
            .map(|a| a.balance.value())
            .max()
            .unwrap_or(0);
        (
            total,
            differences as f64 / (2.0 * self.agents.len() as f64 * total as f64),
            Some(maximum as f64 / total as f64),
        )
    }

    pub fn involves(&self, event: &Envelope, selected: AgentId) -> bool {
        match &event.input {
            ObserverInput::Posted {
                agent,
                to,
                reply_to,
                ..
            } => {
                *agent == selected || *to == Some(selected) || reply_to.as_ref().is_some_and(|id| {
                    self.seen.get(id).and_then(|index| self.events.get(*index)).is_some_and(|e| {
                        matches!(e.input, ObserverInput::Posted { agent, .. } if agent == selected)
                    })
                })
            }
            ObserverInput::Gave { from, to, .. } => *from == selected || *to == selected,
            ObserverInput::TopUp { agent, .. }
            | ObserverInput::Note { agent, .. }
            | ObserverInput::Activity { agent, .. } => *agent == selected,
            ObserverInput::Epoch { .. } | ObserverInput::Connection { .. } => false,
        }
    }
}

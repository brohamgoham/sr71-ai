use crate::{
    model::{AgentId, Envelope, Fixture, ObserverInput},
    observer::{Moment, ObserveError, ObservedWorld},
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Tab {
    #[default]
    Watch,
    Cast,
    Economy,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Playback {
    #[default]
    Live,
    Paused,
    Replay,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Overlay {
    #[default]
    None,
    Help,
    Inspect,
    Search,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Speed {
    Half,
    #[default]
    Normal,
    Double,
    Quadruple,
}

impl Speed {
    pub const fn multiplier(self) -> f64 {
        match self {
            Self::Half => 0.5,
            Self::Normal => 1.0,
            Self::Double => 2.0,
            Self::Quadruple => 4.0,
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Half => Self::Normal,
            Self::Normal => Self::Double,
            Self::Double => Self::Quadruple,
            Self::Quadruple => Self::Half,
        }
    }
}

pub struct App {
    pub fixture: Fixture,
    pub world: ObservedWorld,
    pub tab: Tab,
    pub playback: Playback,
    pub overlay: Overlay,
    pub selected: AgentId,
    pub following: Option<AgentId>,
    pub speed: Speed,
    pub query: String,
    pub feed_offset: usize,
    pub playhead: f64,
    pub live_head: f64,
    pub moments: Vec<Moment>,
    applied: usize,
}

impl App {
    pub fn new(fixture: Fixture) -> Result<Self, crate::fixture::FixtureError> {
        crate::fixture::validate(&fixture)?;
        let mut full = ObservedWorld::new(&fixture);
        for event in &fixture.events {
            full.apply(event)?;
        }
        let world = ObservedWorld::new(&fixture);
        let selected = fixture.focus;
        let mut app = Self {
            fixture,
            world,
            tab: Tab::Watch,
            playback: Playback::Live,
            overlay: Overlay::None,
            selected,
            following: None,
            speed: Speed::Normal,
            query: String::new(),
            feed_offset: 0,
            playhead: 0.0,
            live_head: 0.0,
            moments: full.moments,
            applied: 0,
        };
        app.advance()?;
        Ok(app)
    }

    pub fn tick(&mut self, seconds: f64) -> Result<(), ObserveError> {
        let duration = self.fixture.duration_seconds as f64;
        self.live_head = (self.live_head + seconds).min(duration);
        match self.playback {
            Playback::Live => self.playhead = self.live_head,
            Playback::Paused => {}
            Playback::Replay => {
                self.playhead = (self.playhead + seconds * self.speed.multiplier()).min(duration);
            }
        }
        self.advance()
    }

    fn advance(&mut self) -> Result<(), ObserveError> {
        while let Some(event) = self.fixture.events.get(self.applied) {
            if event.at as f64 > self.playhead {
                break;
            }
            self.world.apply(event)?;
            self.applied += 1;
        }
        Ok(())
    }

    pub fn seek(&mut self, seconds: f64) -> Result<(), ObserveError> {
        self.playhead = seconds.clamp(0.0, self.fixture.duration_seconds as f64);
        self.world = ObservedWorld::new(&self.fixture);
        self.applied = 0;
        self.feed_offset = 0;
        self.advance()
    }

    pub fn toggle_pause(&mut self) {
        self.playback = match self.playback {
            Playback::Live | Playback::Replay => Playback::Paused,
            Playback::Paused => Playback::Replay,
        };
    }

    pub fn go_live(&mut self) -> Result<(), ObserveError> {
        self.playback = Playback::Live;
        self.seek(self.live_head)
    }

    pub fn replay(&mut self) -> Result<(), ObserveError> {
        self.playback = Playback::Replay;
        self.seek(0.0)
    }

    pub fn move_selection(&mut self, forward: bool) {
        let count = self.world.agents.len();
        let current = usize::from(self.selected.0);
        let next = if forward {
            (current + 1) % count
        } else {
            (current + count - 1) % count
        };
        if let Some(agent) = self.world.agents.get(next) {
            self.selected = agent.resident.id;
        }
    }

    pub fn follow(&mut self) {
        self.following = if self.following == Some(self.selected) {
            None
        } else {
            Some(self.selected)
        };
        self.feed_offset = 0;
    }

    pub fn next_moment(&mut self, forward: bool) -> Result<(), ObserveError> {
        let moment = if forward {
            self.moments.iter().find(|m| m.at as f64 > self.playhead)
        } else {
            self.moments
                .iter()
                .rev()
                .find(|m| (m.at as f64) < self.playhead)
        };
        if let Some(moment) = moment {
            let at = moment.at;
            self.playback = Playback::Paused;
            self.seek(at as f64)?;
        }
        Ok(())
    }

    pub fn step(&mut self, forward: bool) -> Result<(), ObserveError> {
        let event = if forward {
            self.fixture
                .events
                .iter()
                .find(|e| e.at as f64 > self.playhead)
        } else {
            self.fixture
                .events
                .iter()
                .rev()
                .find(|e| (e.at as f64) < self.playhead)
        };
        if let Some(event) = event {
            let at = event.at;
            self.playback = Playback::Paused;
            self.seek(at as f64)?;
        }
        Ok(())
    }

    pub fn backlog(&self) -> usize {
        self.fixture
            .events
            .iter()
            .filter(|e| e.at as f64 > self.playhead && e.at as f64 <= self.live_head)
            .count()
    }

    pub fn feed(&self) -> Vec<&Envelope> {
        self.world
            .events
            .iter()
            .filter(|event| {
                let public = matches!(
                    event.input,
                    ObserverInput::Posted { .. }
                        | ObserverInput::Gave { .. }
                        | ObserverInput::TopUp { .. }
                );
                let follows = self
                    .following
                    .is_none_or(|id| self.world.involves(event, id));
                let search = self.query.to_lowercase();
                let searchable = match &event.input {
                    ObserverInput::Posted { agent, text, .. } => {
                        format!("{} {text}", self.world.name(*agent))
                    }
                    ObserverInput::Gave { from, to, .. } => {
                        format!("{} {} gift", self.world.name(*from), self.world.name(*to))
                    }
                    ObserverInput::TopUp { agent, .. } => {
                        format!("{} top-up", self.world.name(*agent))
                    }
                    ObserverInput::Epoch { .. }
                    | ObserverInput::Note { .. }
                    | ObserverInput::Activity { .. }
                    | ObserverInput::Connection { .. } => String::new(),
                };
                public && follows && searchable.to_lowercase().contains(&search)
            })
            .collect()
    }
}

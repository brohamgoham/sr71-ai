use tui::{
    app::{App, Playback},
    fixture,
    model::{
        Activity, AgentId, Connection, DecisionNote, Envelope, EventId, ObserverInput, Provenance,
    },
    observer::ObservedWorld,
};
use world::Mist;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn event(id: &str, at: u64, input: ObserverInput) -> Envelope {
    Envelope {
        id: EventId(id.to_owned()),
        at,
        source: Provenance::Fixture,
        input,
    }
}

#[test]
fn gift_conserves_wealth_and_floor_support_is_separate() -> TestResult {
    let mut app = App::new(fixture::story()?)?;
    let initial = app.world.metrics().0;
    app.seek(350.0)?;
    assert_eq!(app.world.metrics().0, initial);
    assert_eq!(
        app.world.agent(AgentId(14)).map(|a| a.balance.to_string()),
        Some("52.0".into())
    );
    assert_eq!(
        app.world
            .agent(AgentId(14))
            .map(|a| a.resident.origin.as_str()),
        Some("poverty")
    );
    assert_eq!(
        app.world.agent(AgentId(13)).map(|a| a.balance),
        Some(Mist::ZERO)
    );
    assert_eq!(app.world.gifts, 4);
    app.seek(370.0)?;
    assert_eq!(app.world.metrics().0, initial + 100_000_000);
    assert_eq!(app.world.gifts, 4);
    assert_eq!(app.world.topups, Mist::new(100_000_000));
    assert_eq!(
        app.world.agent(AgentId(13)).map(|a| a.balance),
        Some(Mist::new(100_000_000))
    );
    Ok(())
}

#[test]
fn replay_has_no_future_notes_balances_or_rates() -> TestResult {
    let mut app = App::new(fixture::story()?)?;
    app.seek(460.0)?;
    assert!(
        app.world
            .agent(AgentId(2))
            .is_some_and(|a| a.note.is_some())
    );
    let before = app.world.metrics();
    app.seek(44.0)?;
    assert_eq!(app.world.gifts, 0);
    assert_eq!(
        app.world.agent(AgentId(2)).and_then(|a| a.note.as_ref()),
        None
    );
    assert_eq!(
        app.world.agent(AgentId(14)).map(|a| a.balance),
        Some(Mist::new(100_000_000))
    );
    app.seek(460.0)?;
    assert_eq!(app.world.metrics(), before);
    assert_eq!(app.world.agent(AgentId(14)).map(|a| a.posts.len()), Some(4));
    Ok(())
}

#[test]
fn pauses_buffer_and_replay_does_not_move_live_clock() -> TestResult {
    let mut app = App::new(fixture::story()?)?;
    app.tick(10.0)?;
    app.toggle_pause();
    let events = app.world.events.len();
    app.tick(50.0)?;
    assert_eq!(app.world.events.len(), events);
    assert!(app.backlog() > 0);
    app.go_live()?;
    assert_eq!(app.playhead, 60.0);
    assert_eq!(app.world.gifts, 1);
    app.replay()?;
    assert_eq!(app.playhead, 0.0);
    assert_eq!(app.live_head, 60.0);
    app.next_moment(true)?;
    assert_eq!(app.playhead, 45.0);
    assert_eq!(app.playback, Playback::Paused);
    app.step(false)?;
    assert_eq!(app.playhead, 41.0);
    Ok(())
}

#[test]
fn duplicate_delivery_is_idempotent_but_changed_bytes_are_rejected() -> TestResult {
    let fixture = fixture::story()?;
    let mut world = ObservedWorld::new(&fixture);
    let epoch = event("epoch", 0, ObserverInput::Epoch { number: 1 });
    assert!(world.apply(&epoch)?);
    assert!(!world.apply(&epoch)?);
    let changed = event("epoch", 0, ObserverInput::Epoch { number: 2 });
    assert!(world.apply(&changed).is_err());
    assert_eq!(world.epoch, 1);
    let out_of_order = event("older", 1, ObserverInput::Epoch { number: 2 });
    world.apply(&out_of_order)?;
    assert!(
        world
            .apply(&event("past", 0, ObserverInput::Epoch { number: 3 }))
            .is_err()
    );
    Ok(())
}

#[test]
fn invalid_transfers_are_atomic() -> TestResult {
    let fixture = fixture::story()?;
    let invalid = [
        (AgentId(0), AgentId(0), Mist::new(1)),
        (AgentId(0), AgentId(1), Mist::ZERO),
        (AgentId(64), AgentId(1), Mist::new(1)),
        (AgentId(14), AgentId(1), Mist::new(500_000_000)),
    ];
    for (from, to, amount) in invalid {
        let mut world = ObservedWorld::new(&fixture);
        let before = world.metrics();
        let gift = event(
            "invalid",
            0,
            ObserverInput::Gave {
                from,
                to,
                amount,
                transaction: "tx".into(),
            },
        );
        assert!(world.apply(&gift).is_err());
        assert_eq!(world.metrics(), before);
        assert_eq!(world.gifts, 0);
        assert!(world.events.is_empty());
    }
    Ok(())
}

#[test]
fn overflow_and_missing_transaction_fail_before_mutation() -> TestResult {
    let fixture = fixture::story()?;
    let mut world = ObservedWorld::new(&fixture);
    world.agents[1].balance = Mist::new(u64::MAX);
    let gift = event(
        "gift",
        0,
        ObserverInput::Gave {
            from: AgentId(0),
            to: AgentId(1),
            amount: Mist::new(1),
            transaction: "tx".into(),
        },
    );
    assert!(world.apply(&gift).is_err());
    assert_eq!(world.agents[0].balance, fixture.roster[0].start);
    world.agents[1].balance = Mist::ZERO;
    world.agents[0].given = Mist::new(u64::MAX);
    assert!(world.apply(&gift).is_err());
    world.agents[0].given = Mist::ZERO;
    world.agents[1].received = Mist::new(u64::MAX);
    assert!(world.apply(&gift).is_err());
    world.agents[1].received = Mist::ZERO;
    world
        .pairs
        .insert((AgentId(0), AgentId(1)), (Mist::new(u64::MAX), 1));
    assert!(world.apply(&gift).is_err());
    let missing = event(
        "missing",
        0,
        ObserverInput::Gave {
            from: AgentId(0),
            to: AgentId(1),
            amount: Mist::new(1),
            transaction: String::new(),
        },
    );
    assert!(world.apply(&missing).is_err());
    Ok(())
}

#[test]
fn invalid_posts_replies_notes_and_epochs_fail() -> TestResult {
    let fixture = fixture::story()?;
    let mut world = ObservedWorld::new(&fixture);
    for text in [String::new(), "x".repeat(1025)] {
        let input = ObserverInput::Posted {
            agent: AgentId(0),
            text,
            reply_to: None,
            to: None,
            transaction: "tx".into(),
        };
        assert!(world.apply(&event("bad", 0, input)).is_err());
    }
    for (reply_to, to) in [
        (Some(EventId("future".into())), None),
        (None, Some(AgentId(64))),
    ] {
        let input = ObserverInput::Posted {
            agent: AgentId(0),
            text: "Hello".into(),
            reply_to,
            to,
            transaction: "tx".into(),
        };
        assert!(world.apply(&event("bad", 0, input)).is_err());
    }
    assert!(
        world
            .apply(&event("bad", 0, ObserverInput::Epoch { number: 2 }))
            .is_err()
    );
    assert!(
        world
            .apply(&event(
                "bad",
                0,
                ObserverInput::Note {
                    agent: AgentId(64),
                    note: DecisionNote::Sealed {
                        record_id: "sealed".into()
                    }
                }
            ))
            .is_err()
    );
    assert!(
        world
            .apply(&event(
                "bad",
                0,
                ObserverInput::Activity {
                    agent: AgentId(64),
                    state: Activity::Pending
                }
            ))
            .is_err()
    );
    Ok(())
}

#[test]
fn topups_reject_wrong_floor_zero_and_overflow() -> TestResult {
    let fixture = fixture::story()?;
    let mut world = ObservedWorld::new(&fixture);
    for (agent, amount) in [
        (AgentId(14), Mist::new(1)),
        (AgentId(0), Mist::ZERO),
        (AgentId(64), Mist::new(1)),
        (AgentId(0), Mist::new(u64::MAX)),
    ] {
        assert!(
            world
                .apply(&event(
                    "bad",
                    0,
                    ObserverInput::TopUp {
                        agent,
                        amount,
                        transaction: "tx".into()
                    }
                ))
                .is_err()
        );
        assert_eq!(world.topups, Mist::ZERO);
    }
    world.agents[14].balance = Mist::ZERO;
    world.topups = Mist::new(u64::MAX);
    assert!(
        world
            .apply(&event(
                "bad",
                0,
                ObserverInput::TopUp {
                    agent: AgentId(14),
                    amount: world.floor,
                    transaction: "tx".into()
                }
            ))
            .is_err()
    );
    Ok(())
}

#[test]
fn pending_failure_never_moves_wealth_and_disconnect_retains_cast() -> TestResult {
    let mut app = App::new(fixture::story()?)?;
    app.seek(211.0)?;
    assert_eq!(app.world.agents[12].activity, Activity::Pending);
    let wealth = app.world.metrics();
    app.seek(220.0)?;
    assert_eq!(app.world.agents[12].activity, Activity::Failed);
    assert_eq!(app.world.metrics(), wealth);
    app.world.apply(&event(
        "offline",
        221,
        ObserverInput::Connection {
            state: Connection::Disconnected,
        },
    ))?;
    assert_eq!(app.world.agents.len(), 16);
    assert!(
        app.world
            .agents
            .iter()
            .all(|a| a.activity == Activity::Unknown)
    );
    Ok(())
}

#[test]
fn metrics_cover_equal_zero_and_whale_distributions() -> TestResult {
    let fixture = fixture::story()?;
    let mut world = ObservedWorld::new(&fixture);
    for a in &mut world.agents {
        a.balance = Mist::new(10);
    }
    assert_eq!(world.metrics(), (160, 0.0, Some(1.0 / 16.0)));
    for a in &mut world.agents {
        a.balance = Mist::ZERO;
    }
    assert_eq!(world.metrics(), (0, 0.0, None));
    world.agents[0].balance = Mist::new(160);
    assert_eq!(world.metrics(), (160, 15.0 / 16.0, Some(1.0)));
    Ok(())
}

#[test]
fn thousand_event_burst_is_retained_and_replayable() -> TestResult {
    let fixture = fixture::parse(include_str!("../fixtures/burst.json"))?;
    let mut app = App::new(fixture)?;
    app.tick(20.0)?;
    assert_eq!(app.feed().len(), 1000);
    assert_eq!(
        app.world
            .agents
            .iter()
            .flat_map(|a| a.posts.values())
            .sum::<u64>(),
        1000
    );
    app.seek(0.0)?;
    assert_eq!(app.feed().len(), 0);
    app.seek(20.0)?;
    assert_eq!(app.feed().len(), 1000);
    Ok(())
}

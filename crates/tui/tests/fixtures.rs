use serde_json::{Value, json};
use tui::fixture;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn malformed_files_have_actionable_errors() -> TestResult {
    assert!(fixture::parse("not json").is_err());
    assert!(fixture::read(std::path::Path::new("/missing-square-fixture.json")).is_err());
    let original: Value = serde_json::from_str(include_str!("../fixtures/story.json"))?;
    let mutations = [
        ("/schema_version", json!(2)),
        ("/run_id", json!("")),
        ("/title", json!("")),
        ("/duration_seconds", json!(0)),
        ("/duration_seconds", json!(86401)),
        ("/roster", json!([])),
        ("/roster/0/id", json!(20)),
        ("/roster/0/name", json!("../bad")),
        ("/roster/0/name", json!("Blythe")),
        ("/roster/0/name", json!("x".repeat(33))),
        ("/roster/0/origin", json!("")),
        ("/roster/0/disposition", json!(" ")),
        ("/brackets", json!([])),
        ("/brackets/0/min", json!("1")),
        ("/brackets/1/min", json!("0")),
        ("/brackets/1/name", json!("poverty")),
        ("/brackets/1/name", json!("")),
        ("/events/0/id", json!("")),
        ("/events/0/at", json!(601)),
        ("/events/1/id", json!("demo-0001")),
        ("/events/0/source", json!("confirmed_chain")),
        ("/floor", json!("0")),
        ("/focus", json!(99)),
        ("/roster/0/start", json!("0")),
        ("/roster/0/start", json!("+1")),
        ("/roster/0/life", json!("dead")),
    ];
    for (path, value) in mutations {
        let mut changed = original.clone();
        let Some(slot) = changed.pointer_mut(path) else {
            return Err(format!("missing test field {path}").into());
        };
        *slot = value;
        let error = fixture::parse(&serde_json::to_string(&changed)?);
        assert!(error.is_err(), "accepted invalid field {path}");
    }
    let mut unknown = original.clone();
    unknown["secret"] = json!("must not be silently accepted");
    assert!(fixture::parse(&serde_json::to_string(&unknown)?).is_err());
    let mut count = original;
    count["roster"] = json!(vec![count["roster"][0].clone(); 65]);
    assert!(fixture::parse(&serde_json::to_string(&count)?).is_err());
    Ok(())
}

#[test]
fn all_shipped_scenarios_validate_and_roundtrip() -> TestResult {
    for text in [
        include_str!("../fixtures/story.json"),
        include_str!("../fixtures/quiet.json"),
        include_str!("../fixtures/burst.json"),
        include_str!("../fixtures/reconnect.json"),
    ] {
        let fixture = fixture::parse(text)?;
        let encoded = serde_json::to_string(&fixture)?;
        let decoded = fixture::parse(&encoded)?;
        assert_eq!(decoded.events, fixture.events);
        assert_eq!(decoded.roster, fixture.roster);
    }
    Ok(())
}

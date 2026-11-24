use flexolotl_core::activity::{
    Activity, ActivityKind, Button, Field, Link, MemberLine, Picture, Problem, Progress,
};
use serde_json::json;

fn button(label: &str, url: &str) -> Button {
    Button {
        label: label.into(),
        link: Link::new(url),
    }
}

fn problem_of(activity: &Activity, field: Field) -> Option<Problem> {
    activity
        .issues()
        .into_iter()
        .find(|issue| issue.field == field)
        .map(|issue| issue.problem)
}

#[test]
fn builds_the_full_payload() {
    let activity = Activity {
        name: "Axolotl FM".into(),
        kind: ActivityKind::Listening,
        member_line: MemberLine::Details,
        details: "  first line  ".into(),
        details_link: Link::new("https://nuvo.quest/a"),
        state: "artist".into(),
        large_picture: Picture {
            source: "https://nuvo.quest/a.gif".into(),
            caption: "cover".into(),
            link: Link::default(),
        },
        party: Some([2, 5]),
        progress: Progress::Between {
            start_ms: 10,
            end_ms: 20,
        },
        buttons: vec![button("Open", "https://nuvo.quest")],
        ..Activity::default()
    };
    let expected = json!({
        "name": "Axolotl FM",
        "type": 2,
        "status_display_type": 2,
        "details": "first line",
        "details_url": "https://nuvo.quest/a",
        "state": "artist",
        "assets": { "large_image": "https://nuvo.quest/a.gif", "large_text": "cover" },
        "party": { "id": "flexolotl", "size": [2, 5] },
        "timestamps": { "start": 10, "end": 20 },
        "buttons": [{ "label": "Open", "url": "https://nuvo.quest" }],
    });
    assert_eq!(activity.payload(), Ok(expected));
}

#[test]
fn leaves_out_empty_fields() {
    let payload = Activity::default().payload().unwrap();
    assert_eq!(payload, json!({ "type": 0, "status_display_type": 0 }));
}

#[test]
fn counts_characters_instead_of_bytes() {
    let activity = Activity {
        details: "ы".repeat(128),
        ..Activity::default()
    };
    assert_eq!(activity.issues(), []);
}

#[test]
fn rejects_text_outside_the_limits() {
    let short = Activity {
        details: "a".into(),
        ..Activity::default()
    };
    let long = Activity {
        state: "a".repeat(129),
        ..Activity::default()
    };
    assert_eq!(
        problem_of(&short, Field::Details),
        Some(Problem::TooShort { limit: 2 })
    );
    assert_eq!(
        problem_of(&long, Field::State),
        Some(Problem::TooLong { limit: 128 })
    );
}

#[test]
fn accepts_only_public_https_links() {
    for url in [
        "http://nuvo.quest",
        "javascript:alert(1)",
        "https://",
        "https://localhost/a",
        "https://127.0.0.1/a",
        "https://[::1]/a",
        "https://user:pass@nuvo.quest",
        "https://nuvo.quest/a b",
        "https://nuvo.quest\\@evil.example",
    ] {
        let activity = Activity {
            details_link: Link::new(url),
            ..Activity::default()
        };
        assert_eq!(
            problem_of(&activity, Field::DetailsLink),
            Some(Problem::NotAPublicHttpsLink),
            "{url}"
        );
    }
}

#[test]
fn allows_two_complete_buttons() {
    let two = Activity {
        buttons: vec![
            button("One", "https://nuvo.quest"),
            button("Two", "https://nuvo.quest/two"),
        ],
        ..Activity::default()
    };
    let mut three = two.clone();
    three.buttons.push(button("Three", "https://nuvo.quest/3"));
    assert_eq!(two.issues(), []);
    assert_eq!(
        problem_of(&three, Field::Buttons),
        Some(Problem::TooMany { limit: 2 })
    );
}

#[test]
fn reports_which_button_is_incomplete() {
    let activity = Activity {
        buttons: vec![
            button("One", "https://nuvo.quest"),
            button(" ", "ftp://nuvo.quest"),
        ],
        ..Activity::default()
    };
    assert_eq!(
        problem_of(&activity, Field::ButtonLabel(1)),
        Some(Problem::Missing)
    );
    assert_eq!(
        problem_of(&activity, Field::ButtonLink(1)),
        Some(Problem::NotAPublicHttpsLink)
    );
    assert_eq!(problem_of(&activity, Field::ButtonLabel(0)), None);
}

#[test]
fn rejects_impossible_party_and_time() {
    let activity = Activity {
        party: Some([6, 5]),
        progress: Progress::Between {
            start_ms: 20,
            end_ms: 10,
        },
        ..Activity::default()
    };
    assert_eq!(
        problem_of(&activity, Field::Party),
        Some(Problem::OutOfOrder)
    );
    assert_eq!(
        problem_of(&activity, Field::Progress),
        Some(Problem::OutOfOrder)
    );
}

#[test]
fn accepts_a_built_in_picture_and_rejects_a_local_file() {
    let with_source = |source: &str| Activity {
        large_picture: Picture {
            source: source.into(),
            ..Picture::default()
        },
        ..Activity::default()
    };
    assert_eq!(with_source("flexolotl_wave").issues(), []);
    for source in ["/home/me/cat.png", "file:///home/me/cat.png"] {
        assert_eq!(
            problem_of(&with_source(source), Field::LargePicture),
            Some(Problem::NotAPublicHttpsLink)
        );
    }
}

#[test]
fn survives_a_round_trip_through_json() {
    let activity = Activity {
        kind: ActivityKind::Competing,
        progress: Progress::Until { end_ms: 5 },
        buttons: vec![button("One", "https://nuvo.quest")],
        ..Activity::default()
    };
    let text = serde_json::to_string(&activity).unwrap();
    assert_eq!(serde_json::from_str::<Activity>(&text).unwrap(), activity);
}

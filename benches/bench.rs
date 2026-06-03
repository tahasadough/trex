#![expect(clippy::cast_possible_truncation)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_serialization(c: &mut Criterion) {
    let mut data = trex::model::Sessions::new();
    for i in 0..10 {
        let mut session = trex::model::SavedSession {
            name: format!("session_{i}"),
            path: format!("/home/user/project_{i}"),
            windows: Vec::new(),
            session_options: vec!["default-command \"\"".into()],
            window_options: vec![],
        };
        for j in 0..5 {
            let mut window = trex::model::SavedWindow {
                index: j,
                name: format!("window_{j}"),
                layout: "even-horizontal".into(),
                active_pane: 0,
                path: format!("/home/user/project_{i}/src"),
                panes: Vec::new(),
            };
            for k in 0..3 {
                window.panes.push(trex::model::SavedPane {
                    index: k,
                    path: format!("/home/user/project_{i}/src"),
                    active: k == 0,
                    command: if k == 0 {
                        Some("nvim main.rs".into())
                    } else {
                        None
                    },
                });
            }
            session.windows.push(window);
        }
        data.sessions.push(session);
    }

    c.bench_function("serialize_sessions_data", |b| {
        b.iter(|| serde_json::to_string(black_box(&data)));
    });

    let json = serde_json::to_string(&data).unwrap();
    c.bench_function("deserialize_sessions_data", |b| {
        b.iter(|| {
            let _: trex::model::Sessions = serde_json::from_str(black_box(&json)).unwrap();
        });
    });
}

fn bench_ignore_operations(c: &mut Criterion) {
    let tmp = tempfile::TempDir::new().unwrap();
    std::env::set_var("TREX_DIR", tmp.path());

    c.bench_function("add_to_ignore", |b| {
        b.iter(|| {
            let name = format!("session_{}", rand_number());
            let _ = trex::storage::add_to_ignore(&name);
        });
    });

    std::env::remove_var("TREX_DIR");
}

fn rand_number() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

criterion_group!(benches, bench_serialization, bench_ignore_operations);
criterion_main!(benches);

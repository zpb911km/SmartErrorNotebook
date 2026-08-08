use std::sync::Arc;

use sea_orm::{ColumnTrait, ConnectOptions, Database, DbConn, EntityTrait, QueryFilter};
use serde_json::{json, Value};
use tauri::{test::MockRuntime, WebviewWindow};

use crate::{
    database::{entities, init_database},
    AppState,
};

struct Harness {
    _app: tauri::App<MockRuntime>,
    webview: WebviewWindow<MockRuntime>,
    db: DbConn,
}

impl Harness {
    async fn new() -> Self {
        let mut options = ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).min_connections(1);
        let db = Database::connect(options)
            .await
            .expect("connect contract-test database");
        init_database(&db)
            .await
            .expect("migrate contract-test database");

        let app = tauri::test::mock_builder()
            .manage(AppState {
                db: Arc::new(db.clone()),
            })
            .invoke_handler(app_invoke_handler!())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("build mock Tauri app");
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("build mock webview");

        Self {
            _app: app,
            webview,
            db,
        }
    }

    async fn invoke(&self, command: &str, args: Value) -> Result<Value, Value> {
        let webview = self.webview.clone();
        let command = command.to_owned();

        tokio::task::spawn_blocking(move || {
            let response = tauri::test::get_ipc_response(
                &webview,
                tauri::webview::InvokeRequest {
                    cmd: command,
                    callback: tauri::ipc::CallbackFn(0),
                    error: tauri::ipc::CallbackFn(1),
                    url: if cfg!(any(windows, target_os = "android")) {
                        "http://tauri.localhost"
                    } else {
                        "tauri://localhost"
                    }
                    .parse()
                    .unwrap(),
                    body: tauri::ipc::InvokeBody::Json(args),
                    headers: Default::default(),
                    invoke_key: tauri::test::INVOKE_KEY.to_string(),
                },
            );

            response.map(|body| body.deserialize::<Value>().unwrap())
        })
        .await
        .expect("IPC blocking task panicked")
    }

    async fn ok(&self, command: &str, args: Value) -> Value {
        self.invoke(command, args)
            .await
            .unwrap_or_else(|error| panic!("{command} failed: {error}"))
    }

    async fn err(&self, command: &str, args: Value) -> Value {
        self.invoke(command, args)
            .await
            .expect_err(&format!("{command} unexpectedly succeeded"))
    }
}

fn assert_pending_defaults(model: &Value) {
    assert_eq!(model["version"], 0);
    assert_eq!(model["sync_status"], "pending");
    assert!(model["deleted_at"].is_null());
    assert!(model["created_at"].as_i64().unwrap() > 0);
    assert!(model["updated_at"].as_i64().unwrap() > 0);
}

fn string_set(value: &Value) -> std::collections::HashSet<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subject_commands_contract() {
    let h = Harness::new().await;

    assert_eq!(h.ok("get_subjects", json!({})).await, json!([]));
    let created = h
        .ok(
            "create_subject",
            json!({ "input": { "name": "Mathematics", "color": "#123456" } }),
        )
        .await;
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["name"], "Mathematics");
    assert_eq!(created["color"], "#123456");
    assert_pending_defaults(&created);

    let updated = h
        .ok(
            "update_subject",
            json!({ "input": { "id": id, "name": "Math", "color": null } }),
        )
        .await;
    assert_eq!(updated["name"], "Math");
    // Option fields treat null as "not supplied", so color cannot be cleared.
    assert_eq!(updated["color"], "#123456");
    assert_eq!(updated["sync_status"], "pending");
    let unchanged = h
        .ok("update_subject", json!({ "input": { "id": id } }))
        .await;
    assert_eq!(unchanged["name"], "Math");
    assert_eq!(unchanged["color"], "#123456");
    assert_eq!(
        h.err(
            "update_subject",
            json!({ "input": { "id": "missing", "name": "x", "color": null } })
        )
        .await,
        "Subject not found"
    );

    h.ok("delete_subject", json!({ "id": id })).await;
    assert_eq!(h.ok("get_subjects", json!({})).await, json!([]));
    assert_eq!(
        h.err("delete_subject", json!({ "id": "missing" })).await,
        "Subject not found"
    );

    h.ok(
        "upsert_subject",
        json!({
            "input": {
                "id": "remote-subject",
                "version": 7,
                "status": "ignored",
                "name": "Physics"
            }
        }),
    )
    .await;
    h.ok(
        "upsert_subject",
        json!({
            "input": {
                "id": "remote-subject",
                "version": 8,
                "status": "pending",
                "deleted_at": 99,
                "name": "Modern Physics",
                "color": "blue"
            }
        }),
    )
    .await;
    let remote = entities::subject::Entity::find_by_id("remote-subject")
        .one(&h.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(remote.name, "Modern Physics");
    assert_eq!(remote.version, 8);
    assert_eq!(remote.sync_status, "synced");
    assert_eq!(remote.deleted_at, Some(99));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn error_question_commands_contract() {
    let h = Harness::new().await;
    let wrong_nested_name = h
        .err(
            "create_question",
            json!({
                "input": {
                    "user_id": "u", "subject_id": "s", "source_id": null,
                    "prompt": "p", "type_": "short", "answer": null,
                    "analysis": null, "error_note": null
                }
            }),
        )
        .await;
    assert!(wrong_nested_name.to_string().contains("type"));

    let subject = h
        .ok("create_subject", json!({ "input": { "name": "Math" } }))
        .await;
    let subject_id = subject["id"].as_str().unwrap();

    let first = h
        .ok(
            "create_question",
            json!({
                "input": {
                    "user_id": "u1", "subject_id": subject_id, "source_id": null,
                    "prompt": "quadratic prompt", "type": "short",
                    "answer": "a", "analysis": "roots", "error_note": "note-one"
                }
            }),
        )
        .await;
    assert_pending_defaults(&first);
    assert_eq!(first["userid"], "u1");
    assert_eq!(first["subjectid"], subject_id);
    assert_eq!(first["type_"], "short");
    let first_id = first["id"].as_str().unwrap().to_owned();

    // Creation currently accepts an unknown subject ID.
    let second = h
        .ok(
            "create_question",
            json!({
                "input": {
                    "user_id": "u2", "subject_id": "unknown-subject",
                    "prompt": "geometry", "type": "choice",
                    "analysis": "quadratic appears here"
                }
            }),
        )
        .await;
    let second_id = second["id"].as_str().unwrap().to_owned();

    let all = h.ok("get_questions", json!({})).await;
    assert_eq!(all.as_array().unwrap().len(), 2);
    let filtered = h.ok(
        "get_questions",
        json!({ "filter": { "subject_id": subject_id, "search": "note-one", "limit": 1, "offset": 0 } }),
    ).await;
    assert_eq!(filtered.as_array().unwrap().len(), 1);
    assert_eq!(filtered[0]["id"], first_id);
    let search = h
        .ok(
            "get_questions",
            json!({ "filter": { "search": "quadratic" } }),
        )
        .await;
    assert_eq!(search.as_array().unwrap().len(), 2);

    assert_eq!(
        h.ok("get_question", json!({ "id": first_id })).await["id"],
        first_id
    );
    assert_eq!(
        h.err("get_question", json!({ "id": "missing" })).await,
        "Question not found"
    );
    assert_eq!(
        h.err(
            "update_question",
            json!({ "input": { "id": first_id, "subject_id": "missing" } })
        )
        .await,
        "Subject not found"
    );
    let updated = h
        .ok(
            "update_question",
            json!({
                "input": {
                    "id": first_id, "subject_id": subject_id, "source_id": "source-x",
                    "prompt": "updated", "type": "essay", "answer": null,
                    "analysis": "new analysis", "error_note": null
                }
            }),
        )
        .await;
    assert_eq!(updated["sourceid"], "source-x");
    assert_eq!(updated["answer"], "a");
    assert_eq!(updated["error_note"], "note-one");

    h.ok(
        "create_srs_data",
        json!({ "input": { "question_id": first_id, "difficulty": null } }),
    )
    .await;
    h.ok("delete_question", json!({ "id": first_id })).await;
    assert_eq!(
        h.ok("get_question_stats", json!({})).await,
        json!({ "total": 1 })
    );
    assert_eq!(
        h.ok("get_questions", json!({ "filter": null })).await,
        json!([second])
    );
    // Direct lookup does not filter soft-deleted questions.
    assert!(!h.ok("get_question", json!({ "id": first_id })).await["deleted_at"].is_null());
    let srs = entities::srs_data::Entity::find()
        .filter(entities::srs_data::Column::QuestionId.eq(&first_id))
        .one(&h.db)
        .await
        .unwrap()
        .unwrap();
    assert!(srs.deleted_at.is_some());

    h.ok(
        "upsert_error_question",
        json!({
                "input": {
                "id": "remote-question", "version": 3, "status": "ignored",
                "userid": "remote-user", "subject_id": subject_id,
                "prompt": "remote", "type_": "choice", "sync_hash": "hash"
            }
        }),
    )
    .await;
    h.ok(
        "upsert_error_question",
        json!({
            "input": {
                "id": "remote-question", "version": 4, "status": "pending", "deleted_at": 123,
                "userid": "changed-user", "subjectid": subject_id, "sourceid": "s",
                "prompt": "remote-updated", "type_": "essay", "answer": "x",
                "analysis": null, "error_note": null, "sync_hash": "changed"
            }
        }),
    )
    .await;
    let remote = entities::error_question::Entity::find_by_id("remote-question")
        .one(&h.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(remote.userid, "remote-user");
    assert_eq!(remote.prompt, "remote-updated");
    assert_eq!(remote.sync_hash.as_deref(), Some("hash"));
    assert_eq!(remote.sync_status, "synced");
    assert_eq!(remote.deleted_at, Some(123));
    assert!(entities::error_question::Entity::find_by_id(second_id)
        .one(&h.db)
        .await
        .unwrap()
        .is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn source_commands_contract() {
    let h = Harness::new().await;
    assert_eq!(h.ok("get_sources", json!({})).await, json!([]));
    assert_eq!(
        h.ok("get_sources", json!({ "filter": null })).await,
        json!([])
    );

    let empty_source = h.ok("create_source", json!({ "input": {} })).await;
    assert!(empty_source["subject_id"].is_null());
    assert!(empty_source["book"].is_null());

    let one = h.ok(
        "create_source",
        json!({ "input": { "subject_id": "s1", "book": "Book A", "chapter": "C1", "knowledge": "K1" } }),
    ).await;
    let one_id = one["id"].as_str().unwrap().to_owned();
    assert_pending_defaults(&one);
    let two = h.ok(
        "create_source",
        json!({ "input": { "subject_id": "s1", "book": "Book A", "chapter": "C2", "knowledge": "K2" } }),
    ).await;
    let three = h.ok(
        "create_source",
        json!({ "input": { "subject_id": "s2", "book": "Book B", "chapter": "C1", "knowledge": null } }),
    ).await;

    assert_eq!(
        h.ok("get_source", json!({ "id": one_id })).await["book"],
        "Book A"
    );
    assert_eq!(
        h.err("get_source", json!({ "id": "missing" })).await,
        "Source not found"
    );
    assert_eq!(
        h.ok("get_sources", json!({ "filter": { "subject_id": "s1" } }))
            .await
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        string_set(&h.ok("get_books", json!({ "subjectId": null })).await),
        ["Book A".to_owned(), "Book B".to_owned()].into()
    );
    assert_eq!(
        string_set(&h.ok("get_books", json!({})).await),
        ["Book A".to_owned(), "Book B".to_owned()].into()
    );
    assert_eq!(
        string_set(
            &h.ok(
                "get_chapters",
                json!({ "subjectId": "s1", "book": "Book A" })
            )
            .await
        ),
        ["C1".to_owned(), "C2".to_owned()].into()
    );
    assert_eq!(
        h.ok(
            "get_knowledges",
            json!({ "subjectId": "s1", "book": "Book A", "chapter": "C1" })
        )
        .await,
        json!(["K1"])
    );

    let updated = h.ok(
        "update_source",
        json!({
            "input": { "id": one_id, "subject_id": null, "book": "Book A2", "chapter": null, "knowledge": null }
        }),
    ).await;
    assert_eq!(updated["book"], "Book A2");
    assert_eq!(updated["subject_id"], "s1");
    assert_eq!(updated["chapter"], "C1");
    let unchanged = h
        .ok("update_source", json!({ "input": { "id": one_id } }))
        .await;
    assert_eq!(unchanged["book"], "Book A2");
    assert_eq!(unchanged["chapter"], "C1");
    assert_eq!(
        h.err(
            "update_source",
            json!({ "input": { "id": "missing", "subject_id": null, "book": null, "chapter": null, "knowledge": null } })
        ).await,
        "Source not found"
    );

    let exact_id = h.ok(
        "get_or_create_source_id",
        json!({ "input": { "subject_id": "s1", "book": "Book A", "chapter": "C2", "knowledge": "K2" } }),
    ).await;
    assert_eq!(exact_id, two["id"]);
    let null_id = h.ok(
        "get_or_create_source_id",
        json!({ "input": { "subject_id": null, "book": null, "chapter": null, "knowledge": null } }),
    ).await;
    assert_eq!(null_id, empty_source["id"]);
    assert_eq!(
        h.ok("get_or_create_source_id", json!({ "input": {} }))
            .await,
        empty_source["id"]
    );

    h.ok("delete_source", json!({ "id": one_id })).await;
    assert_eq!(
        h.err("delete_source", json!({ "id": "missing" })).await,
        "Source not found"
    );
    // Direct lookup includes a soft-deleted source, list APIs do not.
    assert!(!h.ok("get_source", json!({ "id": one_id })).await["deleted_at"].is_null());
    assert!(
        !string_set(&h.ok("get_books", json!({ "subjectId": "s1" })).await).contains("Book A2")
    );

    h.ok(
        "upsert_source",
        json!({
            "input": {
                "id": "remote-source", "version": 2, "status": "ignored"
            }
        }),
    )
    .await;
    h.ok(
        "upsert_source",
        json!({
            "input": {
                "id": "remote-source", "version": 3, "status": "pending", "deleted_at": 77,
                "question_id": "q", "subject_id": null, "book": "Remote 2", "chapter": "C", "knowledge": "K"
            }
        }),
    ).await;
    let remote = entities::source::Entity::find_by_id("remote-source")
        .one(&h.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(remote.version, 3);
    assert_eq!(remote.sync_status, "synced");
    assert_eq!(remote.deleted_at, Some(77));
    assert_eq!(remote.question_id.as_deref(), Some("q"));
    assert_eq!(three["book"], "Book B");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn error_tag_commands_contract() {
    let h = Harness::new().await;
    assert_eq!(h.ok("get_error_tags", json!({})).await, json!([]));
    assert_eq!(h.ok("get_full_error_tags", json!({})).await, json!([]));

    let created = h
        .ok(
            "create_error_tags_for_question",
            json!({
                "input": {
                    "question_id": "q1",
                    "tags": [
                        { "name": "Calculation", "color": "red" },
                        { "name": "Calculation", "color": "blue" },
                        { "name": "Concept", "color": "green" }
                    ]
                }
            }),
        )
        .await;
    assert_eq!(created.as_array().unwrap().len(), 3);
    created
        .as_array()
        .unwrap()
        .iter()
        .for_each(assert_pending_defaults);
    assert_eq!(
        h.ok(
            "create_error_tags_for_question",
            json!({ "input": { "question_id": "q1", "tags": [] } })
        )
        .await,
        json!([])
    );

    let unique = h.ok("get_error_tags", json!({})).await;
    assert_eq!(unique.as_array().unwrap().len(), 2);
    let unique_names: std::collections::HashSet<_> = unique
        .as_array()
        .unwrap()
        .iter()
        .map(|tag| tag["name"].as_str().unwrap())
        .collect();
    assert_eq!(unique_names, ["Calculation", "Concept"].into());
    assert_eq!(
        h.ok("get_full_error_tags", json!({}))
            .await
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        h.ok("get_error_tags_for_question", json!({ "questionId": "q1" }))
            .await
            .as_array()
            .unwrap()
            .len(),
        3
    );

    h.ok(
        "update_error_tag_by_name",
        json!({ "oldName": "Calculation", "newName": "Arithmetic", "newColor": "orange" }),
    )
    .await;
    let q1_tags = h
        .ok("get_error_tags_for_question", json!({ "questionId": "q1" }))
        .await;
    assert_eq!(
        q1_tags
            .as_array()
            .unwrap()
            .iter()
            .filter(|tag| tag["name"] == "Arithmetic" && tag["color"] == "orange")
            .count(),
        2
    );

    let concept_id = created[2]["id"].as_str().unwrap();
    h.ok(
        "update_error_tag_by_id",
        json!({ "tagId": concept_id, "newTagName": "Theory" }),
    )
    .await;
    let concept = entities::error_tag::Entity::find_by_id(concept_id)
        .one(&h.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(concept.name, "Theory");
    assert_eq!(concept.color, "green");
    assert_eq!(concept.sync_status, "pending");
    assert_eq!(
        h.err(
            "update_error_tag_by_id",
            json!({ "tagId": "missing", "newTagName": "x", "newTagColor": null })
        )
        .await,
        "标签不存在"
    );

    let deleted_id = created[0]["id"].as_str().unwrap();
    h.ok("delete_error_tag", json!({ "tagId": deleted_id }))
        .await;
    h.ok("delete_error_tag", json!({ "tagId": "missing" }))
        .await;
    assert_eq!(
        h.ok("get_error_tags_for_question", json!({ "questionId": "q1" }))
            .await
            .as_array()
            .unwrap()
            .len(),
        2
    );

    h.ok(
        "upsert_error_tag",
        json!({
            "input": {
                "id": "remote-tag", "version": 4, "status": "ignored",
                "question_id": "q2", "name": "Remote", "color": "black"
            }
        }),
    )
    .await;
    h.ok(
        "upsert_error_tag",
        json!({
            "input": {
                "id": "remote-tag", "version": 5, "status": "pending", "deleted_at": 42,
                "question_id": "q3", "name": "Remote 2", "color": "white"
            }
        }),
    )
    .await;
    let remote = entities::error_tag::Entity::find_by_id("remote-tag")
        .one(&h.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(remote.version, 5);
    assert_eq!(remote.sync_status, "synced");
    assert_eq!(remote.deleted_at, Some(42));
    assert_eq!(remote.question_id, "q3");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn attachment_commands_contract() {
    let h = Harness::new().await;
    let payload = "a".repeat(120);
    let created = h
        .ok(
            "create_attachment",
            json!({
                "input": {
                    "question_id": "q1", "type_": "original", "file_type": "img",
                    "base64_data": payload
                }
            }),
        )
        .await;
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["question_id"], "q1");
    assert_eq!(created["base64_data"], payload);
    assert_eq!(created["hash"], &id[..8]);
    assert!(created.get("version").is_none());

    let batch = h.ok(
        "create_attachments_for_question",
        json!({
            "questionId": "q2",
            "attachments": [
                { "question_id": "ignored", "type_": "original", "file_type": "img", "base64_data": "b".repeat(120) },
                { "question_id": "ignored", "type_": "answer", "file_type": "img", "base64_data": "c".repeat(120) }
            ]
        }),
    ).await;
    assert_eq!(batch.as_array().unwrap().len(), 2);
    assert!(batch
        .as_array()
        .unwrap()
        .iter()
        .all(|attachment| attachment["question_id"] == "q2"));
    assert_eq!(
        h.ok("get_attachments_by_question", json!({ "questionId": "q2" }))
            .await,
        batch
    );

    h.ok("delete_attachment", json!({ "id": id })).await;
    assert_eq!(
        h.ok("get_attachments_by_question", json!({ "questionId": "q1" }))
            .await,
        json!([])
    );
    assert_eq!(
        h.err("delete_attachment", json!({ "id": "missing" })).await,
        "Attachment not found"
    );

    h.ok(
        "upsert_attachment",
        json!({
            "input": {
                "id": "remote-attachment", "version": 2, "status": "ignored",
                "question_id": "q3", "type_": "answer", "file_type": "img",
                "base64_data": [255, 254], "hash": "remote-hash"
            }
        }),
    )
    .await;
    assert_eq!(
        h.ok("get_attachments_by_question", json!({ "questionId": "q3" }))
            .await[0]["base64_data"],
        ""
    );
    h.ok(
        "upsert_attachment",
        json!({
            "input": {
                "id": "remote-attachment", "version": 3, "status": "pending", "deleted_at": 88,
                "question_id": "q4", "type_": "original", "file_type": "img",
                "base64_data": [120], "hash": "new-hash"
            }
        }),
    )
    .await;
    let remote = entities::attachment::Entity::find_by_id("remote-attachment")
        .one(&h.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(remote.version, 3);
    assert_eq!(remote.sync_status, "synced");
    assert_eq!(remote.deleted_at, Some(88));
    assert_eq!(remote.base64_data, b"x");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn srs_commands_contract() {
    let h = Harness::new().await;
    let omission_h = Harness::new().await;
    let omitted_difficulty = omission_h
        .ok(
            "create_srs_data",
            json!({ "input": { "question_id": "omitted-difficulty" } }),
        )
        .await;
    let default_difficulty = omitted_difficulty["difficulty"].as_f64().unwrap();
    assert!((default_difficulty - crate::srs::config::INITIAL_DIFFICULTY as f64).abs() < 1e-5);
    assert_eq!(
        omission_h
            .ok("get_due_questions", json!({}))
            .await
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let wrong_top_level_name = h
        .err(
            "get_question_srs_status",
            json!({ "question_id": "missing" }),
        )
        .await;
    assert!(wrong_top_level_name.to_string().contains("questionId"));

    assert_eq!(
        h.ok(
            "get_question_srs_status",
            json!({ "questionId": "missing" })
        )
        .await,
        Value::Null
    );
    assert_eq!(h.ok("get_due_count", json!({})).await, 0);
    assert_eq!(
        h.ok("get_srs_statistics", json!({})).await,
        json!({
            "total": 0, "due_count": 0, "new_cards": 0,
            "avg_stability": 0.0, "avg_difficulty": 0.0, "total_reviews": 0
        })
    );
    assert_eq!(h.ok("get_all_cards", json!({})).await, json!([]));

    let created = h
        .ok(
            "create_srs_data",
            json!({ "input": { "question_id": "q1", "difficulty": null } }),
        )
        .await;
    assert_eq!(created["question_id"], "q1");
    assert_eq!(created["review_count"], 1);
    assert_eq!(created["is_due"], false);
    assert!(
        created["next_review_at"].as_i64().unwrap() > created["last_review_at"].as_i64().unwrap()
    );
    assert_eq!(
        h.err(
            "create_srs_data",
            json!({ "input": { "question_id": "q1", "difficulty": 8.0 } })
        )
        .await,
        "SrsData is exist: q1"
    );

    let reset = h
        .ok("reset_srs_progress", json!({ "questionId": "q1" }))
        .await;
    assert_eq!(reset["review_count"], 1);
    assert_eq!(reset["is_due"], true);

    h.ok(
        "upsert_srs_data",
        json!({
            "input": {
                "id": "remote-srs", "version": 4, "status": "ignored",
                "question_id": "q2", "stability": 1.0, "difficulty": 7.0,
                "review_count": 3, "feedback_history": "[0.5]"
            }
        }),
    )
    .await;
    let remote_insert = entities::srs_data::Entity::find_by_id("remote-srs")
        .one(&h.db)
        .await
        .unwrap()
        .unwrap();
    // Insert currently ignores the incoming deleted_at, unlike the update path.
    assert_eq!(remote_insert.deleted_at, None);
    assert_eq!(remote_insert.sync_status, "synced");

    let due = h.ok("get_due_questions", json!({ "limit": 1 })).await;
    assert_eq!(due.as_array().unwrap().len(), 1);
    assert_eq!(due[0]["question_id"], "q2");
    assert_eq!(due[0]["is_due"], true);
    assert_eq!(
        h.ok("get_due_questions", json!({}))
            .await
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(h.ok("get_due_count", json!({})).await, 2);

    assert_eq!(
        h.err(
            "submit_review_result",
            json!({ "input": { "question_id": "missing", "feedback": 0.5 } })
        )
        .await,
        "SRS data not found"
    );
    assert_eq!(
        h.err(
            "submit_review_result",
            json!({ "input": { "question_id": "q1", "feedback": 2.0 } })
        )
        .await,
        "Feedback must be in [0, 1], got 2"
    );
    let review = h
        .ok(
            "submit_review_result",
            json!({ "input": { "question_id": "q1", "feedback": 0.8 } }),
        )
        .await;
    assert!(review["next_interval_days"].as_f64().unwrap() > 0.0);
    let reviewed_status = h
        .ok("get_question_srs_status", json!({ "questionId": "q1" }))
        .await;
    assert_eq!(reviewed_status["review_count"], 2);
    let q1 = entities::srs_data::Entity::find()
        .filter(entities::srs_data::Column::QuestionId.eq("q1"))
        .one(&h.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(q1.feedback_history, "[0.8]");

    h.ok(
        "upsert_srs_data",
        json!({
            "input": {
                "id": "remote-srs", "version": 5, "status": "pending", "deleted_at": 456,
                "question_id": "q2", "stability": 2.0, "difficulty": 6.0,
                "next_review_at": null, "lastreviewed_at": null,
                "review_count": 4, "feedback_history": "[0.5,0.6]"
            }
        }),
    )
    .await;
    assert_eq!(
        entities::srs_data::Entity::find_by_id("remote-srs")
            .one(&h.db)
            .await
            .unwrap()
            .unwrap()
            .deleted_at,
        Some(456)
    );
    assert_eq!(
        h.ok("get_all_cards", json!({}))
            .await
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let stats = h.ok("get_srs_statistics", json!({})).await;
    assert_eq!(stats["total"], 1);
    assert_eq!(stats["total_reviews"], 2);

    let newly_reset = h
        .ok(
            "reset_srs_progress",
            json!({ "questionId": "new-question" }),
        )
        .await;
    assert_eq!(newly_reset["question_id"], "new-question");
    assert_eq!(newly_reset["is_due"], false);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sync_commands_contract() {
    let h = Harness::new().await;
    let subject = h
        .ok(
            "create_subject",
            json!({ "input": { "name": "Subject", "color": null } }),
        )
        .await;
    let subject_id = subject["id"].as_str().unwrap().to_owned();
    let question = h
        .ok(
            "create_question",
            json!({
                "input": {
                    "user_id": "u", "subject_id": subject_id, "source_id": null,
                    "prompt": "p", "type": "short", "answer": null,
                    "analysis": null, "error_note": null
                }
            }),
        )
        .await;
    let question_id = question["id"].as_str().unwrap().to_owned();
    h.ok(
        "create_source",
        json!({ "input": { "subject_id": subject_id, "book": "B", "chapter": null, "knowledge": null } }),
    ).await;
    h.ok(
        "create_error_tags_for_question",
        json!({ "input": { "question_id": question_id, "tags": [{ "name": "T", "color": "red" }] } }),
    ).await;
    h.ok(
        "create_attachment",
        json!({
            "input": {
                "question_id": question_id, "type_": "original", "file_type": "img",
                "base64_data": "z".repeat(120)
            }
        }),
    )
    .await;
    h.ok(
        "create_srs_data",
        json!({ "input": { "question_id": question_id, "difficulty": null } }),
    )
    .await;

    let pending = h.ok("get_all_pending_records", json!({})).await;
    assert_eq!(pending.as_array().unwrap().len(), 6);
    let table_order: Vec<_> = pending
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["table_name"].as_str().unwrap())
        .collect();
    assert_eq!(
        table_order,
        [
            "error_questions",
            "subjects",
            "srs_data",
            "attachments",
            "error_tags",
            "sources"
        ]
    );
    for record in pending.as_array().unwrap() {
        assert!(record["data"].get("version").is_none());
        assert!(record["data"].get("sync_status").is_none());
        assert!(record["data"].get("created_at").is_none());
        assert!(record["data"].get("id").is_some());
    }

    let headers = h.ok("get_all_records", json!({})).await;
    assert_eq!(headers.as_array().unwrap().len(), 6);
    assert!(headers[0].get("data").is_none());
    assert!(headers[0].get("created_at").is_some());

    let upload = h
        .ok("get_record_for_upload", json!({ "recordId": question_id }))
        .await;
    assert_eq!(upload["table_name"], "error_questions");
    assert_eq!(upload["data"]["prompt"], "p");
    assert_eq!(
        h.err("get_record_for_upload", json!({ "recordId": "missing" }))
            .await,
        "Record not found with id: missing"
    );

    let status_result = h
        .ok(
            "set_record_sync_status_version",
            json!({ "recordId": subject_id, "status": "synced", "version": 9 }),
        )
        .await;
    assert!(status_result.as_str().unwrap().contains(&subject_id));
    let stored_subject = entities::subject::Entity::find_by_id(&subject_id)
        .one(&h.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_subject.sync_status, "synced");
    assert_eq!(stored_subject.version, 9);
    assert_eq!(
        h.err(
            "set_record_sync_status_version",
            json!({ "recordId": "missing", "status": "synced", "version": 1 })
        )
        .await,
        "Record not found with id: missing"
    );

    // When IDs collide across tables, lookup follows the documented table order.
    h.ok(
        "upsert_subject",
        json!({
            "input": {
                "id": question_id, "version": 1, "status": "ignored",
                "deleted_at": null, "name": "Collision", "color": null
            }
        }),
    )
    .await;
    assert_eq!(
        h.ok("get_record_for_upload", json!({ "recordId": question_id }))
            .await["table_name"],
        "error_questions"
    );

    h.ok("delete_subject", json!({ "id": subject_id })).await;
    h.ok(
        "set_record_sync_status_version",
        json!({ "recordId": subject_id, "status": "synced", "version": 10 }),
    )
    .await;
    let purge = h.ok("purge_synced_deletions", json!({})).await;
    assert_eq!(purge["subjects"]["deleted"], 1);
    for table in [
        "error_questions",
        "srs_data",
        "attachments",
        "error_tags",
        "sources",
    ] {
        assert_eq!(purge[table]["deleted"], 0);
    }
    assert!(entities::subject::Entity::find_by_id(&subject_id)
        .one(&h.db)
        .await
        .unwrap()
        .is_none());

    let orphan_h = Harness::new().await;
    orphan_h
        .ok(
            "create_question",
            json!({
                "input": {
                    "user_id": "u", "subject_id": "missing-subject", "source_id": null,
                    "prompt": "orphan", "type": "short", "answer": null,
                    "analysis": null, "error_note": null
                }
            }),
        )
        .await;
    orphan_h.ok(
        "create_source",
        json!({ "input": { "subject_id": "missing-subject", "book": "B", "chapter": null, "knowledge": null } }),
    ).await;
    orphan_h
        .ok(
            "create_srs_data",
            json!({ "input": { "question_id": "missing-question", "difficulty": null } }),
        )
        .await;
    orphan_h.ok(
        "create_error_tags_for_question",
        json!({ "input": { "question_id": "missing-question", "tags": [{ "name": "T", "color": "red" }] } }),
    ).await;
    orphan_h
        .ok(
            "create_attachment",
            json!({
                "input": {
                    "question_id": "missing-question", "type_": "original", "file_type": "img",
                    "base64_data": "o".repeat(120)
                }
            }),
        )
        .await;
    let orphan_result = orphan_h.ok("check_orphan_records", json!({})).await;
    assert_eq!(orphan_result["total_checked"], 5);
    let reported: std::collections::HashSet<_> = orphan_result["orphan_records_soft_deleted"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().split(':').next().unwrap())
        .collect();
    assert_eq!(
        reported,
        ["source", "srs_data", "error_tag", "attachment"].into()
    );
    let remapped = entities::error_question::Entity::find()
        .one(&orphan_h.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(remapped.subjectid, "");
    assert!(remapped.deleted_at.is_none());
    for deleted_at in [
        entities::source::Entity::find()
            .one(&orphan_h.db)
            .await
            .unwrap()
            .unwrap()
            .deleted_at,
        entities::srs_data::Entity::find()
            .one(&orphan_h.db)
            .await
            .unwrap()
            .unwrap()
            .deleted_at,
        entities::error_tag::Entity::find()
            .one(&orphan_h.db)
            .await
            .unwrap()
            .unwrap()
            .deleted_at,
        entities::attachment::Entity::find()
            .one(&orphan_h.db)
            .await
            .unwrap()
            .unwrap()
            .deleted_at,
    ] {
        assert!(deleted_at.is_some());
    }
}

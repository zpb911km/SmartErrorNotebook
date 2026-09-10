use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbConn};
use serde_json::{json, Value};
use tauri::{test::MockRuntime, WebviewWindow};

use crate::{command::CommandRegistry, data::database::connection::init_database, AppState};

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
            .expect("connect test database");
        init_database(&db).await.expect("migrate test database");

        let app = tauri::test::mock_builder()
            .manage(AppState {
                repository_transaction_executor:
                    crate::data::SeaOrmRepositoryTransactionExecutor::new(db.clone()),
            })
            .register_command()
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
        .expect("IPC task panicked")
    }

    async fn ok(&self, command: &str, args: Value) -> Value {
        self.invoke(command, args)
            .await
            .unwrap_or_else(|error| panic!("{command} failed: {error}"))
    }

    async fn err(&self, command: &str, args: Value) -> Value {
        match self.invoke(command, args).await {
            Ok(value) => panic!("{command} unexpectedly succeeded: {value}"),
            Err(error) => error,
        }
    }
}

fn assert_pending(model: &Value) {
    assert_eq!(model["version"], 0);
    assert_eq!(model["sync_status"], "pending");
    assert!(model["created_at"].as_i64().unwrap() > 0);
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn string_set(value: &Value) -> std::collections::HashSet<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_str().unwrap().to_owned())
        .collect()
}

async fn create_subject_and_question(harness: &Harness) -> (String, String) {
    let subject = harness
        .ok(
            "legacy_create_subject",
            json!({"input":{"name":"Math","color":"blue"}}),
        )
        .await;
    let subject_id = subject["id"].as_str().unwrap().to_owned();
    let question = harness
        .ok(
            "legacy_create_question",
            json!({"input":{
                "user_id":"user", "subject_id":subject_id, "source_id":null,
                "prompt":"2 + 2", "type":"简答题", "answer":"4",
                "analysis":null, "error_note":null
            }}),
        )
        .await;
    (subject_id, question["id"].as_str().unwrap().to_owned())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subject_source_and_question_commands_use_normalized_models() {
    let harness = Harness::new().await;
    let subject = harness
        .ok(
            "legacy_create_subject",
            json!({"input":{"name":"Math","color":"blue"}}),
        )
        .await;
    assert_pending(&subject);
    let subject_id = subject["id"].as_str().unwrap().to_owned();

    let source = harness
        .ok(
            "legacy_create_source",
            json!({"input":{"subject_id":subject_id,"book":"Book","chapter":"1","knowledge":"Algebra"}}),
        )
        .await;
    let source_id = source["id"].as_str().unwrap().to_owned();
    let question = harness
        .ok(
            "legacy_create_question",
            json!({"input":{
                "user_id":"user", "subject_id":subject_id, "source_id":source_id,
                "prompt":"x + 1 = 2", "type":"简答题", "answer":"1",
                "analysis":"subtract one", "error_note":null
            }}),
        )
        .await;
    assert_pending(&question);
    assert_eq!(question["subject_id"], subject_id);
    assert_eq!(question["source_id"], source_id);

    let filtered = harness
        .ok(
            "legacy_get_sources",
            json!({"filter":{"subject_id":subject_id}}),
        )
        .await;
    assert_eq!(filtered.as_array().unwrap().len(), 1);
    assert_eq!(filtered[0]["book"], "Book");
    assert_eq!(filtered[0]["subject_id"], subject_id);
    assert_eq!(filtered[0]["question_id"], question["id"]);

    let unfiltered = harness.ok("legacy_get_sources", json!({"filter":{}})).await;
    assert_eq!(unfiltered.as_array().unwrap().len(), 1);
    assert_eq!(unfiltered[0]["subject_id"], subject_id);
    assert_eq!(unfiltered[0]["question_id"], question["id"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn attachment_tag_and_srs_commands_persist_relations() {
    let harness = Harness::new().await;
    let (_, question_id) = create_subject_and_question(&harness).await;

    let attachment = harness
        .ok(
            "legacy_create_attachment",
            json!({"input":{
                "question_id":question_id,"type":"answer","file_type":"image/png","base64_data":"aW1hZ2U="
            }}),
        )
        .await;
    assert_eq!(attachment["type"], "original");
    assert_eq!(attachment["type_"], "original");
    let listed = harness
        .ok(
            "legacy_get_attachments_by_question",
            json!({"questionId":question_id}),
        )
        .await;
    assert_eq!(listed.as_array().unwrap().len(), 1);

    let tags = harness
        .ok(
            "legacy_create_error_tags_for_question",
            json!({"input":{"question_id":question_id,"tags":[{"name":"Arithmetic","color":"red"}]}}),
        )
        .await;
    assert_eq!(tags[0]["question_id"], question_id);
    let listed_tags = harness
        .ok(
            "legacy_get_error_tags_for_question",
            json!({"questionId":question_id}),
        )
        .await;
    assert_eq!(listed_tags.as_array().unwrap().len(), 1);

    let card = harness
        .ok(
            "legacy_create_srs_data",
            json!({"input":{"question_id":question_id}}),
        )
        .await;
    assert_eq!(card["question_id"], question_id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sync_commands_keep_legacy_wire_shape() {
    let harness = Harness::new().await;
    let (subject_id, question_id) = create_subject_and_question(&harness).await;
    harness
        .ok(
            "legacy_create_srs_data",
            json!({"input":{"question_id":question_id}}),
        )
        .await;
    let headers = harness.ok("legacy_get_all_records", json!({})).await;
    assert!(headers
        .as_array()
        .unwrap()
        .iter()
        .any(|header| header["id"] == subject_id));
    assert!(headers.as_array().unwrap().iter().any(|header| {
        header["id"] == question_id && header["table_name"] == "error_questions"
    }));
    assert!(headers
        .as_array()
        .unwrap()
        .iter()
        .any(|header| { header["id"] == question_id && header["table_name"] == "srs_data" }));

    let pending = harness
        .ok("legacy_get_all_pending_records", json!({}))
        .await;
    assert!(pending
        .as_array()
        .unwrap()
        .iter()
        .all(|record| record["status"] == "pending"));
    let upload = harness
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":question_id,"tableName":"error_questions"}),
        )
        .await;
    assert_eq!(upload["table_name"], "error_questions");
    assert!(upload["data"].get("sync_status").is_none());
    let srs_upload = harness
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":question_id,"tableName":"srs_data"}),
        )
        .await;
    assert_eq!(srs_upload["table_name"], "srs_data");
    harness
        .ok(
            "legacy_set_record_sync_status_version",
            json!({"recordId":question_id,"tableName":"srs_data","status":"synced","version":9}),
        )
        .await;
    let headers = harness.ok("legacy_get_all_records", json!({})).await;
    assert!(headers.as_array().unwrap().iter().any(|header| {
        header["id"] == question_id && header["table_name"] == "srs_data" && header["version"] == 9
    }));
    assert!(headers.as_array().unwrap().iter().any(|header| {
        header["id"] == question_id
            && header["table_name"] == "error_questions"
            && header["version"] != 9
    }));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subject_source_and_question_command_contract_is_complete() {
    let harness = Harness::new().await;
    let (subject_id, question_id) = create_subject_and_question(&harness).await;

    let subjects = harness.ok("legacy_get_subjects", json!({})).await;
    assert_eq!(subjects.as_array().unwrap().len(), 1);
    let updated_subject = harness
        .ok(
            "legacy_update_subject",
            json!({"input":{"id":subject_id,"name":"Mathematics","color":"navy"}}),
        )
        .await;
    assert_eq!(updated_subject["name"], "Mathematics");

    let source = harness
        .ok(
            "legacy_create_source",
            json!({"input":{"subject_id":subject_id,"book":"Book A","chapter":"C1","knowledge":"K1"}}),
        )
        .await;
    let source_id = source["id"].as_str().unwrap().to_owned();
    assert_eq!(
        harness
            .ok("legacy_get_source", json!({"id":source_id}))
            .await["book"],
        "Book A"
    );
    harness
        .ok(
            "legacy_update_question",
            json!({"input":{"id":question_id,"source_id":source_id}}),
        )
        .await;
    assert_eq!(
        harness
            .ok("legacy_get_books", json!({"subjectId":subject_id}))
            .await,
        json!(["Book A"])
    );
    assert_eq!(
        harness
            .ok(
                "legacy_get_chapters",
                json!({"subjectId":subject_id,"book":"Book A"}),
            )
            .await,
        json!(["C1"])
    );
    assert_eq!(
        harness
            .ok(
                "legacy_get_knowledges",
                json!({"subjectId":subject_id,"book":"Book A","chapter":"C1"}),
            )
            .await,
        json!(["K1"])
    );
    assert_eq!(
        harness
            .ok(
                "legacy_get_or_create_source_id",
                json!({"input":{"subject_id":subject_id,"book":"Book A","chapter":"C1","knowledge":"K1"}}),
            )
            .await,
        source_id
    );
    let updated_source = harness
        .ok(
            "legacy_update_source",
            json!({"input":{"id":source_id,"subject_id":subject_id,"book":"Book B","chapter":"C2","knowledge":"K2"}}),
        )
        .await;
    assert_eq!(updated_source["book"], "Book B");

    let question = harness
        .ok("legacy_get_question", json!({"id":question_id}))
        .await;
    assert_eq!(question["id"], question_id);
    assert_eq!(question["subjectid"], subject_id);
    assert_eq!(question["type_"], "简答题");
    let updated_question = harness
        .ok(
            "legacy_update_question",
            json!({"input":{"id":question_id,"prompt":"updated prompt","type":"论述题"}}),
        )
        .await;
    assert_eq!(updated_question["prompt"], "updated prompt");
    assert_eq!(updated_question["type_"], "论述题");
    assert_eq!(
        harness
            .ok(
                "legacy_get_questions",
                json!({"filter":{"subject_id":subject_id,"search":"updated"}}),
            )
            .await
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        harness.ok("legacy_get_question_stats", json!({})).await["total"],
        1
    );

    harness
        .ok("legacy_delete_question", json!({"id":question_id}))
        .await;
    harness
        .ok("legacy_delete_source", json!({"id":source_id}))
        .await;
    harness
        .ok("legacy_delete_subject", json!({"id":subject_id}))
        .await;
    assert_eq!(
        harness.ok("legacy_get_subjects", json!({})).await,
        json!([])
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn attachment_tag_and_srs_tool_commands_keep_legacy_shapes() {
    let harness = Harness::new().await;
    let (_, question_id) = create_subject_and_question(&harness).await;

    let attachments = harness
        .ok(
            "legacy_create_attachments_for_question",
            json!({"questionId":question_id,"attachments":[
                {"question_id":question_id,"type_":"original","file_type":"image/png","base64_data":"AQID"},
                {"question_id":question_id,"type":"answer","file_type":"image/jpeg","base64_data":"/9j/2Q=="}
            ]}),
        )
        .await;
    assert_eq!(attachments.as_array().unwrap().len(), 2);
    let first_attachment_id = attachments[0]["id"].as_str().unwrap().to_owned();
    harness
        .ok(
            "legacy_delete_attachment",
            json!({"id":first_attachment_id,"questionId":question_id}),
        )
        .await;
    let synced_attachment_id = new_id();
    harness
        .ok(
            "legacy_upsert_attachment",
            json!({"input":{
                "id":synced_attachment_id,"version":3,"status":"pending","deleted_at":null,
                "question_id":question_id,"type_":"answer","file_type":"image/png",
                "base64_data":[65,81,73,68],"hash":"hash"
            }}),
        )
        .await;
    let active_attachments = harness
        .ok(
            "legacy_get_attachments_by_question",
            json!({"questionId":question_id}),
        )
        .await;
    assert_eq!(active_attachments.as_array().unwrap().len(), 2);
    let synced_attachment = active_attachments
        .as_array()
        .unwrap()
        .iter()
        .find(|attachment| attachment["id"] == synced_attachment_id)
        .unwrap();
    assert_eq!(synced_attachment["base64_data"], "AQID");
    assert_eq!(synced_attachment["file_type"], "png");
    assert_eq!(synced_attachment["hash"].as_str().unwrap().len(), 64);
    assert_ne!(synced_attachment["hash"], "hash");
    let synced_upload = harness
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":synced_attachment_id,"tableName":"attachments"}),
        )
        .await;
    assert_eq!(
        synced_upload["data"]["base64_data"],
        json!([65, 81, 73, 68])
    );
    assert_eq!(synced_upload["data"]["file_type"], "png");
    assert_eq!(synced_upload["data"]["hash"].as_str().unwrap().len(), 64);

    let tags = harness
        .ok(
            "legacy_create_error_tags_for_question",
            json!({"input":{"question_id":question_id,"tags":[
                {"name":"Arithmetic","color":"red"},
                {"name":"Concept","color":"blue"}
            ]}}),
        )
        .await;
    let arithmetic_id = tags[0]["id"].as_str().unwrap().to_owned();
    let concept_id = tags[1]["id"].as_str().unwrap().to_owned();
    harness
        .ok(
            "legacy_update_error_tag_by_name",
            json!({"oldName":"Arithmetic","newName":"Calculation","newColor":"orange"}),
        )
        .await;
    harness
        .ok(
            "legacy_update_error_tag_by_id",
            json!({"tagId":concept_id,"newTagName":"Theory","newTagColor":"purple"}),
        )
        .await;
    assert_eq!(
        harness
            .ok("legacy_get_error_tags", json!({}))
            .await
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let full_tags = harness.ok("legacy_get_full_error_tags", json!({})).await;
    assert_eq!(full_tags.as_array().unwrap().len(), 2);
    assert!(full_tags
        .as_array()
        .unwrap()
        .iter()
        .all(|tag| tag["question_id"] == question_id));
    harness
        .ok(
            "legacy_delete_error_tag",
            json!({"tagId":arithmetic_id,"questionId":question_id}),
        )
        .await;
    let synced_tag_id = new_id();
    harness
        .ok(
            "legacy_upsert_error_tag",
            json!({"input":{
                "id":synced_tag_id,"version":2,"status":"pending","deleted_at":null,
                "question_id":question_id,"name":"Synced","color":"green"
            }}),
        )
        .await;

    harness
        .ok(
            "legacy_create_srs_data",
            json!({"input":{"question_id":question_id,"difficulty":0.4}}),
        )
        .await;
    assert!(harness
        .ok(
            "legacy_get_question_srs_status",
            json!({"questionId":question_id}),
        )
        .await
        .is_object());
    assert_eq!(
        harness
            .ok("legacy_get_all_cards", json!({}))
            .await
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(harness.ok("legacy_get_due_count", json!({})).await, 0);
    assert_eq!(
        harness
            .ok("legacy_get_due_questions", json!({"limit":10}))
            .await,
        json!([])
    );
    let statistics = harness.ok("legacy_get_srs_statistics", json!({})).await;
    assert_eq!(statistics["total"], 1);
    harness
        .ok(
            "legacy_reset_srs_progress",
            json!({"questionId":question_id}),
        )
        .await;
    let review = harness
        .ok(
            "legacy_submit_review_result",
            json!({"input":{"question_id":question_id,"feedback":0.8}}),
        )
        .await;
    assert!(review["next_review_at"].as_i64().unwrap() > 0);
    harness
        .ok(
            "legacy_upsert_srs_data",
            json!({"input":{
                "id":question_id,"version":4,"status":"pending","deleted_at":null,
                "question_id":question_id,"stability":3.0,"difficulty":0.3,
                "next_review_at":null,"lastreviewed_at":null,"review_count":5,
                "feedback_history":"[]"
            }}),
        )
        .await;
    assert_eq!(
        harness
            .ok(
                "legacy_get_question_srs_status",
                json!({"questionId":question_id}),
            )
            .await["review_count"],
        5
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn attachments_and_tags_preserve_many_to_many_relations() {
    let harness = Harness::new().await;
    let (subject_id, first_question_id) = create_subject_and_question(&harness).await;
    let second_question = harness
        .ok(
            "legacy_create_question",
            json!({"input":{
                "user_id":"user", "subject_id":subject_id, "source_id":null,
                "prompt":"3 + 3", "type":"简答题", "answer":"6",
                "analysis":null, "error_note":null
            }}),
        )
        .await;
    let second_question_id = second_question["id"].as_str().unwrap().to_owned();

    let attachment_id = new_id();
    harness
        .ok(
            "legacy_upsert_attachment",
            json!({"input":{
                "id":attachment_id,"version":3,"status":"pending","deleted_at":null,
                "question_ids":[first_question_id,second_question_id],
                "type_":"answer","file_type":"image/png","base64_data":[65,81,73,68],"hash":"ignored"
            }}),
        )
        .await;
    let tag_id = new_id();
    harness
        .ok(
            "legacy_upsert_error_tag",
            json!({"input":{
                "id":tag_id,"version":2,"status":"pending","deleted_at":null,
                "question_ids":[first_question_id,second_question_id],
                "name":"Shared","color":"green"
            }}),
        )
        .await;

    for question_id in [&first_question_id, &second_question_id] {
        assert_eq!(
            harness
                .ok(
                    "legacy_get_attachments_by_question",
                    json!({"questionId":question_id}),
                )
                .await
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            harness
                .ok(
                    "legacy_get_error_tags_for_question",
                    json!({"questionId":question_id}),
                )
                .await
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    let expected_question_ids = {
        let mut ids = vec![first_question_id.clone(), second_question_id.clone()];
        ids.sort();
        json!(ids)
    };
    assert_eq!(
        harness
            .ok(
                "legacy_get_record_for_upload",
                json!({"recordId":attachment_id,"tableName":"attachments"}),
            )
            .await["data"]["question_ids"],
        expected_question_ids
    );
    assert_eq!(
        harness
            .ok(
                "legacy_get_record_for_upload",
                json!({"recordId":tag_id,"tableName":"error_tags"}),
            )
            .await["data"]["question_ids"],
        expected_question_ids
    );

    harness
        .ok(
            "legacy_delete_attachment",
            json!({"id":attachment_id,"questionId":first_question_id}),
        )
        .await;
    harness
        .ok(
            "legacy_delete_error_tag",
            json!({"tagId":tag_id,"questionId":first_question_id}),
        )
        .await;
    assert_eq!(
        harness
            .ok(
                "legacy_get_attachments_by_question",
                json!({"questionId":second_question_id}),
            )
            .await
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        harness
            .ok(
                "legacy_get_error_tags_for_question",
                json!({"questionId":second_question_id}),
            )
            .await
            .as_array()
            .unwrap()
            .len(),
        1
    );

    harness
        .ok(
            "legacy_delete_attachment",
            json!({"id":attachment_id,"questionId":second_question_id}),
        )
        .await;
    harness
        .ok(
            "legacy_delete_error_tag",
            json!({"tagId":tag_id,"questionId":second_question_id}),
        )
        .await;
    let deleted_attachment = harness
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":attachment_id,"tableName":"attachments"}),
        )
        .await;
    let deleted_tag = harness
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":tag_id,"tableName":"error_tags"}),
        )
        .await;
    assert!(!deleted_attachment["deleted_at"].is_null());
    assert!(!deleted_tag["deleted_at"].is_null());
    assert_eq!(deleted_attachment["data"]["question_ids"], json!([]));
    assert_eq!(deleted_tag["data"]["question_ids"], json!([]));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sync_upsert_status_cleanup_and_orphan_commands_are_registered() {
    let harness = Harness::new().await;
    let (subject_id, question_id) = create_subject_and_question(&harness).await;

    let synced_subject_id = new_id();
    harness
        .ok(
            "legacy_upsert_subject",
            json!({"input":{"id":synced_subject_id,"version":2,"status":"pending","name":"Remote","color":null}}),
        )
        .await;
    let synced_question_id = new_id();
    harness
        .ok(
            "legacy_upsert_error_question",
            json!({"input":{
                "id":synced_question_id,"version":2,"status":"pending","deleted_at":null,
                "userid":"remote","subjectid":subject_id,"sourceid":null,
                "prompt":"remote question","type_":"简答题","answer":"answer",
                "analysis":null,"error_note":null,"sync_hash":"hash"
            }}),
        )
        .await;
    let synced_source_id = new_id();
    harness
        .ok(
            "legacy_upsert_source",
            json!({"input":{
                "id":synced_source_id,"version":2,"status":"pending","deleted_at":null,
                "question_id":question_id,"subject_id":subject_id,
                "book":"Remote Book","chapter":"Remote Chapter","knowledge":"Remote Knowledge"
            }}),
        )
        .await;

    let status_result = harness
        .ok(
            "legacy_set_record_sync_status_version",
            json!({"recordId":subject_id,"tableName":"subjects","status":"synced","version":7}),
        )
        .await;
    assert!(status_result.as_str().unwrap().contains(&subject_id));
    let pending = harness
        .ok("legacy_get_all_pending_records", json!({}))
        .await;
    assert!(!pending
        .as_array()
        .unwrap()
        .iter()
        .any(|record| record["id"] == subject_id));

    let orphan_result = harness.ok("legacy_check_orphan_records", json!({})).await;
    assert!(orphan_result["total_checked"].is_number());

    harness
        .ok("legacy_delete_subject", json!({"id":subject_id}))
        .await;
    harness
        .ok(
            "legacy_set_record_sync_status_version",
            json!({"recordId":subject_id,"tableName":"subjects","status":"synced","version":8}),
        )
        .await;
    let purge = harness.ok("legacy_purge_synced_deletions", json!({})).await;
    assert!(purge.is_object());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subject_commands_contract() {
    let h = Harness::new().await;
    assert_eq!(h.ok("legacy_get_subjects", json!({})).await, json!([]));

    let created = h
        .ok(
            "legacy_create_subject",
            json!({"input":{"name":"Mathematics","color":"#123456"}}),
        )
        .await;
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["name"], "Mathematics");
    assert_eq!(created["color"], "#123456");
    assert_pending(&created);

    let updated = h
        .ok(
            "legacy_update_subject",
            json!({"input":{"id":id,"name":"Math","color":null}}),
        )
        .await;
    assert_eq!(updated["name"], "Math");
    assert_eq!(updated["color"], "#123456");
    let unchanged = h
        .ok("legacy_update_subject", json!({"input":{"id":id}}))
        .await;
    assert_eq!(unchanged["name"], "Math");

    let missing = new_id();
    assert_eq!(
        h.err(
            "legacy_update_subject",
            json!({"input":{"id":missing,"name":"missing"}}),
        )
        .await,
        "Subject not found"
    );
    h.ok("legacy_delete_subject", json!({"id":id})).await;
    assert_eq!(h.ok("legacy_get_subjects", json!({})).await, json!([]));

    let remote_id = new_id();
    h.ok(
        "legacy_upsert_subject",
        json!({"input":{"id":remote_id,"version":7,"status":"ignored","name":"Physics"}}),
    )
    .await;
    h.ok(
        "legacy_upsert_subject",
        json!({"input":{"id":remote_id,"version":8,"status":"pending","deleted_at":99,"name":"Modern Physics","color":"blue"}}),
    )
    .await;
    let upload = h
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":remote_id,"tableName":"subjects"}),
        )
        .await;
    assert_eq!(upload["version"], 8);
    assert_eq!(upload["status"], "synced");
    assert_eq!(upload["deleted_at"], 99);
    assert_eq!(upload["data"]["name"], "Modern Physics");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn error_question_commands_contract() {
    let h = Harness::new().await;
    let (subject_id, first_id) = create_subject_and_question(&h).await;
    let second = h
        .ok(
            "legacy_create_question",
            json!({"input":{
                "user_id":"u2","subject_id":subject_id,"source_id":null,
                "prompt":"geometry quadratic","type":"选择题","answer":null,
                "analysis":"quadratic appears here","error_note":null
            }}),
        )
        .await;
    let second_id = second["id"].as_str().unwrap().to_owned();
    assert_eq!(
        h.ok("legacy_get_questions", json!({}))
            .await
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let search = h
        .ok(
            "legacy_get_questions",
            json!({"filter":{"subject_id":subject_id,"search":"quadratic"}}),
        )
        .await;
    assert_eq!(search.as_array().unwrap().len(), 1);
    assert_eq!(search[0]["id"], second_id);

    assert_eq!(
        h.ok("legacy_get_question", json!({"id":first_id})).await["id"],
        first_id
    );
    let updated = h
        .ok(
            "legacy_update_question",
            json!({"input":{"id":first_id,"prompt":"updated","type":"论述题","answer":null}}),
        )
        .await;
    assert_eq!(updated["prompt"], "updated");
    assert_eq!(updated["answer"], "4");

    h.ok(
        "legacy_create_srs_data",
        json!({"input":{"question_id":first_id}}),
    )
    .await;
    h.ok("legacy_delete_question", json!({"id":first_id})).await;
    assert_eq!(
        h.ok("legacy_get_question_stats", json!({})).await,
        json!({"total":1})
    );
    assert!(!h.ok("legacy_get_question", json!({"id":first_id})).await["deleted_at"].is_null());

    let srs_upload = h
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":first_id,"tableName":"srs_data"}),
        )
        .await;
    assert!(!srs_upload["deleted_at"].is_null());

    let remote_id = new_id();
    h.ok(
        "legacy_upsert_error_question",
        json!({"input":{
            "id":remote_id,"version":4,"status":"pending","deleted_at":123,
            "userid":"remote-user","subjectid":subject_id,"sourceid":null,
            "prompt":"remote-updated","type_":"论述题","answer":"x",
            "analysis":null,"error_note":null,"sync_hash":"hash"
        }}),
    )
    .await;
    let remote = h
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":remote_id,"tableName":"error_questions"}),
        )
        .await;
    assert_eq!(remote["version"], 4);
    assert_eq!(remote["status"], "synced");
    assert_eq!(remote["data"]["prompt"], "remote-updated");
    assert!(remote["data"]["sourceid"].is_null());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn synced_question_tombstone_unlinks_and_tombstones_dependents() {
    let h = Harness::new().await;
    let (subject_id, question_id) = create_subject_and_question(&h).await;
    let source_id = h.ok("legacy_get_question", json!({"id":question_id})).await["source_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let attachment = h
        .ok(
            "legacy_create_attachment",
            json!({"input":{
                "question_id":question_id,"type":"original",
                "file_type":"image/png","base64_data":"aW1hZ2U="
            }}),
        )
        .await;
    let attachment_id = attachment["id"].as_str().unwrap().to_owned();
    let tags = h
        .ok(
            "legacy_create_error_tags_for_question",
            json!({"input":{"question_id":question_id,"tags":[
                {"name":"Arithmetic","color":"red"}
            ]}}),
        )
        .await;
    let tag_id = tags[0]["id"].as_str().unwrap().to_owned();
    h.ok(
        "legacy_create_srs_data",
        json!({"input":{"question_id":question_id}}),
    )
    .await;

    h.ok(
        "legacy_upsert_error_question",
        json!({"input":{
            "id":question_id,"version":7,"status":"ignored","deleted_at":1700000000,
            "userid":"remote-user","subjectid":subject_id,"sourceid":null,
            "prompt":"deleted remotely","type_":"简答题","answer":"4",
            "analysis":null,"error_note":null,"sync_hash":"hash"
        }}),
    )
    .await;

    assert_eq!(h.ok("legacy_get_questions", json!({})).await, json!([]));
    assert_eq!(
        h.ok(
            "legacy_get_attachments_by_question",
            json!({"questionId":question_id})
        )
        .await,
        json!([])
    );
    assert_eq!(
        h.ok(
            "legacy_get_error_tags_for_question",
            json!({"questionId":question_id})
        )
        .await,
        json!([])
    );
    assert_eq!(
        h.ok(
            "legacy_get_question_srs_status",
            json!({"questionId":question_id})
        )
        .await,
        Value::Null
    );

    let question_upload = h
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":question_id,"tableName":"error_questions"}),
        )
        .await;
    assert_eq!(question_upload["status"], "synced");
    assert_eq!(question_upload["version"], 7);
    assert_eq!(question_upload["deleted_at"], 1700000000);
    assert_eq!(question_upload["data"]["sourceid"], source_id);

    for (id, table_name) in [
        (&question_id, "srs_data"),
        (&attachment_id, "attachments"),
        (&tag_id, "error_tags"),
    ] {
        let upload = h
            .ok(
                "legacy_get_record_for_upload",
                json!({"recordId":id,"tableName":table_name}),
            )
            .await;
        assert_eq!(upload["status"], "pending");
        assert!(!upload["deleted_at"].is_null());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn source_commands_contract() {
    let h = Harness::new().await;
    assert_eq!(h.ok("legacy_get_sources", json!({})).await, json!([]));
    let first_subject = h
        .ok("legacy_create_subject", json!({"input":{"name":"First"}}))
        .await;
    let second_subject = h
        .ok("legacy_create_subject", json!({"input":{"name":"Second"}}))
        .await;
    let first_subject_id = first_subject["id"].as_str().unwrap().to_owned();
    let second_subject_id = second_subject["id"].as_str().unwrap().to_owned();

    let one = h
        .ok(
            "legacy_create_source",
            json!({"input":{"subject_id":first_subject_id,"book":"Book A","chapter":"C1","knowledge":"K1"}}),
        )
        .await;
    let one_id = one["id"].as_str().unwrap().to_owned();
    assert_pending(&one);
    let two = h
        .ok(
            "legacy_create_source",
            json!({"input":{"subject_id":first_subject_id,"book":"Book A","chapter":"C2","knowledge":"K2"}}),
        )
        .await;
    h.ok(
        "legacy_create_source",
        json!({"input":{"subject_id":second_subject_id,"book":"Book B","chapter":"C1","knowledge":null}}),
    )
    .await;

    assert_eq!(
        h.ok(
            "legacy_get_sources",
            json!({"filter":{"subject_id":first_subject_id}}),
        )
        .await
        .as_array()
        .unwrap()
        .len(),
        2
    );
    assert_eq!(
        string_set(&h.ok("legacy_get_books", json!({})).await),
        ["Book A".to_owned(), "Book B".to_owned()].into()
    );
    assert_eq!(
        string_set(
            &h.ok(
                "legacy_get_chapters",
                json!({"subjectId":first_subject_id,"book":"Book A"}),
            )
            .await,
        ),
        ["C1".to_owned(), "C2".to_owned()].into()
    );
    assert_eq!(
        h.ok(
            "legacy_get_or_create_source_id",
            json!({"input":{"subject_id":first_subject_id,"book":"Book A","chapter":"C2","knowledge":"K2"}}),
        )
        .await,
        two["id"]
    );

    let updated = h
        .ok(
            "legacy_update_source",
            json!({"input":{"id":one_id,"subject_id":null,"book":"Book A2","chapter":null,"knowledge":null}}),
        )
        .await;
    assert_eq!(updated["book"], "Book A2");
    assert_eq!(updated["subject_id"], first_subject_id);
    assert_eq!(updated["chapter"], "C1");
    h.ok("legacy_delete_source", json!({"id":one_id})).await;
    assert!(!h.ok("legacy_get_source", json!({"id":one_id})).await["deleted_at"].is_null());

    let remote_id = new_id();
    h.ok(
        "legacy_upsert_source",
        json!({"input":{
            "id":remote_id,"version":3,"status":"pending","deleted_at":77,
            "question_id":null,"subject_id":second_subject_id,
            "book":"Remote","chapter":"C","knowledge":"K"
        }}),
    )
    .await;
    let remote = h
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":remote_id,"tableName":"sources"}),
        )
        .await;
    assert_eq!(remote["version"], 3);
    assert_eq!(remote["status"], "synced");
    assert_eq!(remote["deleted_at"], 77);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn error_tag_commands_contract() {
    let h = Harness::new().await;
    let (_, question_id) = create_subject_and_question(&h).await;
    assert_eq!(h.ok("legacy_get_error_tags", json!({})).await, json!([]));
    let created = h
        .ok(
            "legacy_create_error_tags_for_question",
            json!({"input":{"question_id":question_id,"tags":[
                {"name":"Calculation","color":"red"},
                {"name":"Concept","color":"green"}
            ]}}),
        )
        .await;
    assert_eq!(created.as_array().unwrap().len(), 2);
    assert_eq!(
        h.ok(
            "legacy_get_error_tags_for_question",
            json!({"questionId":question_id})
        )
        .await
        .as_array()
        .unwrap()
        .len(),
        2
    );

    h.ok(
        "legacy_update_error_tag_by_name",
        json!({"oldName":"Calculation","newName":"Arithmetic","newColor":"orange"}),
    )
    .await;
    let concept_id = created[1]["id"].as_str().unwrap().to_owned();
    h.ok(
        "legacy_update_error_tag_by_id",
        json!({"tagId":concept_id,"newTagName":"Theory"}),
    )
    .await;
    let tags = h
        .ok(
            "legacy_get_error_tags_for_question",
            json!({"questionId":question_id}),
        )
        .await;
    assert!(tags
        .as_array()
        .unwrap()
        .iter()
        .any(|tag| tag["name"] == "Arithmetic" && tag["color"] == "orange"));
    assert!(tags
        .as_array()
        .unwrap()
        .iter()
        .any(|tag| tag["name"] == "Theory" && tag["color"] == "green"));

    let first_id = created[0]["id"].as_str().unwrap().to_owned();
    h.ok(
        "legacy_delete_error_tag",
        json!({"tagId":first_id,"questionId":question_id}),
    )
    .await;
    assert_eq!(
        h.ok(
            "legacy_get_error_tags_for_question",
            json!({"questionId":question_id})
        )
        .await
        .as_array()
        .unwrap()
        .len(),
        1
    );

    let remote_id = new_id();
    h.ok(
        "legacy_upsert_error_tag",
        json!({"input":{
            "id":remote_id,"version":5,"status":"pending","deleted_at":null,
            "question_ids":[question_id],"name":"Remote","color":"white"
        }}),
    )
    .await;
    let remote = h
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":remote_id,"tableName":"error_tags"}),
        )
        .await;
    assert_eq!(remote["version"], 5);
    assert_eq!(remote["data"]["question_ids"], json!([question_id]));

    h.db.execute_unprepared(
        "CREATE TRIGGER reject_tag_update BEFORE UPDATE ON tag BEGIN SELECT RAISE(FAIL, 'forced update failure'); END;",
    )
    .await
    .unwrap();
    let update_error = h
        .err(
            "legacy_update_error_tag_by_id",
            json!({"tagId":concept_id,"newTagName":"must fail"}),
        )
        .await;
    assert!(update_error.to_string().contains("forced update failure"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn attachment_commands_contract() {
    let h = Harness::new().await;
    let (_, first_question_id) = create_subject_and_question(&h).await;
    let payload = "aW1hZ2U=";
    let created = h
        .ok(
            "legacy_create_attachment",
            json!({"input":{
                "question_id":first_question_id,"type":"original",
                "file_type":"image/png","base64_data":payload
            }}),
        )
        .await;
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["question_id"], first_question_id);
    assert_eq!(created["type_"], "original");
    assert_eq!(created["base64_data"], payload);
    assert_eq!(created["hash"].as_str().unwrap().len(), 64);

    let subject_id = h.ok("legacy_get_subjects", json!({})).await[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let second_question = h
        .ok(
            "legacy_create_question",
            json!({"input":{
                "user_id":"u","subject_id":subject_id,"source_id":null,
                "prompt":"second","type":"简答题","answer":"a",
                "analysis":null,"error_note":null
            }}),
        )
        .await;
    let second_question_id = second_question["id"].as_str().unwrap().to_owned();
    let batch = h
        .ok(
            "legacy_create_attachments_for_question",
            json!({"questionId":second_question_id,"attachments":[
                {"question_id":first_question_id,"type_":"original","file_type":"image/png","base64_data":"AQID"},
                {"question_id":first_question_id,"type_":"answer","file_type":"image/jpeg","base64_data":"/9j/2Q=="}
            ]}),
        )
        .await;
    assert_eq!(batch.as_array().unwrap().len(), 2);
    assert!(batch
        .as_array()
        .unwrap()
        .iter()
        .all(|attachment| attachment["question_id"] == second_question_id));

    h.ok(
        "legacy_delete_attachment",
        json!({"id":id,"questionId":first_question_id}),
    )
    .await;
    assert_eq!(
        h.ok(
            "legacy_get_attachments_by_question",
            json!({"questionId":first_question_id})
        )
        .await,
        json!([])
    );

    let remote_id = new_id();
    h.ok(
        "legacy_upsert_attachment",
        json!({"input":{
            "id":remote_id,"version":3,"status":"pending","deleted_at":null,
            "question_ids":[second_question_id],"type_":"answer",
            "file_type":"image/png","base64_data":[65,81,73,68],"hash":"ignored"
        }}),
    )
    .await;
    let remote = h
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":remote_id,"tableName":"attachments"}),
        )
        .await;
    assert_eq!(remote["version"], 3);
    assert_eq!(remote["data"]["question_ids"], json!([second_question_id]));
    assert_eq!(remote["data"]["base64_data"], json!([65, 81, 73, 68]));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn srs_commands_contract() {
    let h = Harness::new().await;
    assert_eq!(
        h.ok("legacy_get_srs_statistics", json!({})).await,
        json!({
            "total":0,"due_count":0,"new_cards":0,
            "avg_stability":0.0,"avg_difficulty":0.0,"total_reviews":0
        })
    );
    let (_, question_id) = create_subject_and_question(&h).await;
    let created = h
        .ok(
            "legacy_create_srs_data",
            json!({"input":{"question_id":question_id}}),
        )
        .await;
    assert_eq!(created["question_id"], question_id);
    assert_eq!(created["review_count"], 1);
    assert_eq!(created["is_due"], false);
    assert_eq!(
        h.err(
            "legacy_create_srs_data",
            json!({"input":{"question_id":question_id,"difficulty":8.0}}),
        )
        .await,
        format!("SrsData is exist: {question_id}")
    );

    let reset = h
        .ok(
            "legacy_reset_srs_progress",
            json!({"questionId":question_id}),
        )
        .await;
    assert_eq!(reset["review_count"], 1);
    assert_eq!(reset["is_due"], true);
    let invalid = h
        .err(
            "legacy_submit_review_result",
            json!({"input":{"question_id":question_id,"feedback":2.0}}),
        )
        .await;
    assert!(invalid.to_string().contains("Feedback must be in [0, 1]"));
    let review = h
        .ok(
            "legacy_submit_review_result",
            json!({"input":{"question_id":question_id,"feedback":0.8}}),
        )
        .await;
    assert!(review["next_interval_days"].as_f64().unwrap() > 0.0);
    assert_eq!(
        h.ok(
            "legacy_get_question_srs_status",
            json!({"questionId":question_id})
        )
        .await["review_count"],
        2
    );
    let statistics = h.ok("legacy_get_srs_statistics", json!({})).await;
    assert_eq!(statistics["total"], 1);
    assert_eq!(statistics["total_reviews"], 2);

    h.ok(
        "legacy_upsert_srs_data",
        json!({"input":{
            "id":question_id,"version":4,"status":"pending","deleted_at":null,
            "question_id":question_id,"stability":3.0,"difficulty":0.3,
            "next_review_at":null,"lastreviewed_at":null,"review_count":5,
            "feedback_history":"[0.8]"
        }}),
    )
    .await;
    let synced = h
        .ok(
            "legacy_get_question_srs_status",
            json!({"questionId":question_id}),
        )
        .await;
    assert_eq!(synced["review_count"], 5);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reset_srs_progress_revives_deleted_cards_and_creates_due_cards() {
    let h = Harness::new().await;
    let (subject_id, deleted_question_id) = create_subject_and_question(&h).await;
    h.ok(
        "legacy_create_srs_data",
        json!({"input":{"question_id":deleted_question_id}}),
    )
    .await;
    h.ok(
        "legacy_upsert_srs_data",
        json!({"input":{
            "id":deleted_question_id,"version":4,"status":"synced","deleted_at":1700000000,
            "question_id":deleted_question_id,"stability":3.0,"difficulty":0.3,
            "next_review_at":null,"lastreviewed_at":null,"review_count":5,
            "feedback_history":"[0.8]"
        }}),
    )
    .await;

    let revived = h
        .ok(
            "legacy_reset_srs_progress",
            json!({"questionId":deleted_question_id}),
        )
        .await;
    assert_eq!(revived["question_id"], deleted_question_id);
    assert_eq!(revived["review_count"], 1);
    assert_eq!(revived["is_due"], true);
    assert!(revived["deleted_at"].is_null());
    assert!(revived["next_review_at"].as_i64().unwrap() <= chrono::Utc::now().timestamp());
    let revived_upload = h
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":deleted_question_id,"tableName":"srs_data"}),
        )
        .await;
    assert_eq!(revived_upload["status"], "pending");
    assert!(revived_upload["deleted_at"].is_null());

    let question = h
        .ok(
            "legacy_create_question",
            json!({"input":{
                "user_id":"user","subject_id":subject_id,"source_id":null,
                "prompt":"new card","type":"简答题","answer":"answer",
                "analysis":null,"error_note":null
            }}),
        )
        .await;
    let new_question_id = question["id"].as_str().unwrap().to_owned();
    let created = h
        .ok(
            "legacy_reset_srs_progress",
            json!({"questionId":new_question_id}),
        )
        .await;
    assert_eq!(created["question_id"], new_question_id);
    assert_eq!(created["is_due"], true);
    assert!(created["next_review_at"].as_i64().unwrap() <= chrono::Utc::now().timestamp());
    let due = h.ok("legacy_get_due_questions", json!({"limit":10})).await;
    assert!(due
        .as_array()
        .unwrap()
        .iter()
        .any(|question| question["id"] == new_question_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sync_commands_contract() {
    let h = Harness::new().await;
    let (subject_id, question_id) = create_subject_and_question(&h).await;
    h.ok(
        "legacy_create_srs_data",
        json!({"input":{"question_id":question_id}}),
    )
    .await;

    let pending = h.ok("legacy_get_all_pending_records", json!({})).await;
    assert!(pending
        .as_array()
        .unwrap()
        .iter()
        .all(|record| record["status"] == "pending"));
    let table_order: Vec<_> = pending
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["table_name"].as_str().unwrap())
        .collect();
    assert_eq!(
        table_order,
        ["error_questions", "subjects", "srs_data", "sources"]
    );
    for record in pending.as_array().unwrap() {
        assert!(record["data"].get("sync_status").is_none());
        assert!(record["data"].get("created_at").is_none());
    }

    let upload = h
        .ok(
            "legacy_get_record_for_upload",
            json!({"recordId":question_id}),
        )
        .await;
    assert_eq!(upload["table_name"], "error_questions");
    assert!(upload["data"].get("userid").is_some());
    assert!(upload["data"].get("type_").is_some());
    assert!(upload["data"].get("user_id").is_none());

    h.ok(
        "legacy_set_record_sync_status_version",
        json!({"recordId":subject_id,"status":"conflict","version":10}),
    )
    .await;
    let subject = h
        .ok("legacy_get_subjects", json!({}))
        .await
        .as_array()
        .unwrap()
        .iter()
        .find(|subject| subject["id"] == subject_id)
        .unwrap()
        .clone();
    assert_eq!(subject["sync_status"], "conflict");
    assert_eq!(subject["version"], 10);

    let invalid_status = h
        .err(
            "legacy_set_record_sync_status_version",
            json!({"recordId":subject_id,"status":"invalid","version":99}),
        )
        .await;
    assert!(invalid_status
        .to_string()
        .contains("Unknown sync status: invalid"));
    let unchanged_subject = h
        .ok("legacy_get_subjects", json!({}))
        .await
        .as_array()
        .unwrap()
        .iter()
        .find(|subject| subject["id"] == subject_id)
        .unwrap()
        .clone();
    assert_eq!(unchanged_subject["sync_status"], "conflict");
    assert_eq!(unchanged_subject["version"], 10);

    let collision_id = question_id.clone();
    h.ok(
        "legacy_upsert_subject",
        json!({"input":{"id":collision_id,"version":1,"status":"ignored","name":"Collision","color":null}}),
    )
    .await;
    assert_eq!(
        h.ok(
            "legacy_get_record_for_upload",
            json!({"recordId":question_id})
        )
        .await["table_name"],
        "error_questions"
    );

    h.ok("legacy_delete_subject", json!({"id":subject_id}))
        .await;
    h.ok(
        "legacy_set_record_sync_status_version",
        json!({"recordId":subject_id,"status":"synced","version":11}),
    )
    .await;
    let purge = h.ok("legacy_purge_synced_deletions", json!({})).await;
    assert_eq!(purge["subjects"]["deleted"], 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn repository_transaction_executor_commits_and_rolls_back() {
    use crate::domain::repository::legacy::repository_model::subject::NewSubject;
    use crate::domain::repository::legacy::SubjectRepository;
    use crate::domain::repository::{RepositoryFactory, RepositoryTransactionExecutor};

    let mut options = ConnectOptions::new("sqlite::memory:");
    options.max_connections(1).min_connections(1);
    let db = Database::connect(options)
        .await
        .expect("connect transaction test database");
    init_database(&db)
        .await
        .expect("migrate transaction test database");

    let executor = crate::data::SeaOrmRepositoryTransactionExecutor::new(db.clone());
    let committed_id = uuid::Uuid::new_v4().to_string();
    let committed: Result<(), &str> = executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_subject_repository()
                    .create(NewSubject {
                        id: committed_id,
                        name: "Committed".into(),
                        color: None,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await;
                Ok(())
            })
        })
        .await;
    assert_eq!(committed, Ok(()));

    let rolled_back_id = uuid::Uuid::new_v4().to_string();
    let rolled_back: Result<(), &str> = executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_subject_repository()
                    .create(NewSubject {
                        id: rolled_back_id,
                        name: "Rolled back".into(),
                        color: None,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await;
                Err("rollback")
            })
        })
        .await;
    assert_eq!(rolled_back, Err("rollback"));

    let factory = crate::data::SeaOrmRepositoryFactory::new(&db);
    let subjects = factory.legacy_subject_repository().list_active().await;
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].name, "Committed");
}

#[tokio::test]
async fn current_ipc_creates_and_reads_a_question_with_related_outputs() {
    let harness = Harness::new().await;
    let subject = harness
        .ok(
            "create_subject",
            json!({"request":{"name":"Mathematics","color":"#336699"}}),
        )
        .await;
    let subject_id = subject["subject"]["id"].as_str().unwrap();
    let source = harness
        .ok("create_source", json!({"request":{"subjectId":subject_id}}))
        .await;
    let source_id = source["source"]["id"].as_str().unwrap();
    let tag = harness
        .ok(
            "create_tag",
            json!({"request":{"name":"careless","color":"#ff0000"}}),
        )
        .await;
    let attachment = harness
        .ok(
            "create_attachment",
            json!({"request":{
                "mimeType":"image/png",
                "base64Data":"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII="
            }}),
        )
        .await;

    let created_response = harness
        .ok(
            "create_question",
            json!({"request":{
                "sourceId": source_id,
                "questionType": "SHORT_ANSWER",
                "stem": "What is 2 + 2?",
                "correctAnswer": "4",
                "explanation": null,
                "note": "mental arithmetic",
                "tagIds": [tag["tag"]["id"]],
                "attachmentIds": [attachment["id"]]
            }}),
        )
        .await;
    let created = created_response["question"].clone();
    assert_eq!(created["sourceId"], source_id);
    assert_eq!(created["questionType"], "SHORT_ANSWER");
    assert_eq!(created["tagIds"], json!([tag["tag"]["id"]]));
    assert_eq!(created["attachmentIds"], json!([attachment["id"]]));
    for relation in ["subject", "source", "tags", "srs", "attachments"] {
        assert!(created.get(relation).is_none());
    }
    let updated = harness
        .ok(
            "update_question",
            json!({"request":{
                "id":created["id"],
                "sourceId":source_id,
                "questionType":"SHORT_ANSWER",
                "stem":"What is 2 + 2?",
                "correctAnswer":"4",
                "explanation":null,
                "note":"updated note",
                "tagIds":[tag["tag"]["id"]],
                "attachmentIds":[attachment["id"]]
            }}),
        )
        .await;
    assert_eq!(updated["question"]["note"], "updated note");
    let fetched_question = harness
        .ok("get_question", json!({"request":{"id":created["id"]}}))
        .await;
    assert_eq!(fetched_question["question"]["id"], created["id"]);
    assert_eq!(
        fetched_question["question"]["attachmentIds"],
        created["attachmentIds"]
    );
    let fetched_attachment = harness
        .ok("get_attachment", json!({"request":{"id":attachment["id"]}}))
        .await;
    assert_eq!(fetched_attachment["attachment"]["mimeType"], "image/png");
    assert!(fetched_attachment["attachment"]["base64Data"]
        .as_str()
        .unwrap()
        .starts_with("iVBOR"));
    assert_eq!(
        harness
            .err(
                "delete_attachment",
                json!({"request":{"id":attachment["id"]}}),
            )
            .await["code"],
        "RESOURCE_IN_USE"
    );
    let srs = harness
        .ok(
            "get_srs_data",
            json!({"request":{
                "questionId":created["id"],
                "at":chrono::Utc::now().to_rfc3339()
            }}),
        )
        .await;
    assert!(srs["srs"]["nextReviewAt"].as_str().unwrap().contains('T'));
    assert!(created.get("syncStatus").is_none());

    let page = harness
        .ok("list_questions", json!({"request":{"sort":[]}}))
        .await;
    assert_eq!(page["total"], 1);
    assert_eq!(page["items"][0]["stem"], "What is 2 + 2?");
    assert_eq!(page["items"][0]["attachmentIds"], created["attachmentIds"]);

    // Legacy remains registered during the migration window.
    assert_eq!(
        harness.ok("legacy_get_question_stats", json!({})).await["total"],
        1
    );
    let deleted = harness
        .ok("delete_question", json!({"request":{"id":created["id"]}}))
        .await;
    assert_eq!(deleted["id"], created["id"]);
}

#[tokio::test]
async fn current_subject_ipc_uses_operation_specific_wrappers() {
    let harness = Harness::new().await;
    let created = harness
        .ok(
            "create_subject",
            json!({"request":{"name":"   ","color":""}}),
        )
        .await;
    let id = created["subject"]["id"].as_str().unwrap();
    assert_eq!(created["subject"]["name"], "   ");
    assert_eq!(created["subject"]["color"], "");

    let listed = harness.ok("list_subjects", json!({})).await;
    assert_eq!(listed["subjects"].as_array().unwrap().len(), 1);
    assert_eq!(listed["subjects"][0]["id"], id);

    let updated = harness
        .ok(
            "update_subject",
            json!({"request":{"id":id,"name":"","color":"  "}}),
        )
        .await;
    assert_eq!(updated["subject"]["name"], "");
    assert_eq!(updated["subject"]["color"], "  ");

    let deleted = harness
        .ok("delete_subject", json!({"request":{"id":id}}))
        .await;
    assert_eq!(deleted["id"], id);
    assert!(harness.ok("list_subjects", json!({})).await["subjects"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn current_ipc_validates_the_full_request_before_writing() {
    let harness = Harness::new().await;
    let error = harness
        .err(
            "create_question",
            json!({"input":{"stem":"old wire shape","correctAnswer":""}}),
        )
        .await;
    assert!(error
        .as_str()
        .is_some_and(|message| message.contains("request")));

    let error = harness
        .err(
            "create_question",
            json!({"request":{
                "sourceId": null,
                "questionType": "SHORT_ANSWER",
                "stem": "invalid attachment",
                "correctAnswer": "",
                "newAttachments": [{"mimeType":"image/png","base64Data":"not base64"}]
            }}),
        )
        .await;
    assert!(error
        .as_str()
        .is_some_and(|message| message.contains("unknown field `newAttachments`")));

    let error = harness
        .err(
            "create_question",
            json!({"request":{
                "stem":"inline tag",
                "correctAnswer":"",
                "tags":[{"name":"temporary","color":"#000000"}]
            }}),
        )
        .await;
    assert!(error
        .as_str()
        .is_some_and(|message| message.contains("unknown field `tags`")));

    let error = harness
        .err(
            "create_attachment",
            json!({"request":{"mimeType":"image/png","base64Data":"not base64"}}),
        )
        .await;
    assert_eq!(error["code"], "INVALID_ARGUMENT");

    let error = harness
        .err(
            "create_attachment",
            json!({"request":{
                "mimeType":"image/jpeg",
                "base64Data":"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII="
            }}),
        )
        .await;
    assert_eq!(error["code"], "INVALID_ARGUMENT");

    let oversized = crate::util::codec::encode_base64(&vec![0; 10 * 1024 * 1024 + 1]);
    let error = harness
        .err(
            "create_attachment",
            json!({"request":{"mimeType":"image/png","base64Data":oversized}}),
        )
        .await;
    assert_eq!(error["code"], "INVALID_ARGUMENT");

    let attachment = harness
        .ok(
            "create_attachment",
            json!({"request":{
                "mimeType":"image/png",
                "base64Data":"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII="
            }}),
        )
        .await;
    assert_eq!(
        harness
            .ok(
                "delete_attachment",
                json!({"request":{"id":attachment["id"]}}),
            )
            .await["id"],
        attachment["id"]
    );
    assert_eq!(
        harness
            .err("get_attachment", json!({"request":{"id":attachment["id"]}}),)
            .await["code"],
        "NOT_FOUND"
    );

    let stats = harness
        .ok(
            "get_library_statistics",
            json!({"request":{"at":chrono::Utc::now().to_rfc3339()}}),
        )
        .await;
    let stats = &stats["statistics"];
    assert_eq!(stats["questionTotal"], 0);
    assert_eq!(stats["cardTotal"], 0);

    for (command, request) in [
        (
            "create_question",
            json!({"sourceId":"","stem":"empty source","correctAnswer":""}),
        ),
        (
            "create_source",
            json!({"subjectId":"","book":null,"chapter":null,"knowledge":null}),
        ),
        ("list_sources", json!({"subjectId":""})),
    ] {
        assert_eq!(
            harness.err(command, json!({"request":request})).await["code"],
            "INVALID_ARGUMENT"
        );
    }

    let empty_stem = harness
        .ok(
            "create_question",
            json!({"request":{"stem":"","correctAnswer":""}}),
        )
        .await;
    assert_eq!(empty_stem["question"]["stem"], "");
    let empty_stem = harness
        .ok(
            "update_question",
            json!({"request":{
                "id":empty_stem["question"]["id"],
                "sourceId":null,
                "questionType":null,
                "stem":"   ",
                "correctAnswer":"",
                "explanation":null,
                "note":null,
                "tagIds":[],
                "attachmentIds":[]
            }}),
        )
        .await;
    assert_eq!(empty_stem["question"]["stem"], "   ");

    assert_eq!(
        harness
            .err(
                "create_question",
                json!({"request":{
                    "questionType":"UNSUPPORTED",
                    "stem":"type",
                    "correctAnswer":""
                }}),
            )
            .await["code"],
        "INVALID_ARGUMENT"
    );
    assert_eq!(
        harness
            .err(
                "list_questions",
                json!({"request":{"filter":{"updatedSince":"invalid"},"sort":[]}}),
            )
            .await["code"],
        "INVALID_ARGUMENT"
    );
    assert!(harness
        .err(
            "list_questions",
            json!({"request":{"filter":{"updatedAfter":"2026-01-02T00:00:00Z"},"sort":[]}}),
        )
        .await
        .is_string());

    let missing_tag_id = new_id();
    let error = harness
        .err(
            "create_question",
            json!({"request":{
                "stem":"invalid tag reference",
                "correctAnswer":"",
                "tagIds":[missing_tag_id]
            }}),
        )
        .await;
    assert_eq!(error["code"], "MISSING_REFERENCE");
    assert!(harness.ok("list_tags", json!({})).await["tags"]
        .as_array()
        .unwrap()
        .is_empty());

    let missing_attachment_id = new_id();
    let error = harness
        .err(
            "create_question",
            json!({"request":{
                "stem":"invalid attachment reference",
                "correctAnswer":"",
                "attachmentIds":[missing_attachment_id]
            }}),
        )
        .await;
    assert_eq!(error["code"], "MISSING_REFERENCE");

    let error = harness
        .err(
            "create_question",
            json!({"request":{
                "subjectId":null,
                "sourceId":null,
                "stem":"subject is not a question input",
                "correctAnswer":""
            }}),
        )
        .await;
    assert!(error
        .as_str()
        .is_some_and(|message| message.contains("unknown field `subjectId`")));

    let error = harness
        .err(
            "update_question",
            json!({"request":{
                "id":new_id(),
                "subjectId":null,
                "stem":"subject is not an update input",
                "correctAnswer":""
            }}),
        )
        .await;
    assert!(error
        .as_str()
        .is_some_and(|message| message.contains("unknown field `subjectId`")));

    let error = harness
        .err(
            "list_questions",
            json!({"request":{"filter":{"subjectId":null},"sort":[]}}),
        )
        .await;
    assert!(error
        .as_str()
        .is_some_and(|message| message.contains("unknown field `subjectId`")));
}

#[tokio::test]
async fn current_ipc_preserves_distinct_tag_identities() {
    let harness = Harness::new().await;
    let first = harness
        .ok(
            "create_tag",
            json!({"request":{"name":"same","color":"#123456"}}),
        )
        .await;
    let second = harness
        .ok(
            "create_tag",
            json!({"request":{"name":"same","color":"#123456"}}),
        )
        .await;

    assert_ne!(first["tag"]["id"], second["tag"]["id"]);
    let tags = harness.ok("list_tags", json!({})).await;
    assert_eq!(tags["tags"].as_array().unwrap().len(), 2);

    let question = harness
        .ok(
            "create_question",
            json!({"request":{
                "stem":"reuse tag",
                "correctAnswer":"answer",
                "tagIds":[first["tag"]["id"]]
            }}),
        )
        .await;
    assert_eq!(question["question"]["tagIds"][0], first["tag"]["id"]);
    assert_eq!(
        harness.ok("list_tags", json!({})).await["tags"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let second_id = second["tag"]["id"].as_str().unwrap();
    let updated = harness
        .ok(
            "update_tag",
            json!({"request":{"id":second_id,"name":"  ","color":""}}),
        )
        .await;
    assert_eq!(updated["tag"]["name"], "  ");
    assert_eq!(updated["tag"]["color"], "");

    let empty = harness
        .ok("create_tag", json!({"request":{"name":"","color":"  "}}))
        .await;
    assert_eq!(empty["tag"]["name"], "");
    assert_eq!(empty["tag"]["color"], "  ");
    let deleted = harness
        .ok("delete_tag", json!({"request":{"id":second_id}}))
        .await;
    assert_eq!(deleted["id"], second_id);
}

#[tokio::test]
async fn current_ipc_create_source_always_creates_a_new_source() {
    let harness = Harness::new().await;
    let request = json!({"request":{
        "subjectId": null,
        "book": "Rust",
        "chapter": "Ownership",
        "knowledge": "Borrowing"
    }});
    let first = harness.ok("create_source", request.clone()).await;
    let second = harness.ok("create_source", request).await;

    assert_ne!(first["source"]["id"], second["source"]["id"]);
    assert_eq!(
        harness
            .ok("list_sources", json!({"request":{"subjectId":null}}))
            .await["sources"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let first_id = first["source"]["id"].as_str().unwrap();
    let fetched = harness
        .ok("get_source", json!({"request":{"id":first_id}}))
        .await;
    assert_eq!(fetched["source"]["book"], "Rust");
    let updated = harness
        .ok(
            "update_source",
            json!({"request":{
                "id":first_id,
                "subjectId":null,
                "book":"Rust",
                "chapter":"Lifetimes",
                "knowledge":null
            }}),
        )
        .await;
    assert_eq!(updated["source"]["chapter"], "Lifetimes");
    let second_id = second["source"]["id"].as_str().unwrap();
    let deleted = harness
        .ok("delete_source", json!({"request":{"id":second_id}}))
        .await;
    assert_eq!(deleted["id"], second_id);
}

#[tokio::test]
async fn current_ipc_updates_require_complete_replace_state() {
    let harness = Harness::new().await;
    let subject = harness
        .ok(
            "create_subject",
            json!({"request":{"name":"Math","color":"blue"}}),
        )
        .await;
    let source = harness
        .ok(
            "create_source",
            json!({"request":{
                "subjectId":subject["subject"]["id"],
                "book":"Algebra",
                "chapter":"Linear equations",
                "knowledge":"Elimination"
            }}),
        )
        .await;
    let tag = harness
        .ok(
            "create_tag",
            json!({"request":{"name":"careless","color":"#ff0000"}}),
        )
        .await;
    let question = harness
        .ok(
            "create_question",
            json!({"request":{
                "sourceId":source["source"]["id"],
                "questionType":"SHORT_ANSWER",
                "stem":"Solve x + 1 = 2",
                "correctAnswer":"x = 1",
                "explanation":"Subtract one",
                "note":"Check the sign",
                "tagIds":[tag["tag"]["id"]],
                "attachmentIds":[]
            }}),
        )
        .await;

    let complete_question_update = json!({
        "id":question["question"]["id"],
        "sourceId":null,
        "questionType":null,
        "stem":"Updated",
        "correctAnswer":"Updated answer",
        "explanation":null,
        "note":null,
        "tagIds":[],
        "attachmentIds":[]
    });
    for field in [
        "sourceId",
        "questionType",
        "explanation",
        "note",
        "tagIds",
        "attachmentIds",
    ] {
        let mut request = complete_question_update.clone();
        request.as_object_mut().unwrap().remove(field);
        let error = harness
            .err("update_question", json!({"request":request}))
            .await;
        assert!(
            error
                .as_str()
                .is_some_and(|message| message.contains("missing field")),
            "missing {field} returned {error}"
        );
    }
    let unchanged_question = harness
        .ok(
            "get_question",
            json!({"request":{"id":question["question"]["id"]}}),
        )
        .await;
    assert_eq!(unchanged_question["question"], question["question"]);

    let cleared_question = harness
        .ok(
            "update_question",
            json!({"request":complete_question_update}),
        )
        .await;
    for field in ["sourceId", "questionType", "explanation", "note"] {
        assert!(cleared_question["question"][field].is_null());
    }
    assert_eq!(cleared_question["question"]["tagIds"], json!([]));
    assert_eq!(cleared_question["question"]["attachmentIds"], json!([]));

    let complete_source_update = json!({
        "id":source["source"]["id"],
        "subjectId":null,
        "book":null,
        "chapter":null,
        "knowledge":null
    });
    for field in ["subjectId", "book", "chapter", "knowledge"] {
        let mut request = complete_source_update.clone();
        request.as_object_mut().unwrap().remove(field);
        let error = harness
            .err("update_source", json!({"request":request}))
            .await;
        assert!(
            error
                .as_str()
                .is_some_and(|message| message.contains("missing field")),
            "missing {field} returned {error}"
        );
    }
    let unchanged_source = harness
        .ok(
            "get_source",
            json!({"request":{"id":source["source"]["id"]}}),
        )
        .await;
    assert_eq!(unchanged_source["source"], source["source"]);

    let cleared_source = harness
        .ok("update_source", json!({"request":complete_source_update}))
        .await;
    for field in ["subjectId", "book", "chapter", "knowledge"] {
        assert!(cleared_source["source"][field].is_null());
    }
}

#[tokio::test]
async fn current_ipc_paginates_questions_after_counting_matches() {
    let harness = Harness::new().await;
    let mut ids = Vec::new();
    for stem in ["first", "second", "third"] {
        let created = harness
            .ok(
                "create_question",
                json!({"request":{"stem":stem,"correctAnswer":"answer"}}),
            )
            .await;
        ids.push(created["question"]["id"].as_str().unwrap().to_owned());
    }

    let unsorted = harness.ok("list_questions", json!({"request":{}})).await;
    assert_eq!(unsorted["total"], 3);
    assert_eq!(unsorted["items"].as_array().unwrap().len(), 3);

    let page = harness
        .ok(
            "list_questions",
            json!({"request":{"sort":["UPDATED_AT_DESC"],"offset":1,"limit":1}}),
        )
        .await;
    assert_eq!(page["total"], 3);
    assert_eq!(page["items"].as_array().unwrap().len(), 1);

    ids.sort();
    let single_level_sort = harness
        .ok("list_questions", json!({"request":{"sort":["ID_ASC"]}}))
        .await;
    assert_eq!(
        single_level_sort["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ids.iter().map(String::as_str).collect::<Vec<_>>()
    );

    let multi_sort = harness
        .ok(
            "list_questions",
            json!({"request":{"sort":["ID_DESC","UPDATED_AT_ASC"]}}),
        )
        .await;
    assert_eq!(
        multi_sort["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ids.iter().rev().map(String::as_str).collect::<Vec<_>>()
    );

    let unsorted = harness
        .ok("list_questions", json!({"request":{"sort":[]}}))
        .await;
    assert_eq!(unsorted["total"], 3);
    assert_eq!(unsorted["items"].as_array().unwrap().len(), 3);

    assert_eq!(
        harness
            .err(
                "list_questions",
                json!({"request":{"sort":["UPDATED_AT_DESC","UNSUPPORTED"]}}),
            )
            .await["code"],
        "INVALID_ARGUMENT"
    );
    assert!(harness
        .err("list_questions", json!({"request":{"sort":"ID_ASC"}}),)
        .await
        .as_str()
        .is_some());
    assert!(harness
        .err("list_questions", json!({}))
        .await
        .as_str()
        .is_some_and(|message| message.contains("request")));
}

#[tokio::test]
async fn current_ipc_subject_deletion_keeps_sources_and_questions_atomically() {
    let harness = Harness::new().await;
    let subject = harness
        .ok(
            "create_subject",
            json!({"request":{"name":"Math","color":"blue"}}),
        )
        .await;
    let subject_id = subject["subject"]["id"].as_str().unwrap();
    let source = harness
        .ok(
            "create_source",
            json!({"request":{"subjectId":subject_id,"book":"Algebra"}}),
        )
        .await;
    let source_id = source["source"]["id"].as_str().unwrap();
    let question = harness
        .ok(
            "create_question",
            json!({"request":{"sourceId":source_id,"stem":"1+1","correctAnswer":"2"}}),
        )
        .await;
    let question_id = question["question"]["id"].as_str().unwrap();

    let empty_subject = harness
        .ok(
            "create_subject",
            json!({"request":{"name":"Empty","color":"gray"}}),
        )
        .await;
    let empty_subject_id = empty_subject["subject"]["id"].as_str().unwrap();
    harness
        .ok("delete_subject", json!({"request":{"id":empty_subject_id}}))
        .await;
    let still_classified = harness
        .ok("get_question", json!({"request":{"id":question_id}}))
        .await;
    assert_eq!(still_classified["question"]["sourceId"], source_id);

    harness
        .ok("delete_subject", json!({"request":{"id":subject_id}}))
        .await;
    let retained = harness
        .ok("get_question", json!({"request":{"id":question_id}}))
        .await;
    assert_eq!(retained["question"]["sourceId"], source_id);
    let retained_source = harness
        .ok("get_source", json!({"request":{"id":source_id}}))
        .await;
    assert_eq!(retained_source["source"]["id"], source_id);
    assert_eq!(
        retained_source["source"]["subjectId"],
        serde_json::Value::Null
    );
    assert!(harness.ok("list_subjects", json!({})).await["subjects"]
        .as_array()
        .unwrap()
        .iter()
        .all(|subject| subject["id"] != subject_id));
}

#[tokio::test]
async fn current_ipc_batch_source_delete_validates_before_mutating() {
    let harness = Harness::new().await;
    let source = harness
        .ok("create_source", json!({"request":{"book":"kept"}}))
        .await;
    let source_id = source["source"]["id"].as_str().unwrap();
    let question = harness
        .ok(
            "create_question",
            json!({"request":{"sourceId":source_id,"stem":"kept","correctAnswer":"answer"}}),
        )
        .await;
    let question_id = question["question"]["id"].as_str().unwrap();
    harness
        .ok("delete_sources", json!({"request":{"ids":[]}}))
        .await;
    assert_eq!(
        harness
            .ok("get_source", json!({"request":{"id":source_id}}))
            .await["source"]["id"],
        source_id
    );
    let missing_id = new_id();
    let error = harness
        .err(
            "delete_sources",
            json!({"request":{"ids":[source_id, missing_id]}}),
        )
        .await;
    assert_eq!(error["code"], "NOT_FOUND");
    assert_eq!(error["details"]["id"], missing_id);
    assert_eq!(
        harness
            .ok("get_source", json!({"request":{"id":source_id}}))
            .await["source"]["id"],
        source_id
    );
    harness
        .ok("delete_sources", json!({"request":{"ids":[source_id]}}))
        .await;
    assert_eq!(
        harness
            .ok("get_question", json!({"request":{"id":question_id}}))
            .await["question"]["sourceId"],
        serde_json::Value::Null
    );
}

#[tokio::test]
async fn current_ipc_composes_review_operations_from_question_and_srs_use_cases() {
    let harness = Harness::new().await;
    let first = harness
        .ok(
            "create_question",
            json!({"request":{"stem":"due question","correctAnswer":"answer"}}),
        )
        .await;
    harness
        .ok(
            "create_question",
            json!({"request":{"stem":"future question","correctAnswer":"answer"}}),
        )
        .await;
    let first_id = first["question"]["id"].as_str().unwrap();
    for (command, arguments) in [
        (
            "submit_review",
            json!({"input":{"questionId":first_id,"feedback":0.5}}),
        ),
        ("reset_review_progress", json!({"questionId":first_id})),
        ("get_library_statistics", json!({})),
    ] {
        assert!(harness
            .err(command, arguments)
            .await
            .as_str()
            .is_some_and(|message| message.contains("request")));
    }

    for (command, request) in [
        (
            "submit_review",
            json!({"questionId":first_id,"feedback":0.5,"reviewedAt":"invalid"}),
        ),
        (
            "reset_review_progress",
            json!({"questionId":first_id,"resetAt":"invalid"}),
        ),
        (
            "get_srs_data",
            json!({"questionId":first_id,"at":"invalid"}),
        ),
        ("list_srs_data", json!({"at":"invalid"})),
        ("get_library_statistics", json!({"at":"invalid"})),
    ] {
        assert_eq!(
            harness.err(command, json!({"request":request})).await["code"],
            "INVALID_ARGUMENT"
        );
    }

    assert_eq!(
        harness
            .ok(
                "get_library_statistics",
                json!({"request":{"at":"2100-01-01T00:00:00Z"}}),
            )
            .await["statistics"]["dueCount"],
        2
    );

    let before_invalid_reset = harness
        .ok(
            "get_srs_data",
            json!({"request":{"questionId":first_id,"at":chrono::Utc::now().to_rfc3339()}}),
        )
        .await;
    let invalid_reset = harness
        .err(
            "reset_review_progress",
            json!({"request":{
                "questionId":first_id,
                "resetAt":"1970-01-01T00:00:00Z"
            }}),
        )
        .await;
    assert_eq!(invalid_reset["code"], "INVALID_ARGUMENT");
    let after_invalid_reset = harness
        .ok(
            "get_srs_data",
            json!({"request":{"questionId":first_id,"at":chrono::Utc::now().to_rfc3339()}}),
        )
        .await;
    assert_eq!(after_invalid_reset["srs"], before_invalid_reset["srs"]);

    harness
        .ok(
            "reset_review_progress",
            json!({"request":{
                "questionId":first_id,
                "resetAt":chrono::Utc::now().to_rfc3339()
            }}),
        )
        .await;
    let due = harness
        .ok(
            "list_questions",
            json!({"request":{
                "filter":{"reviewState":"DUE"},
                "sort":["MASTERY_ASC"],
                "limit":1
            }}),
        )
        .await;
    assert_eq!(due["total"], 1);
    assert_eq!(due["items"].as_array().unwrap().len(), 1);
    assert_eq!(due["items"][0]["id"], first_id);
    assert_eq!(due["items"][0]["attachmentIds"], json!([]));

    let all = harness
        .ok(
            "list_questions",
            json!({"request":{"sort":["MASTERY_ASC"],"limit":1}}),
        )
        .await;
    assert_eq!(all["total"], 2);
    assert_eq!(all["items"].as_array().unwrap().len(), 1);

    let review = harness
        .ok(
            "submit_review",
            json!({"request":{
                "questionId":first_id,
                "feedback":0.5,
                "reviewedAt":chrono::Utc::now().to_rfc3339()
            }}),
        )
        .await;
    assert_eq!(review["srs"]["reviewCount"], 2);
    assert!(review["nextIntervalDays"].as_f64().unwrap() > 0.0);

    assert!(harness
        .ok(
            "list_questions",
            json!({"request":{"filter":{"reviewState":"DUE"},"sort":[]}}),
        )
        .await["items"]
        .as_array()
        .unwrap()
        .is_empty());
    let at = chrono::Utc::now().to_rfc3339();
    let srs_items = harness
        .ok("list_srs_data", json!({"request":{"at":&at}}))
        .await;
    assert_eq!(srs_items["items"].as_array().unwrap().len(), 2);
    assert_eq!(
        harness
            .err(
                "get_srs_data",
                json!({"request":{"questionId":new_id(),"at":&at}}),
            )
            .await["code"],
        "NOT_FOUND"
    );
    let statistics = harness
        .ok(
            "get_library_statistics",
            json!({"request":{"at":chrono::Utc::now().to_rfc3339()}}),
        )
        .await;
    let statistics = &statistics["statistics"];
    assert_eq!(statistics["questionTotal"], 2);
    assert_eq!(statistics["cardTotal"], 2);
    assert_eq!(statistics["dueCount"], 0);
    assert_eq!(statistics["newCardCount"], 1);
    assert_eq!(statistics["totalReviews"], 3);
}

#[tokio::test]
async fn current_ipc_preserves_the_domain_interval_for_an_immediate_lapse() {
    let harness = Harness::new().await;
    let question = harness
        .ok(
            "create_question",
            json!({"request":{"stem":"lapsed question","correctAnswer":"answer"}}),
        )
        .await;
    let question_id = question["question"]["id"].as_str().unwrap();
    let reviewed_at = chrono::Utc::now().to_rfc3339();

    let review = harness
        .ok(
            "submit_review",
            json!({"request":{
                "questionId":question_id,
                "feedback":0.0,
                "reviewedAt":reviewed_at
            }}),
        )
        .await;

    assert_eq!(review["nextIntervalDays"], 1.0);
    assert_eq!(review["srs"]["nextReviewAt"], reviewed_at);
    assert_eq!(review["srs"]["reviewCount"], 2);
    assert_eq!(review["srs"]["isDue"], true);
}

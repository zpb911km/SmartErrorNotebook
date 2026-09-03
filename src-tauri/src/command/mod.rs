pub mod legacy;

pub trait CommandRegistry {
    fn register_command(self) -> Self;
}

impl<R: tauri::Runtime> CommandRegistry for tauri::Builder<R> {
    fn register_command(self) -> Self {
        self.invoke_handler(tauri::generate_handler![
            // Sync
            legacy::legacy_get_all_pending_records,
            legacy::legacy_get_record_for_upload,
            legacy::legacy_set_record_sync_status_version,
            legacy::legacy_get_all_records,
            legacy::legacy_purge_synced_deletions,
            legacy::legacy_check_orphan_records,
            // Subject
            legacy::legacy_get_subjects,
            legacy::legacy_create_subject,
            legacy::legacy_update_subject,
            legacy::legacy_delete_subject,
            legacy::legacy_upsert_subject,
            // Error Question
            legacy::legacy_get_questions,
            legacy::legacy_get_question,
            legacy::legacy_create_question,
            legacy::legacy_update_question,
            legacy::legacy_delete_question,
            legacy::legacy_upsert_error_question,
            legacy::legacy_get_question_stats,
            // Error Tag
            legacy::legacy_create_error_tags_for_question,
            legacy::legacy_get_error_tags,
            legacy::legacy_get_full_error_tags,
            legacy::legacy_get_error_tags_for_question,
            legacy::legacy_delete_error_tag,
            legacy::legacy_update_error_tag_by_id,
            legacy::legacy_update_error_tag_by_name,
            legacy::legacy_upsert_error_tag,
            // SRS Data and tools
            legacy::legacy_create_srs_data,
            legacy::legacy_get_due_questions,
            legacy::legacy_submit_review_result,
            legacy::legacy_get_question_srs_status,
            legacy::legacy_reset_srs_progress,
            legacy::legacy_upsert_srs_data,
            legacy::legacy_get_due_count,
            legacy::legacy_get_srs_statistics,
            legacy::legacy_get_all_cards,
            // Attachment
            legacy::legacy_create_attachment,
            legacy::legacy_create_attachments_for_question,
            legacy::legacy_get_attachments_by_question,
            legacy::legacy_delete_attachment,
            legacy::legacy_upsert_attachment,
            // Source
            legacy::legacy_get_sources,
            legacy::legacy_get_books,
            legacy::legacy_get_chapters,
            legacy::legacy_get_knowledges,
            legacy::legacy_create_source,
            legacy::legacy_update_source,
            legacy::legacy_delete_source,
            legacy::legacy_get_source,
            legacy::legacy_get_or_create_source_id,
            legacy::legacy_upsert_source,
            // File Association
            legacy::legacy_opened_urls,
            legacy::legacy_read_opened_file,
        ])
    }
}

pub mod legacy;

pub trait CommandRegistry {
    fn register_command(self) -> Self;
}

impl<R: tauri::Runtime> CommandRegistry for tauri::Builder<R> {
    fn register_command(self) -> Self {
        self.invoke_handler(tauri::generate_handler![
            // Sync
            legacy::get_all_pending_records,
            legacy::get_record_for_upload,
            legacy::set_record_sync_status_version,
            legacy::get_all_records,
            legacy::purge_synced_deletions,
            legacy::check_orphan_records,
            // Subject
            legacy::get_subjects,
            legacy::create_subject,
            legacy::update_subject,
            legacy::delete_subject,
            legacy::upsert_subject,
            // Error Question
            legacy::get_questions,
            legacy::get_question,
            legacy::create_question,
            legacy::update_question,
            legacy::delete_question,
            legacy::upsert_error_question,
            legacy::get_question_stats,
            // Error Tag
            legacy::create_error_tags_for_question,
            legacy::get_error_tags,
            legacy::get_full_error_tags,
            legacy::get_error_tags_for_question,
            legacy::delete_error_tag,
            legacy::update_error_tag_by_id,
            legacy::update_error_tag_by_name,
            legacy::upsert_error_tag,
            // SRS Data and tools
            legacy::create_srs_data,
            legacy::get_due_questions,
            legacy::submit_review_result,
            legacy::get_question_srs_status,
            legacy::reset_srs_progress,
            legacy::upsert_srs_data,
            legacy::get_due_count,
            legacy::get_srs_statistics,
            legacy::get_all_cards,
            // Attachment
            legacy::create_attachment,
            legacy::create_attachments_for_question,
            legacy::get_attachments_by_question,
            legacy::delete_attachment,
            legacy::upsert_attachment,
            // Source
            legacy::get_sources,
            legacy::get_books,
            legacy::get_chapters,
            legacy::get_knowledges,
            legacy::create_source,
            legacy::update_source,
            legacy::delete_source,
            legacy::get_source,
            legacy::get_or_create_source_id,
            legacy::upsert_source,
            // File Association
            legacy::opened_urls,
            legacy::read_opened_file,
        ])
    }
}

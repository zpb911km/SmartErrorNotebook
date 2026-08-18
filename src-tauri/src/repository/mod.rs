pub mod legacy;

use std::future::Future;
use std::pin::Pin;

// RepositoryTransactionExecutor提供事务保证。
pub trait RepositoryFactory {
    fn legacy_attachment_repository(&self) -> impl legacy::AttachmentRepository;
    fn legacy_error_question_repository(&self) -> impl legacy::ErrorQuestionRepository;
    fn legacy_error_tag_repository(&self) -> impl legacy::ErrorTagRepository;
    fn legacy_source_repository(&self) -> impl legacy::SourceRepository;
    fn legacy_srs_data_repository(&self) -> impl legacy::SrsDataRepository;
    fn legacy_subject_repository(&self) -> impl legacy::SubjectRepository;
    fn legacy_sync_repository(&self) -> impl legacy::SyncRepository;
}

#[async_trait::async_trait]
pub trait RepositoryTransactionExecutor {
    type Factory<'tx>: RepositoryFactory;

    async fn execute<T, E, F>(&self, f: F) -> Result<T, E>
    where
        T: Send,
        E: Send,
        F: Send,
        for<'tx> F: FnOnce(
            Self::Factory<'tx>,
            &'tx (), // dummy parameter
                     // See https://doc.rust-lang.org/error_codes/E0582.html and
                     // https://github.com/rust-lang/rust/issues/107572 for more details.
        ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'tx>>;
}

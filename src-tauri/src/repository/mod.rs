#![allow(dead_code)] // Internal interfaces and legacy wire models intentionally retain staged fields.

mod attachment_repository;
mod question_repository;
mod source_repository;
mod srs_data_repository;
mod subject_repository;
mod tag_repository;

pub(crate) mod error;

pub(crate) mod legacy;

pub(crate) use attachment_repository::AttachmentRepository;
pub(crate) use question_repository::QuestionRepository;
pub(crate) use source_repository::SourceRepository;
pub(crate) use srs_data_repository::SrsDataRepository;
pub(crate) use subject_repository::SubjectRepository;
pub(crate) use tag_repository::TagRepository;

use std::future::Future;
use std::pin::Pin;

// RepositoryTransactionExecutor提供事务保证。
pub(crate) trait RepositoryFactory: Send {
    fn legacy_attachment_repository(&self) -> impl legacy::AttachmentRepository;
    fn legacy_error_question_repository(&self) -> impl legacy::ErrorQuestionRepository;
    fn legacy_error_tag_repository(&self) -> impl legacy::ErrorTagRepository;
    fn legacy_source_repository(&self) -> impl legacy::SourceRepository;
    fn legacy_srs_data_repository(&self) -> impl legacy::SrsDataRepository;
    fn legacy_subject_repository(&self) -> impl legacy::SubjectRepository;
    fn legacy_sync_repository(&self) -> impl legacy::SyncRepository;

    fn attachment_repository(&self) -> impl AttachmentRepository;
    fn question_repository(&self) -> impl QuestionRepository;
    fn tag_repository(&self) -> impl TagRepository;
    fn source_repository(&self) -> impl SourceRepository;
    fn srs_data_repository(&self) -> impl SrsDataRepository;
    fn subject_repository(&self) -> impl SubjectRepository;
}

#[async_trait::async_trait]
pub(crate) trait RepositoryTransactionExecutor {
    type Factory<'tx>: RepositoryFactory + 'tx;

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

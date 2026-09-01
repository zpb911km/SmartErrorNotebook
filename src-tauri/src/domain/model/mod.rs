//! Domain models expose ordinary data directly to avoid getter/setter boilerplate.
//! Fields whose invariants depend on coordinated updates are private and must be
//! changed through their model's constructor or domain methods.

mod attachment;
mod macros;
mod metadata;
mod question;
mod source;
mod srs_data;
mod subject;
mod tag;

pub(crate) mod error;

pub(crate) mod legacy;

pub(crate) use attachment::Attachment;
pub(crate) use metadata::{Metadata, SyncStatus};
pub(crate) use question::{Question, QuestionType};
pub(crate) use source::Source;
pub(crate) use srs_data::SrsData;
pub(crate) use subject::Subject;
pub(crate) use tag::Tag;

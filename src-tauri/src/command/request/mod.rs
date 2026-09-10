use serde::{Deserialize, Deserializer};

pub(crate) mod attachment;
pub(crate) mod question;
pub(crate) mod review;
pub(crate) mod source;
pub(crate) mod subject;
pub(crate) mod tag;

pub(crate) fn deserialize_required_nullable<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

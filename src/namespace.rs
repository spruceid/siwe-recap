use crate::error::Error;

use iri_string::{spec::UriSpec, validate::absolute_iri};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
/// The namespace that a capability is valid for.
pub struct Namespace(String);

impl From<Namespace> for String {
    fn from(from: Namespace) -> String {
        from.0
    }
}

impl AsRef<str> for Namespace {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for Namespace {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        absolute_iri::<UriSpec>(&format!("{}:", s))
            .map(|()| Self(s.into()))
            .map_err(Error::InvalidNamespace)
    }
}

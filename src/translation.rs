use crate::{Capability, Error, RESOURCE_PREFIX};

use iri_string::types::UriString;
use siwe::Message;

/// Extract the encoded capabilities from a SIWE message.
pub fn extract_capabilities(message: &Message) -> Result<Option<Capability>, Error> {
    message
        .resources
        .iter()
        .find(|res| res.as_str().starts_with(RESOURCE_PREFIX))
        .map(Capability::from_resource)
        .transpose()
}

/// Generate a ReCap statement from capabilities and URI (delegee).
pub fn capabilities_to_statement(capabilities: &Capability, delegee_uri: &UriString) -> String {
    [
        "I further authorize ".to_string(),
        delegee_uri.to_string(),
        " to perform the following actions on my behalf:".to_string(),
        capabilities
            .to_statement_lines()
            .enumerate()
            .map(|(n, line)| [format!(" ({}) ", n), line].concat())
            .collect(),
    ]
    .concat()
}

pub trait ToResource {
    fn to_resource(self) -> Result<UriString, Error>;
}

trait FromResource {
    fn from_resource(resource: &UriString) -> Result<Self, Error>
    where
        Self: Sized;
}

impl ToResource for &Capability {
    fn to_resource(self) -> Result<UriString, Error> {
        self.encode()
            .map(|encoded| format!("{}:{}", RESOURCE_PREFIX, encoded))
            .and_then(|s| s.parse().map_err(Error::UriParse))
    }
}

impl FromResource for Capability {
    fn from_resource(resource: &UriString) -> Result<Self, Error> {
        resource
            .as_str()
            .strip_prefix(RESOURCE_PREFIX)
            .ok_or_else(|| Error::InvalidResourcePrefix(resource.to_string()))
            .and_then(|data| Capability::decode(data))
    }
}

use crate::{Capability, Error, Namespace, RESOURCE_PREFIX};

use std::collections::BTreeMap;

use iri_string::types::UriString;
use siwe::Message;

/// Extract the encoded capabilities from a SIWE message.
pub fn extract_capabilities(message: &Message) -> Result<BTreeMap<Namespace, Capability>, Error> {
    message
        .resources
        .iter()
        .filter(|res| res.as_str().starts_with(RESOURCE_PREFIX))
        .map(<(Namespace, Capability)>::from_resource)
        .collect()
}

/// Generate a ReCap statement from capabilities and URI (delegee).
pub fn capabilities_to_statement<'l>(
    capabilities: impl Iterator<Item = &'l Capability>,
    delegee_uri: &UriString,
) -> String {
    [
        "I further authorize ".to_string(),
        delegee_uri.to_string(),
        " to perform the following actions on my behalf:".to_string(),
        capabilities
            .map(|c| c.to_statement_lines())
            .flatten()
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

impl ToResource for (&Namespace, &Capability) {
    fn to_resource(self) -> Result<UriString, Error> {
        self.1
            .encode()
            .map(|encoded| format!("{}{}:{}", RESOURCE_PREFIX, self.0, encoded))
            .and_then(|s| s.parse().map_err(Error::UriParse))
    }
}

impl FromResource for (Namespace, Capability) {
    fn from_resource(resource: &UriString) -> Result<Self, Error> {
        resource
            .as_str()
            .strip_prefix(RESOURCE_PREFIX)
            .ok_or_else(|| Error::InvalidResourcePrefix(resource.to_string()))
            .and_then(|rest| {
                rest.rsplit_once(':')
                    .ok_or_else(|| Error::MissingBody(resource.to_string()))
            })
            .and_then(|(namespace, data)| {
                Capability::decode(data).and_then(|cap| Ok((namespace.parse()?, cap)))
            })
    }
}

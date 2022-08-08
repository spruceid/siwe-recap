use crate::{Capability, Error, Namespace, RESOURCE_PREFIX};

use std::collections::BTreeMap;
use std::fmt::Write;

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

/// Generate a capgrok statement from capabilities and URI (delegee).
pub fn capabilities_to_statement(
    capabilities: &BTreeMap<Namespace, Capability>,
    delegee_uri: &UriString,
) -> Option<String> {
    if capabilities.is_empty() {
        return None;
    }

    let mut statement = format!(
        "I further authorize {} to perform the following actions on my behalf:",
        delegee_uri
    );

    let mut line_no = 0;
    capabilities
        .iter()
        .flat_map(|(ns, cap)| cap.to_statement_lines(ns))
        .for_each(|line| {
            line_no += 1;
            // Ignore the error as write! is infallible for String.
            // See: https://rust-lang.github.io/rust-clippy/master/index.html#format_push_string
            let _ = write!(statement, " ({}) {}", line_no, line);
        });

    Some(statement)
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

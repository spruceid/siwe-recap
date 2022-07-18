use crate::Namespace;
use crate::RESOURCE_PREFIX;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("namespace can only contain alphanumeric chars or '-'")]
    InvalidNamespaceChars,
    #[error("namespace cannot begin with, end with, or contain consecutive hyphens")]
    InvalidNamespaceHyphens,
    #[error("failed to decode base64 capability resource: {0}")]
    Base64Decode(base64::DecodeError),
    #[error("failed to serialize capability to json: {0}")]
    Ser(serde_json::Error),
    #[error("failed to deserialize capability from json: {0}")]
    De(serde_json::Error),
    #[error(
        "invalid resource prefix (expected prefix: {}, found: {0})",
        RESOURCE_PREFIX
    )]
    InvalidResourcePrefix(String),
    #[error("duplicated resource namespace: {0}")]
    DuplicateNamespace(Namespace),
    #[error("capability resource is missing a body: {0}")]
    MissingBody(String),
    #[error("unable to parse capability as a URI: {0}")]
    UriParse(iri_string::validate::Error),
}

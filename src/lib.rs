mod serde_uri_string;

use std::collections::{BTreeMap, HashMap};

use iri_string::{spec::UriSpec, types::UriString, validate::absolute_iri};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use siwe::Message;

#[derive(Clone, Debug)]
pub struct Capability {
    namespace: String,
    inner: CapabilityInner,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct CapabilityInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    default_actions: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        with = "serde_uri_string"
    )]
    targeted_actions: BTreeMap<UriString, Vec<String>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    extra_fields: HashMap<String, Value>,
}

pub struct DelegationBuilder {
    message: Message,
    capabilities: Vec<Capability>,
}

pub fn verify_statement_matches_delegations(message: &Message) -> Result<bool, Error> {
    let capabilities = extract_capabilities(message)?;
    let generated_statement = DelegationBuilder::capabilities_to_statement(&capabilities);
    Ok(message.statement == Some(generated_statement))
}

pub fn extract_capabilities(message: &Message) -> Result<Vec<Capability>, Error> {
    message
        .resources
        .iter()
        .map(Capability::from_resource)
        .collect()
}

impl Capability {
    const RESOURCE_PREFIX: &'static str = "urn:capability:";

    pub fn new(namespace: String, default_actions: Option<Vec<String>>) -> Result<Self, Error> {
        absolute_iri::<UriSpec>(&format!("{}:", namespace))
            .map_err(Error::InvalidNamespace)
            .map(|()| Self {
                namespace,
                inner: CapabilityInner::new(default_actions),
            })
    }

    pub fn with_field(mut self, key: String, value: Value) -> Self {
        self.inner.extra_fields.insert(key, value);
        self
    }

    pub fn with_action(mut self, target: UriString, action: String) -> Self {
        if let Some(actions) = self.inner.targeted_actions.get_mut(&target) {
            actions.push(action);
        } else {
            self.inner.targeted_actions.insert(target, vec![action]);
        }
        self
    }

    pub fn with_actions(mut self, target: UriString, actions: Vec<String>) -> Self {
        if let Some(current_actions) = self.inner.targeted_actions.get_mut(&target) {
            current_actions.extend_from_slice(&actions);
        } else {
            self.inner.targeted_actions.insert(target, actions);
        }
        self
    }

    fn to_resource(&self) -> Result<UriString, Error> {
        self.inner
            .encode()
            .map(|encoded| format!("{}{}:{}", Self::RESOURCE_PREFIX, self.namespace, encoded))
            .and_then(|s| s.parse().map_err(Error::UriParse))
    }

    fn from_resource(resource: &UriString) -> Result<Self, Error> {
        resource
            .as_str()
            .strip_prefix(Self::RESOURCE_PREFIX)
            .ok_or_else(|| Error::InvalidResourcePrefix(resource.to_string()))
            .and_then(|rest| {
                rest.split_once(':')
                    .ok_or_else(|| Error::MissingBody(resource.to_string()))
            })
            .and_then(|(namespace, data)| {
                CapabilityInner::decode(data).map(|inner| Capability {
                    namespace: namespace.to_string(),
                    inner,
                })
            })
    }

    fn to_statement_lines(&self) -> impl Iterator<Item = String> + '_ {
        self.inner.to_statement(&self.namespace)
    }
}

impl DelegationBuilder {
    pub fn new(message: Message) -> DelegationBuilder {
        DelegationBuilder {
            message,
            capabilities: vec![],
        }
    }

    pub fn with_capability(mut self, capability: Capability) -> DelegationBuilder {
        self.capabilities.push(capability);
        self
    }

    pub fn build(mut self) -> Result<Message, Error> {
        let statement = Self::capabilities_to_statement(&self.capabilities);
        let resources = self
            .capabilities
            .iter()
            .map(|cap| cap.to_resource())
            .collect::<Result<Vec<UriString>, Error>>()?;

        self.message.statement = Some(statement);
        self.message.resources = resources;

        Ok(self.message)
    }

    fn capabilities_to_statement(capabilities: &[Capability]) -> String {
        let mut statement = String::from("By signing this message I am signing in with Ethereum");

        if capabilities.is_empty() {
            statement.push('.');
            return statement;
        }

        statement.push_str(
            " and authorizing the presented URI to perform the following actions on my behalf:",
        );

        let mut line_no = 0;
        capabilities
            .iter()
            .flat_map(|cap| cap.to_statement_lines())
            .for_each(|line| {
                line_no += 1;
                statement.push_str(&format!(" ({}) {}", line_no, line));
            });

        statement
    }
}

impl CapabilityInner {
    fn new(default_actions: Option<Vec<String>>) -> Self {
        Self {
            default_actions: default_actions.unwrap_or_default(),
            extra_fields: Default::default(),
            targeted_actions: Default::default(),
        }
    }

    fn encode(&self) -> Result<String, Error> {
        serde_json::to_vec(self)
            .map_err(Error::Ser)
            .map(|bytes| base64::encode_config(bytes, base64::URL_SAFE_NO_PAD))
    }

    fn decode(encoded: &str) -> Result<Self, Error> {
        base64::decode_config(encoded, base64::URL_SAFE_NO_PAD)
            .map_err(Error::Base64Decode)
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(Error::De))
    }

    fn to_statement<'l>(&'l self, namespace: &'l str) -> impl Iterator<Item = String> + 'l {
        std::iter::once(self.default_actions.join(", "))
            .filter(|actions| !actions.is_empty())
            .map(move |actions| format!("{}: {} for any.", namespace, actions))
            .chain(self.targeted_actions.iter().map(move |(target, actions)| {
                format!("{}: {} for {}.", namespace, actions.join(", "), target)
            }))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("the capability namespace must be a valid URI scheme: {0}")]
    InvalidNamespace(iri_string::validate::Error),
    #[error("failed to decode base64 capability resource: {0}")]
    Base64Decode(base64::DecodeError),
    #[error("failed to serialize capability to json: {0}")]
    Ser(serde_json::Error),
    #[error("failed to deserialize capability from json: {0}")]
    De(serde_json::Error),
    #[error(
        "invalid resource prefix (expected prefix: {}, found: {0})",
        Capability::RESOURCE_PREFIX
    )]
    InvalidResourcePrefix(String),
    #[error("capability resource is missing a body: {0}")]
    MissingBody(String),
    #[error("unable to parse capability as a URI: {0}")]
    UriParse(iri_string::validate::Error),
}

#[cfg(test)]
mod test {
    use super::*;

    const SIWE: &'static str =
"example.com wants you to sign in with your Ethereum account:
0x0000000000000000000000000000000000000000

By signing this message I am signing in with Ethereum and authorizing the presented URI to perform the following actions on my behalf: (1) credential: present for any. (2) kepler: list, get, metadata for kepler:ens:example.eth://default/kv. (3) kepler: list, get, metadata, put, delete for kepler:ens:example.eth://default/kv/dapp-space.

URI: did:key:example
Version: 1
Chain ID: 1
Nonce: mynonce1
Issued At: 2022-06-21T12:00:00.000Z
Resources:
- urn:capability:credential:eyJkZWZhdWx0X2FjdGlvbnMiOlsicHJlc2VudCJdfQ
- urn:capability:kepler:eyJ0YXJnZXRlZF9hY3Rpb25zIjp7ImtlcGxlcjplbnM6ZXhhbXBsZS5ldGg6Ly9kZWZhdWx0L2t2IjpbImxpc3QiLCJnZXQiLCJtZXRhZGF0YSJdLCJrZXBsZXI6ZW5zOmV4YW1wbGUuZXRoOi8vZGVmYXVsdC9rdi9kYXBwLXNwYWNlIjpbImxpc3QiLCJnZXQiLCJtZXRhZGF0YSIsInB1dCIsImRlbGV0ZSJdfX0";

    #[test]
    fn build_delegation() {
        let msg = DelegationBuilder::new(Message {
            domain: "example.com".parse().unwrap(),
            address: Default::default(),
            statement: None,
            uri: "did:key:example".parse().unwrap(),
            version: siwe::Version::V1,
            chain_id: 1,
            nonce: "mynonce1".into(),
            issued_at: "2022-06-21T12:00:00.000Z".parse().unwrap(),
            expiration_time: None,
            not_before: None,
            request_id: None,
            resources: vec![],
        })
        .with_capability(
            Capability::new("credential".into(), Some(vec!["present".into()])).unwrap(),
        )
        .with_capability(
            Capability::new("kepler".into(), None)
                .unwrap()
                .with_actions(
                    "kepler:ens:example.eth://default/kv".parse().unwrap(),
                    vec!["list".into(), "get".into(), "metadata".into()],
                )
                .with_actions(
                    "kepler:ens:example.eth://default/kv/dapp-space"
                        .parse()
                        .unwrap(),
                    vec![
                        "list".into(),
                        "get".into(),
                        "metadata".into(),
                        "put".into(),
                        "delete".into(),
                    ],
                ),
        )
        .build()
        .expect("failed to build SIWE delegation");

        assert_eq!(
            SIWE,
            msg.to_string(),
            "generated SIWE message did not match expectation"
        );
    }

    #[test]
    fn verify_statement() {
        let msg: Message = SIWE.parse().unwrap();
        assert!(
            verify_statement_matches_delegations(&msg)
                .expect("unable to parse resources as capabilities"),
            "statement did not match capabilities"
        );

        let mut altered_msg_1 = msg.clone();
        altered_msg_1
            .statement
            .iter_mut()
            .for_each(|statement| statement.push_str(" I am the walrus!"));
        assert!(
            !verify_statement_matches_delegations(&altered_msg_1)
                .expect("unable to parse resources as capabilities"),
            "altered statement incorrectly matched capabilities"
        );
    }
}

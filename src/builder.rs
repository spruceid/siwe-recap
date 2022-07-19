use crate::{Capability, Error, Namespace, Set, RESOURCE_PREFIX};

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use iri_string::types::UriString;
use serde_json::Value;
use siwe::Message;

/// Verifies that the encoded delegations match the human-readable description in the statement.
pub fn verify_statement_matches_delegations(message: &Message) -> Result<bool, Error> {
    let capabilities = extract_capabilities(message)?;
    let generated_statement = Builder::capabilities_to_statement(&capabilities);
    Ok(message.statement == Some(generated_statement))
}

/// Extract the encoded capabilities from a SIWE message.
pub fn extract_capabilities(message: &Message) -> Result<BTreeMap<Namespace, Capability>, Error> {
    message
        .resources
        .iter()
        .map(<(Namespace, Capability)>::from_resource)
        .collect()
}

/// Augments a SIWE message with encoded capability delegations.
#[derive(Default, Debug)]
pub struct Builder {
    capabilities: BTreeMap<Namespace, Capability>,
}

impl Builder {
    /// Initialise a new Builder.
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Inspect the default actions for a namespace.
    pub fn default_actions(&self, namespace: &Namespace) -> Option<&Set<String>> {
        self.capabilities
            .get(namespace)
            .as_ref()
            .map(|cap| &cap.default_actions)
    }

    /// Inspect the targeted actions for a namespace.
    pub fn actions(&self, namespace: &Namespace) -> Option<&BTreeMap<String, Set<String>>> {
        self.capabilities
            .get(namespace)
            .as_ref()
            .map(|cap| &cap.targeted_actions)
    }

    /// Inspect the extra fields for a namespace.
    pub fn extra_fields(&self, namespace: &Namespace) -> Option<&HashMap<String, Value>> {
        self.capabilities
            .get(namespace)
            .as_ref()
            .map(|cap| &cap.extra_fields)
    }

    /// Extend the set of default actions for a namespace.
    pub fn with_default_action<S>(mut self, namespace: &Namespace, action: S) -> Self
    where
        S: Into<String>,
    {
        self.namespace(namespace).default_actions.insert(action);
        self
    }

    /// Extend the set of default actions for a namespace.
    pub fn with_default_actions<I, S>(mut self, namespace: &Namespace, actions: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.namespace(namespace)
            .default_actions
            .insert_all(actions);
        self
    }

    /// Extend the set of actions for a target in a namespace.
    pub fn with_action<T, S>(mut self, namespace: &Namespace, target: T, action: S) -> Self
    where
        T: Into<String>,
        S: Into<String>,
    {
        let target = target.into();
        if let Some(actions) = self.namespace(namespace).targeted_actions.get_mut(&target) {
            actions.insert(action);
        } else {
            self.namespace(namespace)
                .targeted_actions
                .insert(target, Set::from_iter([action]));
        }
        self
    }

    /// Extend the set of actions for a target in a namespace.
    pub fn with_actions<I, S, T>(mut self, namespace: &Namespace, target: T, actions: I) -> Self
    where
        T: Into<String>,
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let target = target.into();
        if let Some(current_actions) = self.namespace(namespace).targeted_actions.get_mut(&target) {
            current_actions.insert_all(actions);
        } else {
            self.namespace(namespace)
                .targeted_actions
                .insert(target, Set::from_iter(actions));
        }
        self
    }

    /// Extend the extra fields for a namespace.
    ///
    /// This function performs a simple HashMap::extend, so does not merge [`serde_json::Value`]s.
    /// Any existing fields with a key matching the incoming fields will be overwritten.
    pub fn with_extra_fields(
        mut self,
        namespace: &Namespace,
        fields: HashMap<String, Value>,
    ) -> Self {
        self.namespace(namespace).extra_fields.extend(fields);
        self
    }

    /// Augment the SIWE message with encoded capabilities.
    pub fn build(&self, mut message: Message) -> Result<Message, Error> {
        let statement = Self::capabilities_to_statement(&self.capabilities);
        let resources = self
            .capabilities
            .iter()
            .map(|cap| cap.to_resource())
            .collect::<Result<Vec<UriString>, Error>>()?;

        message.statement = Some(statement);
        message.resources = resources;

        Ok(message)
    }

    fn namespace(&mut self, namespace: &Namespace) -> &mut Capability {
        if !self.capabilities.contains_key(namespace) {
            self.capabilities
                .insert(namespace.clone(), Capability::default());
        }

        // Safety: it has just been inserted or already exists, so can be safely unwrapped.
        self.capabilities.get_mut(namespace).unwrap()
    }

    fn capabilities_to_statement(capabilities: &BTreeMap<Namespace, Capability>) -> String {
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
            .flat_map(|(ns, cap)| cap.to_statement_lines(ns))
            .for_each(|line| {
                line_no += 1;
                // Ignore the error as write! is infallible for String.
                // See: https://rust-lang.github.io/rust-clippy/master/index.html#format_push_string
                let _ = write!(statement, " ({}) {}", line_no, line);
            });

        statement
    }
}

trait ToResource {
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
                rest.split_once(':')
                    .ok_or_else(|| Error::MissingBody(resource.to_string()))
            })
            .and_then(|(namespace, data)| {
                Capability::decode(data).and_then(|cap| Ok((namespace.parse()?, cap)))
            })
    }
}

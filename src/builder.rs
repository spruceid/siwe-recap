use crate::{
    capabilities_to_statement, translation::ToResource, Capability, Error, Namespace, Set,
};

use std::collections::{BTreeMap, HashMap};

use iri_string::types::UriString;
use serde_json::Value;
use siwe::Message;

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
        let statement = self.statement(&message.uri);
        let resources = self
            .capabilities
            .iter()
            .map(|cap| cap.to_resource())
            .collect::<Result<Vec<UriString>, Error>>()?;

        message.statement = match (message.statement, statement) {
            (s, None) => s,
            (None, s) => s,
            (Some(s), Some(t)) => Some(format!("{} {}", s, t)),
        };

        message.resources.extend(resources);

        Ok(message)
    }

    /// Generate a ReCap statement from capabilities and URI.
    pub fn statement(&self, uri: &UriString) -> Option<String> {
        capabilities_to_statement(&self.capabilities, uri)
    }

    fn namespace(&mut self, namespace: &Namespace) -> &mut Capability {
        if !self.capabilities.contains_key(namespace) {
            self.capabilities
                .insert(namespace.clone(), Capability::default());
        }

        // Safety: it has just been inserted or already exists, so can be safely unwrapped.
        self.capabilities.get_mut(namespace).unwrap()
    }
}

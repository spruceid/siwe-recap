use crate::{Error, Namespace, Set};
use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

fn eq_set_is_empty<T: Eq>(s: &Set<T>) -> bool {
    s.as_ref().is_empty()
}

/// Representation of a delegated Capability.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    #[serde(default, skip_serializing_if = "eq_set_is_empty")]
    /// The default actions that are allowed globally within this namespace.
    pub default_actions: Set<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    /// The actions that are allowed for the given target within this namespace.
    pub targeted_actions: BTreeMap<String, Set<String>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    /// Any additional information that is needed for the verifier to understand this Capability.
    ///
    /// This data is not encoded in the SIWE statement, so it must not contain any information that
    /// the verifier could use to extend the functionality defined by this capability. A good
    /// example of information you might encode here is the Cid of a previous delegation that this
    /// Capability is chaining from.
    pub extra_fields: HashMap<String, Value>,
}

impl Capability {
    /// Check if a particular action is allowed for the specified target, or is allowed globally.
    pub fn can(&self, target: &str, action: &str) -> bool {
        self.can_default(action)
            || self
                .targeted_actions
                .get(target)
                .map(|actions| actions.as_ref().contains_alike(action))
                .unwrap_or(false)
    }

    /// Check if a particular actions is allowed globally.
    pub fn can_default(&self, action: &str) -> bool {
        self.default_actions.as_ref().contains_alike(action)
    }

    pub(crate) fn encode(&self) -> Result<String, Error> {
        serde_json::to_vec(self)
            .map_err(Error::Ser)
            .map(|bytes| base64::encode_config(bytes, base64::URL_SAFE_NO_PAD))
    }

    pub(crate) fn decode(encoded: &str) -> Result<Self, Error> {
        base64::decode_config(encoded, base64::URL_SAFE_NO_PAD)
            .map_err(Error::Base64Decode)
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(Error::De))
    }

    pub(crate) fn to_statement_lines<'l>(
        &'l self,
        namespace: &'l Namespace,
    ) -> impl Iterator<Item = String> + 'l {
        let default_actions = std::iter::once(self.default_actions.as_ref().join(", "))
            .filter(|actions| !actions.is_empty())
            .map(move |actions| format!("{}: {} for any.", namespace, actions));

        let action_sets: Set<&[String]> =
            self.targeted_actions.values().map(AsRef::as_ref).collect();

        let targeted_actions = action_sets.into_iter().map(move |action_set| {
            let targets = self
                .targeted_actions
                .iter()
                .filter(|(_, actions)| actions.as_ref() == action_set)
                .map(|(target, _)| target.as_ref())
                .collect::<Vec<&str>>();
            format!(
                "{}: {} for {}.",
                namespace,
                action_set.join(", "),
                targets.join(", ")
            )
        });

        default_actions.chain(targeted_actions)
    }
}

trait Contains<T: ?Sized> {
    fn contains_alike(&self, other: &T) -> bool;
}

impl Contains<str> for [String] {
    fn contains_alike(&self, other: &str) -> bool {
        self.iter().any(|i| i == other)
    }
}

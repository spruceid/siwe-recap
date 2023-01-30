use crate::{Error, Namespace, Set};
use cid::Cid;
use std::collections::{BTreeMap, HashMap};

use crate::ability::{Ability, AbilityName, AbilityNamespace};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};

use iri_string::types::UriString;

fn eq_set_is_empty<T: Eq>(s: &Set<T>) -> bool {
    s.as_ref().is_empty()
}

/// Representation of a delegated Capability.
#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Capability {
    // a Vec allows for maintaining the ordering when de/serialized as a map
    /// The actions that are allowed for the given target within this namespace.
    #[serde(rename = "att")]
    #[serde_as(as = "BTreeMap<DisplayFromStr, BTreeMap<_, Vec<_>>>")]
    attenuations: Vec<(UriString, Vec<(Ability, Vec<Value>)>)>,

    /// Cids of parent delegations which these capabilities are attenuated from
    #[serde(rename = "prf")]
    #[serde_as(as = "Vec<DisplayFromStr>")]
    proof: Vec<Cid>,
}

#[derive(thiserror::Error, Debug)]
pub enum CanError<T, A> {
    #[error("Invalid Target: {0}")]
    InvalidTarget(T),
    #[error("Invalid Action: {0}")]
    InvalidAction(A),
}

impl Capability {
    /// Check if a particular action is allowed for the specified target, or is allowed globally.
    pub fn can<'l, T, A>(
        &'l self,
        target: T,
        action: A,
    ) -> Result<Option<&'l [Value]>, CanError<T::Error, A::Error>>
    where
        T: TryInto<UriString>,
        A: TryInto<Ability>,
    {
        Ok(self.can_do(
            &target.try_into().map_err(CanError::InvalidTarget)?,
            &action.try_into().map_err(CanError::InvalidAction)?,
        ))
    }

    pub fn can_do<'l>(&'l self, target: &UriString, action: &Ability) -> Option<&'l [Value]> {
        self.attenuations.iter().find_map(|(r, actions)| {
            if r == target {
                actions.iter().find_map(|(a, nb)| {
                    if a == action {
                        Some(nb.as_slice())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
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

    fn to_line_groups<'l>(
        &'l self,
    ) -> impl Iterator<Item = (&'l UriString, &'l AbilityNamespace, Vec<&'l AbilityName>)> + 'l
    {
        self.attenuations
            .iter()
            .map(|(resource, abilities)| {
                // group abilities by namespace
                abilities
                    .iter()
                    .fold(
                        HashMap::<&AbilityNamespace, Vec<&AbilityName>>::new(),
                        |mut map, (ability, _)| {
                            map.entry(ability.namespace())
                                .or_default()
                                .push(ability.name());
                            map
                        },
                    )
                    .into_iter()
                    .map(move |(namespace, names)| (resource, namespace, names))
            })
            .flatten()
    }

    pub(crate) fn to_statement_lines<'l>(&'l self) -> impl Iterator<Item = String> + 'l {
        self.to_line_groups().map(|(resource, namespace, names)| {
            format!(
                "\"{}\": {} for \"{}\".",
                namespace,
                names
                    .iter()
                    .map(|an| format!("\"{}\"", an))
                    .collect::<Vec<String>>()
                    .join(", "),
                resource
            )
        })
    }
}

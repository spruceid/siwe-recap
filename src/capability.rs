use crate::{capabilities_to_statement, translation::ToResource, Error};
use cid::Cid;
use indexmap::{map::IndexMap, set::IndexSet};

use crate::ability::{Ability, AbilityName, AbilityNamespace};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};

use iri_string::types::UriString;
use siwe::Message;

/// Representation of a set of delegated Capabilities.
#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Capability {
    // a Vec allows for maintaining the ordering when de/serialized as a map
    /// The actions that are allowed for the given target within this namespace.
    #[serde(rename = "att")]
    #[serde_as(as = "IndexMap<DisplayFromStr, _>")]
    attenuations: IndexMap<UriString, IndexMap<Ability, Vec<IndexMap<String, Value>>>>,

    /// Cids of parent delegations which these capabilities are attenuated from
    #[serde(rename = "prf")]
    #[serde_as(as = "IndexSet<DisplayFromStr>")]
    proof: IndexSet<Cid>,
}

#[derive(thiserror::Error, Debug)]
pub enum ConvertError<T, A> {
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
    ) -> Result<
        Option<impl Iterator<Item = &'l IndexMap<String, Value>>>,
        ConvertError<T::Error, A::Error>,
    >
    where
        T: TryInto<UriString>,
        A: TryInto<Ability>,
    {
        Ok(self.can_do(
            &target.try_into().map_err(ConvertError::InvalidTarget)?,
            &action.try_into().map_err(ConvertError::InvalidAction)?,
        ))
    }

    pub fn can_do<'l>(
        &'l self,
        target: &UriString,
        action: &Ability,
    ) -> Option<impl Iterator<Item = &'l IndexMap<String, Value>>> {
        self.attenuations
            .get(target)
            .and_then(|m| m.get(action))
            .map(|v| v.iter())
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

    pub fn merge(mut self, other: Self) -> Self {
        self.proof.extend(other.proof);
        for (uri, abs) in other.attenuations.into_iter() {
            for (ab, nbs) in abs.into_iter() {
                self.attenuations
                    .entry(uri)
                    .or_default()
                    .entry(ab)
                    .or_default()
                    .extend(nbs);
            }
        }
        self
    }

    pub fn with_action<T, A>(
        mut self,
        target: T,
        action: A,
        nb: impl IntoIterator<Item = IndexMap<String, Value>>,
    ) -> Result<Self, ConvertError<T::Error, A::Error>>
    where
        T: TryInto<UriString>,
        A: TryInto<Ability>,
    {
        self.attenuations
            .entry(target.try_into().map_err(ConvertError::InvalidTarget)?)
            .or_default()
            .entry(action.try_into().map_err(ConvertError::InvalidAction)?)
            .or_default()
            .extend(nb);
        Ok(self)
    }

    pub fn abilities(
        &self,
    ) -> impl Iterator<Item = (&UriString, &IndexMap<Ability, Vec<IndexMap<String, Value>>>)> {
        self.attenuations.iter()
    }

    pub fn abilities_for<T>(
        &self,
        target: T,
    ) -> Result<Option<impl Iterator<Item = (&Ability, &Vec<IndexMap<String, Value>>)>>, T::Error>
    where
        T: TryInto<UriString>,
    {
        Ok(self.attenuations.get(&target.try_into()?).map(|m| m.iter()))
    }

    pub fn proof(&self) -> impl Iterator<Item = &Cid> {
        self.proof.iter()
    }

    pub fn with_proof(mut self, proof: &Cid) -> Self {
        self.proof.insert(*proof);
        self
    }

    pub fn with_proofs<'l>(mut self, proofs: impl IntoIterator<Item = &'l Cid>) -> Self {
        self.proof.extend(proofs);
        self
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
                        IndexMap::<&AbilityNamespace, Vec<&AbilityName>>::new(),
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

    pub fn build_message(&self, mut message: Message) -> Result<Message, Error> {
        let statement = capabilities_to_statement(self, &message.uri);
        let encoded = self.to_resource()?;
        message.resources.push(encoded);
        message.statement = Some([message.statement.unwrap_or_default(), statement].concat());
        Ok(message)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deser() {
        let ser = r#"{"att":{"http://example.com/public/photos/":{"crud/delete":[]},"mailto:username@example.com":{"msg/send":[{"to":"someone@email.com"},{"to":"joe@email.com"}]"msg/receive":[{"max_count":5,"templates":["newsletter","marketing"]}]}},"prf":["zM5K3WPEexRZN9x7ikg5LoijNjU2ytFTGJ6eeruGiN3Q9xkfGpMVei9cz5jMZAFUsmxXi9k"]}"#;
        let cap: Capability = serde_json::from_str(ser).unwrap();
        let reser = serde_json::to_string(&cap).unwrap();
        assert_eq!(ser, reser);
    }
}

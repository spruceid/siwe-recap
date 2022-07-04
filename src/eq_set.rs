
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Default)]
pub struct EqSet<T: Eq>(Vec<T>);

impl<T: Eq> FromIterator<T> for EqSet<T> {
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = i.into_iter();
        let inner = Vec::with_capacity(iter.size_hint().0);
        let mut this = Self(inner);
        this.insert_all(iter);
        this
    }
}

impl<T: Eq> IntoIterator for EqSet<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Eq> AsRef<[T]> for EqSet<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T: Eq> EqSet<T> {
    pub fn insert(&mut self, s: T) -> bool {
        if !self.0.contains(&s) {
            self.0.push(s);
            return true;
        }
        false
    }

    pub fn insert_all<I: Iterator<Item = T>>(&mut self, ts: I) {
        ts.for_each(|t| {
            self.insert(t);
        })
    }
}

impl<T: Eq + Serialize> Serialize for EqSet<T> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(s)
    }
}

impl<'de, T: Eq + Deserialize<'de>> Deserialize<'de> for EqSet<T> {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(Deserialize::deserialize(d)?))
    }
}

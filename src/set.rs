use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Default)]
/// A simple Set implementation that de-duplicates elements by [`Eq`].
///
/// Set is a glorified Vec with insertion checks, so to perform any read actions on a Set you
/// should use the [`AsRef`] implementation to convert to a slice.
pub struct Set<T: Eq>(Vec<T>);

impl<T: Eq, S: Into<T>> FromIterator<S> for Set<T> {
    fn from_iter<I>(i: I) -> Self
    where
        I: IntoIterator<Item = S>,
    {
        let iter = i.into_iter();
        let inner = Vec::with_capacity(iter.size_hint().0);
        let mut this = Self(inner);
        this.insert_all(iter);
        this
    }
}

impl<T: Eq> IntoIterator for Set<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Eq> AsRef<[T]> for Set<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T: Eq> Set<T> {
    /// Insert a new element.
    ///
    /// Returns true if inserted, false if the element already exists in the set.
    pub fn insert<S: Into<T>>(&mut self, s: S) -> bool {
        let t = s.into();
        if !self.0.contains(&t) {
            self.0.push(t);
            return true;
        }
        false
    }

    /// Insert multiple new elements.
    pub fn insert_all<I, S>(&mut self, ts: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<T>,
    {
        ts.into_iter().for_each(|t| {
            self.insert(t);
        })
    }
}

impl<T: Eq + Serialize> Serialize for Set<T> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(s)
    }
}

impl<'de, T: Eq + Deserialize<'de>> Deserialize<'de> for Set<T> {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(Deserialize::deserialize(d)?))
    }
}

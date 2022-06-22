use std::collections::BTreeMap;

use iri_string::types::UriString;
use serde::{
    de::{Deserialize, Deserializer, Error as DeError},
    ser::{Serialize, Serializer},
};

pub fn serialize<V, S>(map: &BTreeMap<UriString, V>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Serialize,
{
    map.iter()
        .map(|(k, v)| (k.as_str().to_string(), v))
        .collect::<BTreeMap<String, &V>>()
        .serialize(s)
}

pub fn deserialize<'l, V, D>(d: D) -> Result<BTreeMap<UriString, V>, D::Error>
where
    D: Deserializer<'l>,
    V: Deserialize<'l>,
{
    BTreeMap::<String, V>::deserialize(d)?
        .into_iter()
        .map(|(k, v)| Ok((UriString::try_from(k).map_err(D::Error::custom)?, v)))
        .collect()
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use iri_string::types::UriString;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
    struct MapWrapper(#[serde(with = "super")] BTreeMap<UriString, ()>);

    #[test]
    fn ser() {
        let mut map = MapWrapper::default();
        let uri = "hello://world";
        map.0.insert(UriString::try_from(uri).unwrap(), ());
        let map_str = serde_json::to_string(&map).expect("failed to serialize");
        assert_eq!(
            format!(r#"{{"{}":null}}"#, uri),
            map_str,
            "serialized map did not match expectation"
        );
    }

    #[test]
    fn de_empty() {
        let map =
            serde_json::from_str::<MapWrapper>("{}").expect("failed to deserialize empty map");
        assert!(map.0.is_empty(), "expected an empty map")
    }

    #[test]
    fn de_invalid_uri() {
        serde_json::from_str::<MapWrapper>(r#"{"helloworld": null}"#)
            .expect_err("successfully parsed invalid uri");
    }

    #[test]
    fn de_valid_uri() {
        serde_json::from_str::<MapWrapper>(r#"{"hello://world": null}"#)
            .expect("successfully parsed invalid uri");
        serde_json::from_str::<MapWrapper>(r#"{"hello:world": null}"#)
            .expect("successfully parsed invalid uri");
    }

    #[test]
    fn roundtrip() {
        let mut map = MapWrapper::default();
        let uri = "hello://world";
        map.0.insert(UriString::try_from(uri).unwrap(), ());
        let map_str = serde_json::to_string(&map).expect("failed to serialize");
        let roundtripped: MapWrapper =
            serde_json::from_str(&map_str).expect("failed to deserialize");
        assert_eq!(
            map, roundtripped,
            "serde implementation could no perform a consistent roundtrip"
        )
    }
}

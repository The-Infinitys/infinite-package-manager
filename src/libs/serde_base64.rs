pub mod serde_base64 {
    use base64::{engine::general_purpose, Engine as _};
    use serde::{de, Deserializer, Serializer, Deserialize};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&general_purpose::STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        general_purpose::STANDARD
            .decode(s)
            .map_err(de::Error::custom)
    }
}
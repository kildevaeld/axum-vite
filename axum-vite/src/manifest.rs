use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ManifestEntry {
    pub file: String,
    #[serde(default)]
    pub css: Vec<String>,
    #[serde(default, rename = "dynamicImports")]
    pub dynamic_imports: Vec<String>,
    #[serde(default, rename = "isEntry")]
    pub is_entry: bool,
    #[serde(default)]
    pub imports: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Manifest(HashMap<String, ManifestEntry>);

impl core::ops::Deref for Manifest {
    type Target = HashMap<String, ManifestEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct SSRManifest(HashMap<String, Vec<String>>);

impl core::ops::Deref for SSRManifest {
    type Target = HashMap<String, Vec<String>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

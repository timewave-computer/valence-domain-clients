use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Header data
    pub valence: ManifestHeader,

    /// Circuits to be compiled.
    pub circuit: HashMap<String, Circuit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestHeader {
    /// Program name.
    pub name: String,

    /// Program version.
    pub version: String,

    /// Folder to store the artifacts.
    pub artifacts: PathBuf,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Circuit {
    /// Circuit crate name
    pub circuit: Option<String>,

    /// Controller crate name
    pub controller: Option<String>,
}

impl Default for ManifestHeader {
    fn default() -> Self {
        Self {
            name: "valence-program".into(),
            version: "0.1.0".into(),
            artifacts: ".valence/artifacts".into(),
        }
    }
}

impl fmt::Display for Manifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = toml::to_string_pretty(self).map_err(|_| fmt::Error)?;

        write!(f, "{out}")
    }
}

impl Manifest {
    pub fn with_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.valence.name = name.as_ref().into();
        self
    }

    pub fn with_version<S: AsRef<str>>(mut self, version: S) -> Self {
        self.valence.version = version.as_ref().into();
        self
    }

    pub fn with_artifacts<P: AsRef<Path>>(mut self, artifacts: P) -> Self {
        self.valence.artifacts = artifacts.as_ref().into();
        self
    }

    pub fn with_circuit<N>(
        mut self,
        name: N,
        circuit: Option<&str>,
        controller: Option<&str>,
    ) -> Self
    where
        N: AsRef<str>,
    {
        self.circuit.insert(
            name.as_ref().into(),
            Circuit {
                circuit: circuit.map(String::from),
                controller: controller.map(String::from),
            },
        );
        self
    }
}

#[test]
fn parse_manifest_works() {
    let manifest = r#"[valence]
    name = "cool program"
    version = "vx.y.z"
    artifacts = ".valence/artifacts"

    [circuit.foo]
circuit = "bar"
controller = "baz"

[circuit.xxx]
circuit = "yyy"
controller = "zzz"
"#;

    toml::from_str::<Manifest>(manifest).unwrap();
}

use std::fmt::Write as _;

use anyhow::Result;
use parse_display::Display;

use crate::{
    codegen::{Builder, Codegen},
    utils::{FAKE_HASH, fod_hash},
};

#[derive(Clone, Copy, Display)]
#[display("BuildMavenPackage")]
pub struct BuildMavenPackage;

impl Builder for BuildMavenPackage {
    fn function(&self) -> &'static str {
        "maven.buildMavenPackage"
    }

    async fn after_src(&self, cg: &mut Codegen<'_>) -> Result<String> {
        let mut out = String::new();
        let hash = if cg.layout.has_pom_file
            && let Some(hash) = fod_hash(format!(
            r#"(import({}){{}}).maven.buildMavenPackage{{pname={:?};version={:?};src={};mvnHash="{FAKE_HASH}";}}"#,
            cg.nixpkgs, cg.pname, cg.version, cg.src,
        )).await {
            hash
        } else {
            FAKE_HASH.into()
        };

        writeln!(out, "  mvnHash = \"{hash}\";\n")?;
        Ok(out)
    }
}

#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::format_push_string
)]

use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

const BASE_LOCALE: &str = "en";
const DEFAULT_RELATIVE_DIR: &str = "../../packages/i18n/messages";

fn main() {
    println!("cargo:rerun-if-env-changed=CMS_I18N_MESSAGES_DIR");
    println!("cargo:rerun-if-changed=build.rs");

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let dir = env::var_os("CMS_I18N_MESSAGES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest.join(DEFAULT_RELATIVE_DIR));
    println!("cargo:rerun-if-changed={}", dir.display());

    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read `{}`: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("i18n");
    fs::create_dir_all(&out_dir).expect("create OUT_DIR/i18n");

    // 1. Generate i18n_embedded_messages.rs for cms_infra::i18n
    let mut i18n_gen = String::from("pub(super) static MESSAGES: &[(&str, &str)] = &[\n");
    let mut email_gen = String::from("pub(crate) static MESSAGES: &[(&str, &str)] = &[\n");
    let mut saw_base = false;

    for path in &files {
        let locale = path.file_stem().and_then(|s| s.to_str()).expect("utf-8");
        let raw = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let doc: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{}: invalid JSON: {e}", path.display()));

        if locale == BASE_LOCALE {
            saw_base = true;
        }

        // i18n subset (api_error_ + email_)
        let i18n_subset: BTreeMap<&str, &serde_json::Value> = doc
            .iter()
            .filter(|(k, _)| k.starts_with("api_error_") || k.starts_with("email_"))
            .map(|(k, v)| (k.as_str(), v))
            .collect();

        if !i18n_subset.is_empty() {
            let target = out_dir.join(format!("{locale}_i18n.json"));
            fs::write(&target, serde_json::to_string(&i18n_subset).expect("serialize")).expect("write");
            let safe_target = target.to_string_lossy().replace('\\', "/");
            i18n_gen.push_str(&format!("    ({locale:?}, include_str!({safe_target:?})),\n"));
        }

        // email subset (email_)
        let email_subset: BTreeMap<&str, &serde_json::Value> = doc
            .iter()
            .filter(|(k, _)| k.starts_with("email_"))
            .map(|(k, v)| (k.as_str(), v))
            .collect();

        if !email_subset.is_empty() {
            let target = out_dir.join(format!("{locale}_email.json"));
            fs::write(&target, serde_json::to_string(&email_subset).expect("serialize")).expect("write");
            let safe_target = target.to_string_lossy().replace('\\', "/");
            email_gen.push_str(&format!("    ({locale:?}, include_str!({safe_target:?})),\n"));
        }
    }
    i18n_gen.push_str("];\n");
    email_gen.push_str("];\n");

    assert!(saw_base, "Base locale `{BASE_LOCALE}.json` missing in `{}`", dir.display());
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("i18n_embedded_messages.rs"),
        i18n_gen,
    )
    .expect("write i18n_embedded_messages.rs");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("email_embedded_messages.rs"),
        email_gen,
    )
    .expect("write email_embedded_messages.rs");
}

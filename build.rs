use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    if let Err(err) = run() {
        panic!("build.rs failed: {err}");
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    generate_graphql_queries(out_path)?;
    generate_language_map(out_path)?;

    Ok(())
}

// --- GraphQL query embedding ---

fn generate_graphql_queries(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let queries_dir = Path::new("src/api/queries");

    let user_query = fs::read_to_string(queries_dir.join("user.graphql"))
        .map_err(|e| format!("Failed to read user.graphql: {e}"))?;
    let repo_query = fs::read_to_string(queries_dir.join("repo.graphql"))
        .map_err(|e| format!("Failed to read repo.graphql: {e}"))?;

    let generated = format!(
        r##"/// GraphQL query for user profile and contributions (from user.graphql)
pub const USER_QUERY: &str = r#"{user_query}"#;

/// GraphQL query for repository stats with author filter (from repo.graphql)
pub const REPO_QUERY: &str = r#"{repo_query}"#;
"##,
        user_query = user_query.trim(),
        repo_query = repo_query.trim(),
    );

    fs::write(out_dir.join("queries.rs"), generated)
        .map_err(|e| format!("Failed to write queries.rs: {e}"))?;

    println!("cargo:rerun-if-changed=src/api/queries/user.graphql");
    println!("cargo:rerun-if-changed=src/api/queries/repo.graphql");

    Ok(())
}

// --- Language map from Linguist languages.yml + overrides ---

#[derive(Debug, Deserialize)]
struct LanguageEntry {
    #[serde(rename = "type")]
    lang_type: Option<String>,
    extensions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Overrides {
    extensions: HashMap<String, String>,
}

fn generate_language_map(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Parse languages.yml
    let yml_path = Path::new("data/languages.yml");
    let content = fs::read_to_string(yml_path)
        .map_err(|e| format!("Failed to read {}: {e}", yml_path.display()))?;

    let languages: BTreeMap<String, LanguageEntry> = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse languages.yml: {e}"))?;

    // Build name -> type lookup for override validation
    let name_to_type: HashMap<String, String> = languages
        .iter()
        .map(|(name, entry)| {
            (
                name.clone(),
                entry.lang_type.clone().unwrap_or("data".to_string()),
            )
        })
        .collect();

    // Parse overrides
    let overrides_path = Path::new("data/overrides.toml");
    let overrides: Overrides = {
        let content = fs::read_to_string(overrides_path)
            .map_err(|e| format!("Failed to read {}: {e}", overrides_path.display()))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse overrides.toml: {e}"))?
    };

    // Validate overrides reference real languages
    for (ext, lang_name) in &overrides.extensions {
        if !name_to_type.contains_key(lang_name) {
            return Err(format!(
                "overrides.toml: extension '{ext}' maps to unknown language '{lang_name}'"
            )
            .into());
        }
    }

    // Build extension -> (language_name, language_type) map.
    // First pass: populate from languages.yml (alphabetical first-wins for conflicts).
    let mut ext_map: BTreeMap<String, (String, String)> = BTreeMap::new();
    for (name, entry) in &languages {
        let lang_type = entry.lang_type.as_deref().unwrap_or("data");
        if let Some(ref exts) = entry.extensions {
            for ext in exts {
                let key = ext.strip_prefix('.').unwrap_or(ext).to_lowercase();
                ext_map
                    .entry(key)
                    .or_insert_with(|| (name.clone(), lang_type.to_string()));
            }
        }
    }

    // Second pass: apply overrides (wins over first-pass).
    for (ext, lang_name) in &overrides.extensions {
        let lang_type = name_to_type
            .get(lang_name)
            .expect("validated above")
            .clone();
        ext_map.insert(ext.to_lowercase(), (lang_name.clone(), lang_type));
    }

    // Generate Rust lookup function
    let mut code = String::from(
        "/// Auto-generated from GitHub Linguist languages.yml + data/overrides.toml\n\
         /// Returns (language_name, language_type) for a file extension (without leading dot).\n\
         pub fn extension_to_language(ext: &str) -> Option<(&'static str, &'static str)> {\n\
         \x20   match ext {\n",
    );

    for (ext, (name, lang_type)) in &ext_map {
        let escaped_name = name.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_type = lang_type.replace('\\', "\\\\").replace('"', "\\\"");
        code.push_str(&format!(
            "        \"{ext}\" => Some((\"{escaped_name}\", \"{escaped_type}\")),\n"
        ));
    }

    code.push_str("        _ => None,\n    }\n}\n");

    fs::write(out_dir.join("languages.rs"), code)
        .map_err(|e| format!("Failed to write languages.rs: {e}"))?;

    println!("cargo:rerun-if-changed=data/languages.yml");
    println!("cargo:rerun-if-changed=data/overrides.toml");

    Ok(())
}

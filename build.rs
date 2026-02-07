use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("queries.rs");

    let queries_dir = Path::new("src/api/queries");

    let user_query =
        fs::read_to_string(queries_dir.join("user.graphql")).expect("Failed to read user.graphql");
    let repo_query =
        fs::read_to_string(queries_dir.join("repo.graphql")).expect("Failed to read repo.graphql");

    // Use r##"..."## to allow r#"..."# inside the generated code
    let generated = format!(
        r##"/// GraphQL query for user profile and contributions (from user.graphql)
pub const USER_QUERY: &str = r#"{user_query}"#;

/// GraphQL query for repository stats with author filter (from repo.graphql)
pub const REPO_QUERY: &str = r#"{repo_query}"#;
"##,
        user_query = user_query.trim(),
        repo_query = repo_query.trim(),
    );

    fs::write(&dest_path, generated).expect("Failed to write queries.rs");

    // Rerun if query files change
    println!("cargo:rerun-if-changed=src/api/queries/user.graphql");
    println!("cargo:rerun-if-changed=src/api/queries/repo.graphql");
}

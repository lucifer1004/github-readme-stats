use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LanguageUsage {
    pub name: String,
    pub changes: u64,
    pub percent: f64,
}

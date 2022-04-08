pub mod context;
pub mod types;

#[derive(Debug, Clone)]
pub enum TokenValue {
    Null,
    String(String),
}

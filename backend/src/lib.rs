//! PatchWork backend library. Frozen interfaces: `graph` (envelope + run_template).
pub mod graph;
pub mod templates;
pub mod llm;
pub mod github;
pub mod config;
pub mod events;
pub mod agents;
pub mod api;

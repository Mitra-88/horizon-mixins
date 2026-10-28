pub mod bridges;
pub mod diagnostic;
pub mod helpers;
pub mod index;
pub mod manifest;
pub mod plan;
pub mod scopes;
pub mod sites;
pub mod slices;
pub mod source;
pub use diagnostic::{Diagnostic, Result};
pub use plan::{Plan, PlannedInjection, build_plan};

pub mod modules;

mod standard_macros;
pub mod state;

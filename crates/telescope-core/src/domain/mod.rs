//! Pure domain logic: no tauri, reqwest, or tokio imports anywhere in this
//! tree. Every module is unit-testable without an app handle, a runtime, or
//! the network. Side effects are returned as data (effect enums) and executed
//! by the command/service layer that drives these machines.

pub mod deeplink;
pub mod dscan;
pub mod intel_reducer;
pub mod lookup;
pub mod sde_lifecycle;
pub mod threat;
pub mod version;

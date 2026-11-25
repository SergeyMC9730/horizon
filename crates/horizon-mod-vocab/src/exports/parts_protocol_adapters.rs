/// Contracts for native client-side wire protocol extensions.
pub use horizon_mod_runtime::protocol_version;
// The generated native test implementation of the wasm guest names handler traits at the root.
pub use horizon_mod_runtime::protocol_version::AdapterFactory;
mod __private {
    pub mod game {
        pub use crate::parts::protocol_adapter;
    }
}

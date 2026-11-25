//! Load-owned factories for remote client connections. Selection happens once, not per packet.
use super::*;
use horizon_mod_runtime::protocol_version::{AdapterFactory, ProtocolPair};

/// Native protocol extensions registered at mod load.
pub trait ProtocolAdaptersRegistrar {
    /// Register one factory for this exact client/server pair. Equal endpoints and duplicate owners are refused.
    /// Not on wasm: the reader and writer borrow packet buffers and share a native session across threads.
    #[cfg(not(target_arch = "wasm32"))]
    fn register_protocol_adapter(&mut self, pair: ProtocolPair, factory: Arc<dyn AdapterFactory>);
}

/// Look up an enabled owner's factory without opening a session. Called once at remote login.
pub fn protocol_adapter(pair: ProtocolPair) -> Option<(&'static str, Arc<dyn AdapterFactory>)> {
    let (owner, index, factory) = installed()?.keys.protocol_adapters.get(&pair)?;
    crate::mod_index_on(*index).then(|| (*owner, Arc::clone(factory)))
}

use super::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::protocol_version::{AdapterFactory, ProtocolPair};

impl ProtocolAdaptersRegistrar for PartRegistrar {
    #[cfg(not(target_arch = "wasm32"))]
    fn register_protocol_adapter(&mut self, pair: ProtocolPair, factory: Arc<dyn AdapterFactory>) {
        if self.mod_wasm || pair.native == pair.remote || pair.native < 0 || pair.remote < 0 {
            self.refused.push(format!("{}: invalid native protocol adapter {pair:?}", self.mod_id));
            return;
        }
        if let std::collections::hash_map::Entry::Occupied(entry) =
            self.parts.protocol_adapters.entry(pair)
        {
            self.refused.push(format!(
                "protocol adapter {pair:?} requested by both {} and {}",
                entry.get().0,
                self.mod_id
            ));
            return;
        }
        self.parts.protocol_adapters.insert(
            pair,
            (self.mod_id, self.mod_index, factory),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol_version::{PacketAction, PacketContext, PacketSink, PacketTranslator, TranslationError};
    use horizon_mod_runtime::protocol_version::{Direction, ProtocolPair};

    struct Factory;
    impl AdapterFactory for Factory {
        fn open(&self, _: ProtocolPair) -> Result<Box<dyn PacketTranslator>, TranslationError> {
            Ok(Box::new(Translator))
        }
    }
    struct Translator;
    impl PacketTranslator for Translator {
        fn translate(&self, packet: PacketContext<'_>, _: &mut dyn PacketSink) -> Result<PacketAction, TranslationError> {
            Ok(PacketAction::Forward { id: packet.id })
        }
    }

    #[test]
    fn adapter_pairs_are_unique_per_owner_and_rollback_with_refused_registration() {
        let pair = ProtocolPair { native: 777, remote: 340 };
        let mut registrar = PartRegistrar::new("first");
        let rollback = registrar.registration_checkpoint();
        registrar.register_protocol_adapter(pair, Arc::new(Factory));
        assert_eq!(registrar.parts.protocol_adapters.len(), 1);

        registrar.mod_id = "second";
        registrar.register_protocol_adapter(pair, Arc::new(Factory));
        assert_eq!(registrar.parts.protocol_adapters.len(), 1);
        assert_eq!(registrar.refused.len(), 1);
        assert!(registrar.refused[0].contains("both first and second"));

        rollback(&mut registrar);
        assert!(registrar.parts.protocol_adapters.is_empty());
    }

    #[test]
    fn invalid_native_and_same_version_pairs_are_refused() {
        for pair in [
            ProtocolPair { native: 777, remote: 777 },
            ProtocolPair { native: -1, remote: 340 },
            ProtocolPair { native: 777, remote: -1 },
        ] {
            let mut registrar = PartRegistrar::new("invalid");
            registrar.register_protocol_adapter(pair, Arc::new(Factory));
            assert!(registrar.parts.protocol_adapters.is_empty());
            assert_eq!(registrar.refused.len(), 1);
        }
    }

    #[test]
    fn wasm_owner_cannot_register_a_native_adapter() {
        let mut registrar = PartRegistrar::new("wasm");
        registrar.mod_wasm = true;
        registrar.register_protocol_adapter(
            ProtocolPair { native: 777, remote: 340 },
            Arc::new(Factory),
        );
        assert!(registrar.parts.protocol_adapters.is_empty());
        assert_eq!(registrar.refused.len(), 1);
        let _ = Direction::Clientbound;
    }
}

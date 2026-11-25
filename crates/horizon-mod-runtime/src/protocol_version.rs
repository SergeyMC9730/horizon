//! Native, per-connection protocol translation. No socket, game host or account credentials cross this boundary.

/// Left endpoint is the client codec; right endpoint is the wire protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProtocolPair {
    /// Client codec number.
    pub native: i32,
    /// Remote server's protocol number.
    pub remote: i32,
}

/// The protocol phase of a connection endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// The protocol's initial intention packet.
    Handshake,
    /// Server list ping and pong.
    Status,
    /// Login start, encryption and profile completion.
    Login,
    /// Registry and feature negotiation after login.
    Configuration,
    /// World and gameplay traffic.
    Play,
}

/// The endpoint a packet is being delivered to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// From the server, toward the native client codec.
    Clientbound,
    /// From the native client codec, toward the wire.
    Serverbound,
}

/// One complete plaintext packet. The ID and payload are from the source protocol;
/// framing, compression, encryption and the ID prefix have already been removed.
/// `native_phase` is the pinned codec's phase; the translator owns the remote phase.
#[derive(Clone, Copy, Debug)]
pub struct PacketContext<'a> {
    /// Which side supplied the packet, expressed as the direction of travel.
    pub direction: Direction,
    /// The phase of Horizon's codec for this direction, before processing this packet.
    /// A legacy server may already be in play while Horizon is in configuration.
    pub native_phase: Phase,
    /// Numeric packet id in the source direction and phase.
    pub id: i32,
    /// Payload only, borrowed until the callback returns.
    pub payload: &'a [u8],
}

/// Disposition of the original packet; any staged emissions follow it in call order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketAction {
    /// Both layouts are known equivalent. Reuse the original payload without copying it.
    Forward {
        /// Packet id at the destination.
        id: i32,
    },
    /// Consume the input. No emissions means drop; one or more means replace or expand.
    Consume,
}

/// A failed connection cannot resume with the original bytes.
#[derive(Debug)]
pub struct TranslationError(pub String);

impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for TranslationError {}

/// Per-call staging, not socket I/O. The engine enforces the remaining framed-byte budget,
/// packet-count budget, and destination frame limits. Failure invalidates the whole batch,
/// even if the translator ignores the error. A sink belongs to one direction; the translator
/// cannot emit a packet back toward the side that supplied this input.
pub trait PacketSink {
    /// Stage a destination-format packet. Opposite-direction replies do not re-enter translation.
    fn emit(
        &mut self,
        direction: Direction,
        id: i32,
        payload: Vec<u8>,
    ) -> Result<(), TranslationError>;
}

/// One object per connection, shared by its reader and writer. Dropped on disconnect.
/// Never wait for a packet from the other direction inside a callback.
pub trait PacketTranslator: Send + Sync {
    /// Translate one packet. On error nothing from this call is delivered.
    fn translate(
        &self,
        packet: PacketContext<'_>,
        output: &mut dyn PacketSink,
    ) -> Result<PacketAction, TranslationError>;
}

/// Startup registration stores factories, not live sessions. Share immutable mapping tables
/// across opens; keep phase, entity and transaction state in the returned translator.
pub trait AdapterFactory: Send + Sync + 'static {
    /// Open an independent translator before the first handshake to this server.
    fn open(&self, pair: ProtocolPair) -> Result<Box<dyn PacketTranslator>, TranslationError>;
}

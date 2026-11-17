//! Перенос `net.minecraft.commands.arguments.ArgumentSignatures`.

use crate::network::chat::message_signature::MessageSignature;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `MAX_ARGUMENT_COUNT`.
pub const MAX_ARGUMENT_COUNT: usize = 8;
/// `MAX_ARGUMENT_NAME_LENGTH`.
pub const MAX_ARGUMENT_NAME_LENGTH: usize = 16;

/// `ArgumentSignatures.Entry`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentSignatureEntry {
    /// `name`.
    pub name: String,
    /// `signature`.
    pub signature: MessageSignature,
}

impl ArgumentSignatureEntry {
    /// `new Entry(FriendlyByteBuf)` — readUtf(16) + signature.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Self {
        let name = buffer.read_utf_n(MAX_ARGUMENT_NAME_LENGTH).expect("invalid argument name");
        let signature = MessageSignature::read(buffer);
        Self { name, signature }
    }

    /// `write`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_utf_n(&self.name, MAX_ARGUMENT_NAME_LENGTH).expect("argument name too long");
        MessageSignature::write(buffer, &self.signature);
    }
}

/// `ArgumentSignatures` — список подписанных аргументов команды.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArgumentSignatures {
    /// `entries`.
    pub entries: Vec<ArgumentSignatureEntry>,
}

impl ArgumentSignatures {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self { entries: Vec::new() }
    }

    /// `new ArgumentSignatures(FriendlyByteBuf)` — collection max 8.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Self {
        let entries = buffer
            .read_collection(|_| Vec::new(), |b| ArgumentSignatureEntry::read(b))
            .into_iter()
            .take(MAX_ARGUMENT_COUNT)
            .collect();
        Self { entries }
    }

    /// `write`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_collection(&self.entries, |b, e| e.write(b));
    }

    /// `signCommand` — подписать аргументы команды через Signer.
    /// `arguments` — пары (name, value) подписываемых аргументов.
    pub fn sign_command<S: FnMut(&str) -> Option<MessageSignature>>(
        arguments: &[(String, String)],
        mut signer: S,
    ) -> Self {
        let entries = arguments
            .iter()
            .filter_map(|(name, value)| {
                signer(value).map(|sig| ArgumentSignatureEntry {
                    name: name.clone(),
                    signature: sig,
                })
            })
            .collect();
        Self { entries }
    }
}

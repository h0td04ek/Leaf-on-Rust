//! Перенос `net.minecraft.network.codec.StreamCodec`,
//! `StreamEncoder`, `StreamDecoder`, `StreamMemberEncoder`.
//!
//! Java-версии бросают unchecked DecoderException/EncoderException —
//! здесь паникуем (panic = аналог unchecked exception).

use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `StreamEncoder<B, T>`
pub trait StreamEncoder<B, T> {
    fn encode(&mut self, buf: &mut B, value: &T);
}

/// `StreamDecoder<B, T>`
pub trait StreamDecoder<B, T> {
    fn decode(&mut self, buf: &mut B) -> T;
}

/// `StreamMemberEncoder<B, T>` — encode через member-метод.
pub trait StreamMemberEncoder<B, T> {
    fn encode_member(&mut self, buf: &mut B, value: &T);
}

/// `StreamCodec<B, T>` — пара encoder+decoder (encode с &self).
pub trait StreamCodec<B, T> {
    fn encode(&self, buf: &mut B, value: &T);
    fn decode(&self, buf: &mut B) -> T;
}

impl<B, T> StreamCodec<B, T> for std::sync::Arc<dyn StreamCodec<B, T>> {
    fn encode(&self, buf: &mut B, value: &T) {
        (**self).encode(buf, value);
    }
    fn decode(&self, buf: &mut B) -> T {
        (**self).decode(buf)
    }
}

impl<B, T> StreamCodec<B, T> for std::sync::Arc<dyn StreamCodec<B, T> + Send + Sync> {
    fn encode(&self, buf: &mut B, value: &T) {
        (**self).encode(buf, value);
    }
    fn decode(&self, buf: &mut B) -> T {
        (**self).decode(buf)
    }
}

// ---------------------------------------------------------- конструкторы

/// Кодек на замыканиях (RefCell — чтобы encode мог быть &self).
pub struct FnCodec<B, T> {
    enc: std::cell::RefCell<Box<dyn FnMut(&mut B, &T)>>,
    dec: std::cell::RefCell<Box<dyn FnMut(&mut B) -> T>>,
}

impl<B, T> FnCodec<B, T> {
    pub fn new(
        enc: impl FnMut(&mut B, &T) + 'static,
        dec: impl FnMut(&mut B) -> T + 'static,
    ) -> Self {
        Self {
            enc: std::cell::RefCell::new(Box::new(enc)),
            dec: std::cell::RefCell::new(Box::new(dec)),
        }
    }
}

impl<B, T> StreamCodec<B, T> for FnCodec<B, T> {
    fn encode(&self, buf: &mut B, value: &T) {
        (self.enc.borrow_mut())(buf, value);
    }

    fn decode(&self, buf: &mut B) -> T {
        (self.dec.borrow_mut())(buf)
    }
}

/// Send+Sync вариант `FnCodec` — для кодеков, хранящихся в реестрах
/// и разделяемых между потоками (data components, Holder-кодеки).
pub struct SyncFnCodec<B, T> {
    enc: std::sync::Mutex<Box<dyn FnMut(&mut B, &T) + Send>>,
    dec: std::sync::Mutex<Box<dyn FnMut(&mut B) -> T + Send>>,
}

impl<B, T> SyncFnCodec<B, T> {
    pub fn new(
        enc: impl FnMut(&mut B, &T) + Send + 'static,
        dec: impl FnMut(&mut B) -> T + Send + 'static,
    ) -> Self {
        Self {
            enc: std::sync::Mutex::new(Box::new(enc)),
            dec: std::sync::Mutex::new(Box::new(dec)),
        }
    }
}

impl<B, T> StreamCodec<B, T> for SyncFnCodec<B, T> {
    fn encode(&self, buf: &mut B, value: &T) {
        (self.enc.lock().unwrap())(buf, value);
    }
    fn decode(&self, buf: &mut B) -> T {
        (self.dec.lock().unwrap())(buf)
    }
}

/// Конструктор `SyncFnCodec`.
pub fn sync_fn<B, T>(
    enc: impl FnMut(&mut B, &T) + Send + 'static,
    dec: impl FnMut(&mut B) -> T + Send + 'static,
) -> SyncFnCodec<B, T> {
    SyncFnCodec::new(enc, dec)
}

/// `StreamCodec.unit(value)` — пишет/читает ничего, возвращает константу.
pub fn unit<T: Clone + 'static>(value: T) -> FnCodec<FriendlyByteBuf, T> {
    FnCodec::new(
        move |_buf: &mut FriendlyByteBuf, _v: &T| {},
        move |_buf: &mut FriendlyByteBuf| value.clone(),
    )
}

/// `StreamCodec.ofMember` — member encoder/decoder (packet.codec(...)).
/// В отличие от StreamCodec, методы требуют &mut self.
pub struct MemberCodec<B, T, E, D> {
    encoder: E,
    decoder: D,
    _b: std::marker::PhantomData<fn(&mut B) -> T>,
}

impl<B, T, E, D> MemberCodec<B, T, E, D>
where
    E: FnMut(&mut B, &T),
    D: FnMut(&mut B) -> T,
{
    pub fn new(encoder: E, decoder: D) -> Self {
        Self { encoder, decoder, _b: std::marker::PhantomData }
    }

    pub fn encode(&mut self, buf: &mut B, value: &T) {
        (self.encoder)(buf, value);
    }

    pub fn decode(&mut self, buf: &mut B) -> T {
        (self.decoder)(buf)
    }
}

/// `StreamCodec.map(toFunction, fromFunction)` — поверх базового кодека.
/// Хранит базовый кодек по значению (generic).
pub struct MapCodec<B, T, U, F, G, C> {
    to: F,
    from: G,
    base: C,
    _types: std::marker::PhantomData<fn(&mut B) -> (T, U)>,
}

impl<B, T, U, F, G, C> MapCodec<B, T, U, F, G, C>
where
    F: Fn(&T) -> U,
    G: Fn(&U) -> T,
    C: StreamCodec<B, T>,
{
    pub fn new(to: F, from: G, base: C) -> Self {
        Self { to, from, base, _types: std::marker::PhantomData }
    }
}

impl<B, T, U, F, G, C> StreamCodec<B, U> for MapCodec<B, T, U, F, G, C>
where
    T: Clone,
    F: Fn(&T) -> U,
    G: Fn(&U) -> T,
    C: StreamCodec<B, T>,
{
    fn encode(&self, buf: &mut B, value: &U) {
        let t = (self.from)(value);
        self.base.encode(buf, &t);
    }

    fn decode(&self, buf: &mut B) -> U {
        let t = self.base.decode(buf);
        (self.to)(&t)
    }
}

/// MemberCodec как StreamCodec — member-методы через RefCell
/// (аналог Java MemberCodec, реализующего StreamCodec).
pub struct MemberCodecStream<B, T, E, D> {
    inner: std::cell::RefCell<MemberCodec<B, T, E, D>>,
}

impl<B, T, E, D> MemberCodecStream<B, T, E, D>
where
    E: FnMut(&mut B, &T),
    D: FnMut(&mut B) -> T,
{
    pub fn new(encoder: E, decoder: D) -> Self {
        Self { inner: std::cell::RefCell::new(MemberCodec::new(encoder, decoder)) }
    }
}

impl<B, T, E, D> StreamCodec<B, T> for MemberCodecStream<B, T, E, D>
where
    E: FnMut(&mut B, &T),
    D: FnMut(&mut B) -> T,
{
    fn encode(&self, buf: &mut B, value: &T) {
        self.inner.borrow_mut().encode(buf, value);
    }
    fn decode(&self, buf: &mut B) -> T {
        self.inner.borrow_mut().decode(buf)
    }
}

/// Удобный конструктор map.
pub fn map<B, T, U, F, G, C>(to: F, from: G, base: C) -> MapCodec<B, T, U, F, G, C>
where
    T: Clone,
    F: Fn(&T) -> U,
    G: Fn(&U) -> T,
    C: StreamCodec<B, T>,
{
    MapCodec::new(to, from, base)
}

/// `StreamCodec.composite` — несколько кодеков + конструктор.
/// Полная версия с varargs в Rust неудобна — реализуем для 2-4 элементов
/// через макросы по мере необходимости.

// ---------------------------------------------------------- готовые кодеки

pub use crate::network::codec::byte_buf_codecs::*;

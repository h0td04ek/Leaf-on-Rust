//! `net.minecraft.commands`.

pub mod arguments {
    pub use crate::command::arguments::*;
}

pub mod gamerule;
pub mod world;

use crate::network::chat::component::Component;

pub trait CommandSource {
    fn has_permission(&self, level: u8) -> bool;
    fn send_success(&mut self, message: Component, broadcast_to_admins: bool);
}

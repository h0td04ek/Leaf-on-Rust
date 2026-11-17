//! Перенос `net.minecraft.server.level.ClientInformation`.

use crate::entity::humanoid_arm::HumanoidArm;
use crate::entity::player::chat_visibility::ChatVisibility;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::server::level::particle_status::ParticleStatus;

/// `ClientInformation` — настройки клиента (serverbound client_information).
#[derive(Clone, Debug, PartialEq)]
pub struct ClientInformation {
    /// `language` — локаль (`en_us`), max 16 символов.
    pub language: String,
    /// `viewDistance` — дистанция прорисовки.
    pub view_distance: i8,
    /// `chatVisibility`.
    pub chat_visibility: ChatVisibility,
    /// `chatColors`.
    pub chat_colors: bool,
    /// `modelCustomisation` — битовая маска (пол/руки/плащ).
    pub model_customisation: i32,
    /// `mainHand`.
    pub main_hand: HumanoidArm,
    /// `textFilteringEnabled`.
    pub text_filtering_enabled: bool,
    /// `allowsListing`.
    pub allows_listing: bool,
    /// `particleStatus`.
    pub particle_status: ParticleStatus,
}

/// `MAX_LANGUAGE_LENGTH`.
pub const MAX_LANGUAGE_LENGTH: usize = 16;

impl ClientInformation {
    /// Десериализация из буфера (`new ClientInformation(buffer)`).
    pub fn read(buf: &mut FriendlyByteBuf) -> ClientInformation {
        let language = buf
            .read_utf_n(MAX_LANGUAGE_LENGTH)
            .expect("invalid language");
        let view_distance = buf.read_byte();
        let chat_visibility = ChatVisibility::by_id_wrapped(buf.read_byte() as i32);
        let chat_colors = buf.read_bool();
        let model_customisation = buf.read_unsigned_byte();
        let main_hand = HumanoidArm::by_id(buf.read_byte() as i32);
        let text_filtering_enabled = buf.read_bool();
        let allows_listing = buf.read_bool();
        let particle_status = ParticleStatus::by_id_wrapped(buf.read_byte() as i32);
        ClientInformation {
            language,
            view_distance,
            chat_visibility,
            chat_colors,
            model_customisation,
            main_hand,
            text_filtering_enabled,
            allows_listing,
            particle_status,
        }
    }

    /// `write(buffer)`.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf_n(&self.language, MAX_LANGUAGE_LENGTH)
            .expect("language too long");
        buf.write_byte(self.view_distance as i32);
        buf.write_byte(self.chat_visibility.id());
        buf.write_bool(self.chat_colors);
        buf.write_byte(self.model_customisation);
        buf.write_byte(match self.main_hand {
            HumanoidArm::Left => 0,
            HumanoidArm::Right => 1,
        });
        buf.write_bool(self.text_filtering_enabled);
        buf.write_bool(self.allows_listing);
        buf.write_byte(self.particle_status.id());
    }

    /// `createDefault()`.
    pub fn create_default() -> ClientInformation {
        ClientInformation {
            language: "en_us".to_string(),
            view_distance: 2,
            chat_visibility: ChatVisibility::Full,
            chat_colors: true,
            model_customisation: 0,
            main_hand: HumanoidArm::Right, // Player.DEFAULT_MAIN_HAND
            text_filtering_enabled: false,
            allows_listing: false,
            particle_status: ParticleStatus::All,
        }
    }
}

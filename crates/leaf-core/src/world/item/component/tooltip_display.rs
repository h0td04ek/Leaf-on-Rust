//! Перенос `net.minecraft.world.item.component.TooltipDisplay`.

use crate::core::component::data_component_type::{
    data_component_type_stream_codec, DataComponentType,
};
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use std::sync::Arc;

/// `TooltipDisplay` — какие компоненты прятать в tooltip.
#[derive(Clone, Debug)]
pub struct TooltipDisplay {
    /// `hideTooltip`.
    pub hide_tooltip: bool,
    /// `hiddenComponents` — `SequencedSet<DataComponentType<?>>`
    /// (порядок вставки, без дублей).
    pub hidden_components: Vec<Arc<DataComponentType>>,
}

impl TooltipDisplay {
    /// `DEFAULT`.
    pub fn default() -> Self {
        Self { hide_tooltip: false, hidden_components: Vec::new() }
    }

    /// `shows(component)`.
    pub fn shows(&self, component: &DataComponentType) -> bool {
        !self.hidden_components.iter().any(|c| c.id == component.id)
    }

    /// `withHidden(component, isHidden)` — Java-версия возвращает
    /// новый TooltipDisplay.
    pub fn with_hidden(&self, component: Arc<DataComponentType>, is_hidden: bool) -> Self {
        let mut hidden = self.hidden_components.clone();
        if is_hidden {
            if !hidden.iter().any(|c| c.id == component.id) {
                hidden.push(component);
            }
        } else {
            hidden.retain(|c| c.id != component.id);
        }
        Self { hide_tooltip: self.hide_tooltip, hidden_components: hidden }
    }

    /// `STREAM_CODEC` — BOOL + collection(DataComponentType).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, TooltipDisplay> + Send + Sync + 'static {
        let type_codec = data_component_type_stream_codec();
        let set_codec = std::sync::Arc::new(byte_buf_codecs::collection(Vec::with_capacity, type_codec));
                let set_codec2 = set_codec.clone();
sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &TooltipDisplay| {
                buf.write_bool(v.hide_tooltip);
                set_codec.encode(buf, &v.hidden_components);
            },
            move |buf: &mut FriendlyByteBuf| {
                let hide_tooltip = buf.read_bool();
                let mut hidden_components: Vec<Arc<DataComponentType>> = set_codec2.decode(buf);
                hidden_components.dedup_by(|a, b| a.id == b.id);
                TooltipDisplay { hide_tooltip, hidden_components }
            },
        )
    }
}

impl PartialEq for TooltipDisplay {
    fn eq(&self, other: &Self) -> bool {
        self.hide_tooltip == other.hide_tooltip
            && self.hidden_components.len() == other.hidden_components.len()
            && self
                .hidden_components
                .iter()
                .zip(other.hidden_components.iter())
                .all(|(a, b)| a.id == b.id)
    }
}

//! General purpose collection data structures

pub mod slot_map;
pub use slot_map::SlotMap;

pub mod map;
pub use map::Map;

pub mod bimap;
pub use bimap::BiMap;

pub mod named_slot_map;
pub use named_slot_map::NamedSlotMap;
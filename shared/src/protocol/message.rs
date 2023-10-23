use std::any::TypeId;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::protocol::registry::{TypeKind, TypeMapper};
use crate::serialize::writer::WriteBuffer;
use crate::{BitSerializable, Message};

// client writes an Enum containing all their message type
// each message must derive message

// that big enum will implement MessageProtocol via a proc macro
pub trait MessageProtocol: BitSerializable + Serialize + DeserializeOwned + Clone {}

/// MessageKind - internal wrapper around the type of the message
#[derive(Debug, Eq, Hash, Copy, Clone, PartialEq)]
pub struct MessageKind(TypeId);

impl MessageKind {
    pub fn of<M: Message>() -> Self {
        Self(TypeId::of::<M>())
    }
}

impl TypeKind for MessageKind {}

impl From<TypeId> for MessageKind {
    fn from(type_id: TypeId) -> Self {
        Self(type_id)
    }
}

#[derive(Default, Clone)]
pub struct MessageRegistry {
    // pub(in crate::protocol) builder_map: HashMap<MessageKind, MessageMetadata>,
    pub(in crate::protocol) kind_map: TypeMapper<MessageKind>,
    built: bool,
}
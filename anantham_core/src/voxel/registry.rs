use crate::math::Vec4;
use bevy_ecs::prelude::Resource;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default, Debug)]
pub struct BlockAttributes {
    pub is_transparent: bool,
    pub color: Vec4,
}

#[derive(Resource, Default)]
pub struct BlockRegistry {
    pub name_to_id: HashMap<String, u16>,
    pub attributes: Arc<Vec<BlockAttributes>>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        let mut reg = Self::default();
        Arc::make_mut(&mut reg.attributes).push(BlockAttributes {
            is_transparent: true,
            color: Vec4::ZERO,
        });
        reg.name_to_id.insert("air".to_string(), 0);
        reg
    }

    pub fn register(&mut self, namespace: &str, attrs: BlockAttributes) -> u16 {
        if let Some(&existing_id) = self.name_to_id.get(namespace) {
            tracing::warn!("Block '{}' is already registered!", namespace);
            return existing_id;
        }

        let id = self.attributes.len() as u16;
        self.name_to_id.insert(namespace.to_string(), id);

        Arc::make_mut(&mut self.attributes).push(attrs);

        tracing::debug!("Registered Block: {} -> ID {}", namespace, id);
        id
    }

    pub fn get(&self, id: u16) -> &BlockAttributes {
        self.attributes
            .get(id as usize)
            .unwrap_or(&self.attributes[0])
    }

    pub fn get_id(&self, namespace: &str) -> Option<u16> {
        self.name_to_id.get(namespace).copied()
    }
}

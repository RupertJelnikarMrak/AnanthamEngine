use bevy_ecs::prelude::Resource;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct BlockAttributes {
    pub is_transparent: bool,
    pub color: glam::Vec4,
}

#[derive(Resource, Default)]
pub struct BlockRegistry {
    name_to_id: HashMap<String, u16>,
    attributes: Vec<BlockAttributes>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, namespace: &str, attrs: BlockAttributes) -> u16 {
        if let Some(&existing_id) = self.name_to_id.get(namespace) {
            tracing::warn!("Block '{}' is already registered!", namespace);
            return existing_id;
        }

        let id = self.attributes.len() as u16;
        self.name_to_id.insert(namespace.to_string(), id);
        self.attributes.push(attrs);

        tracing::debug!("Registered Block: {} -> ID {}", namespace, id);
        id
    }

    pub fn get(&self, id: u16) -> &BlockAttributes {
        // Fall back to id: 0 Air
        self.attributes
            .get(id as usize)
            .unwrap_or(&self.attributes[0])
    }

    pub fn get_id(&self, namespace: &str) -> Option<u16> {
        self.name_to_id.get(namespace).copied()
    }
}

use std::collections::HashMap;

use glam::Vec2;

use super::{Measurements, UiNode};

pub struct MeasurementsCache {
    cache: HashMap<MeasurementsKey, Measurements>,
}

impl MeasurementsCache {
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    pub fn get(&self, ui_node: &UiNode, boundary_size: Vec2) -> Option<&Measurements> {
        self.cache.get(&MeasurementsKey::new(ui_node, boundary_size))
    }

    pub fn insert(&mut self, ui_node: &UiNode, boundary_size: Vec2, measurements: Measurements) {
        self.cache
            .insert(MeasurementsKey::new(ui_node, boundary_size), measurements);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MeasurementsKey {
    ui_node_address: usize,
    boundary_size_x_bits: u32,
    boundary_size_y_bits: u32,
}

impl MeasurementsKey {
    fn new(ui_node: &UiNode, boundary_size: Vec2) -> Self {
        Self {
            ui_node_address: ui_node as *const _ as usize,
            boundary_size_x_bits: boundary_size.x.to_bits(),
            boundary_size_y_bits: boundary_size.y.to_bits(),
        }
    }
}

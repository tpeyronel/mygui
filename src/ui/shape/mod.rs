pub mod rounded_rectangle_shape;

use std::{any::Any, fmt::Debug};

use dyn_clone::DynClone;
use dyn_hash::DynHash;
use dyn_partial_eq::dyn_partial_eq;

use crate::mesh::mesh::Mesh;

use super::{Layout, Modifiers};

#[dyn_partial_eq]
pub trait Shape: Any + Debug + DynClone + DynHash {
    fn to_shape_data(&self, layout: &Layout, modifiers: &Modifiers) -> ShapeData;
}

dyn_clone::clone_trait_object!(Shape);
dyn_hash::hash_trait_object!(Shape);

pub struct ShapeData {
    pub foreground_mesh: Mesh,
    pub background_mesh: Mesh,
}

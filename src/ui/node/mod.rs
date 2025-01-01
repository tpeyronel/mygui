use std::{any::Any, fmt::Debug};

use dyn_partial_eq::DynPartialEq;
use dyn_partial_eq_derive::*;
use glam::Vec2;

use super::{processor::UiNodeProcessor, Layout, Modifiers};

pub mod block;
pub mod column;
pub mod row;
pub mod text;

#[derive(Debug, PartialEq)]
pub struct UiNode {
    pub props: Box<dyn UiElement>,
    pub modifiers: Modifiers,
    pub children: Vec<UiNode>,
}

impl UiNode {
    pub fn new<P: UiElement>(props: P, modifiers: Modifiers, children: Vec<UiNode>) -> Self {
        Self {
            props: Box::new(props),
            modifiers,
            children,
        }
    }
}

#[dyn_partial_eq]
pub trait UiElement: UiNodeProps + Any + Debug {}

impl<T: UiNodeProps + Any + Debug + DynPartialEq> UiElement for T {}

pub trait UiNodeProps {
    fn measure_fit_content(
        &self,
        modifiers: &Modifiers,
        children: &[UiNode],
        processor: &mut UiNodeProcessor<'_>,
        content_width: Option<f32>,
        content_height: Option<f32>,
    ) -> (Vec2, Vec2);

    fn compute_children_layouts(
        &self,
        modifiers: &Modifiers,
        children: &[UiNode],
        processor: &mut UiNodeProcessor<'_>,
        layout: &Layout,
    ) -> Vec<Layout>;

    fn emit_draw_data(&self, processor: &mut UiNodeProcessor<'_>, layout: &Layout);
}

mod border_radius;
mod border_thickness;
pub mod draw_element;
mod margin;
mod padding;

use border_radius::BorderRadius;
use border_thickness::BorderThickness;
use draw_element::DrawElement;
use glam::Vec2;
use margin::Margin;
use padding::Padding;

use crate::{
    font::font_engine::{FontEngine, TextLayoutOptions},
    image::image_manager::ImageManager,
    is_integer::IsInteger,
    rectangle::Rectangle,
    vertex::Color,
};

#[derive(Default, Debug, Clone)]
pub struct Modifiers {
    width: Extent,
    height: Extent,
    margin: Margin,
    padding: Padding,
    fill_color: Color,
    border_color: Color,
    border_thickness: BorderThickness,
    border_radius: BorderRadius,
    self_alignment: Alignment,
    weight: f32,
}

impl Modifiers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(self, width: Extent) -> Self {
        Self { width, ..self }
    }

    pub fn height(self, height: Extent) -> Self {
        Self { height, ..self }
    }

    pub fn margin(self, margin: Margin) -> Self {
        Self { margin, ..self }
    }

    pub fn padding(self, padding: Padding) -> Self {
        Self { padding, ..self }
    }

    pub fn fill_color(self, fill_color: Color) -> Self {
        Self { fill_color, ..self }
    }

    pub fn border_color(self, border_color: Color) -> Self {
        Self { border_color, ..self }
    }

    pub fn border_thickness(self, border_thickness: BorderThickness) -> Self {
        Self {
            border_thickness,
            ..self
        }
    }

    pub fn border_radius(self, border_radius: BorderRadius) -> Self {
        Self { border_radius, ..self }
    }

    pub fn self_alignment(self, self_alignment: Alignment) -> Self {
        Self { self_alignment, ..self }
    }

    pub fn weight(self, weight: f32) -> Self {
        Self { weight, ..self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Extent {
    FillParent,
    FitContent,
    Px(f32),
}

impl Default for Extent {
    fn default() -> Self {
        Self::FillParent
    }
}

#[derive(Debug, Clone)]
pub struct BoxProps {
    modifiers: Modifiers,
    children: Vec<UiNode>,
}

impl Default for BoxProps {
    fn default() -> Self {
        Self {
            modifiers: Modifiers::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColumnProps {
    modifiers: Modifiers,
    children: Vec<UiNode>,
}

impl Default for ColumnProps {
    fn default() -> Self {
        Self {
            modifiers: Modifiers::new(),
            children: Vec::new(),
        }
    }
}

pub type RowProps = ColumnProps;

#[derive(Debug, Clone)]
pub struct TextProps {
    content: String,
    font: String,
    font_size: f32,
    line_height: f32,
    modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Center,
    Right,
    TopRight,
    Top,
    TopLeft,
    Left,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Center
    }
}

#[derive(Debug, Clone)]
pub enum UiNode {
    Box(BoxProps),
    Column(ColumnProps),
    Row(RowProps),
    Text(TextProps),
}

impl UiNode {
    pub fn to_draw_data(
        self,
        boundary_pos: Vec2,
        boundary_size: Vec2,
        image_manager: &mut ImageManager,
        font_engine: &mut FontEngine,
        out: &mut Vec<DrawElement>,
    ) {
        let mut processor = UiNodeProcessor::new(image_manager, font_engine, out);
        processor.to_draw_data(self, boundary_pos, boundary_size);
    }
}

struct UiNodeProcessor<'a> {
    image_manager: &'a mut ImageManager,
    font_engine: &'a mut FontEngine,
    draw_data: &'a mut Vec<DrawElement>,
}

impl<'a> UiNodeProcessor<'a> {
    fn new(
        image_manager: &'a mut ImageManager,
        font_engine: &'a mut FontEngine,
        draw_data: &'a mut Vec<DrawElement>,
    ) -> Self {
        Self {
            image_manager,
            font_engine,
            draw_data,
        }
    }

    fn to_draw_data(&mut self, ui_node: UiNode, boundary_pos: Vec2, boundary_size: Vec2) {
        let boundary_pos = boundary_pos.round();
        let boundary_size = boundary_size.round();

        let root_node = UiNode::Box(BoxProps {
            modifiers: Modifiers::new()
                .width(Extent::Px(boundary_size.x))
                .height(Extent::Px(boundary_size.y)),
            children: vec![ui_node],
        });

        let root_layout = Layout {
            margin_position: boundary_pos,
            margin_size: boundary_size,
            children_boundary_size: boundary_size,
            margin: Margin::all(0.0),
            border_thickness: BorderThickness::all(0.0),
            padding: Padding::all(0.0),
        };

        let root_layout_node = self.compute_layout_rec(&root_node, root_layout);
        self.to_draw_data_rec(&root_node, &root_layout_node);
        self.draw_data.remove(0); // TODO: remove. This is mainly done to simplify testing (the first rectangle is completely transparent).
    }

    fn to_draw_data_rec(&mut self, ui_node: &UiNode, layout_node: &UiNodeLayout) {
        let (modifiers, children) = match ui_node {
            UiNode::Box(props) => (&props.modifiers, Some(&props.children)),
            UiNode::Column(props) => (&props.modifiers, Some(&props.children)),
            UiNode::Row(props) => (&props.modifiers, Some(&props.children)),
            UiNode::Text(props) => (&props.modifiers, None),
        };

        let layout = &layout_node.layout;

        self.emit_rectangle(
            layout.border_position(),
            layout.border_size(),
            modifiers.fill_color,
            modifiers.border_color,
            modifiers.border_thickness,
            modifiers.border_radius,
        );

        if let UiNode::Text(props) = ui_node {
            self.emit_text_draw_data(props, layout);
        }

        if let Some(children) = children {
            for (i, c) in children.iter().enumerate() {
                self.to_draw_data_rec(c, &layout_node.children[i]);
            }
        }
    }

    fn compute_layout_rec(&mut self, ui_node: &UiNode, layout: Layout) -> UiNodeLayout {
        assert!(layout.margin_position.x.is_integer());
        assert!(layout.margin_position.y.is_integer());
        assert!(layout.margin_size.x.is_integer());
        assert!(layout.margin_size.y.is_integer());

        let children_layouts = self.compute_children_layouts(ui_node, &layout);

        UiNodeLayout {
            layout,
            children: children_layouts,
        }
    }

    fn compute_children_layouts(&mut self, ui_node: &UiNode, layout: &Layout) -> Vec<UiNodeLayout> {
        match ui_node {
            UiNode::Box(props) => self.compute_box_children_layouts(props, layout),
            UiNode::Column(props) => self.compute_column_children_layouts(props, layout),
            UiNode::Row(props) => self.compute_row_children_layouts(props, layout),
            UiNode::Text(_) => vec![],
        }
    }

    fn compute_box_children_layouts(
        &mut self,
        BoxProps { children, .. }: &BoxProps,
        layout: &Layout,
    ) -> Vec<UiNodeLayout> {
        children
            .iter()
            .map(|c| {
                let child_measurements = self.measure(c, layout.children_boundary_size());
                let child_margin_size = child_measurements.margin_size;
                let child_modifiers = match c {
                    UiNode::Box(props) => &props.modifiers,
                    UiNode::Column(props) => &props.modifiers,
                    UiNode::Row(props) => &props.modifiers,
                    UiNode::Text(props) => &props.modifiers,
                };
                let child_margin_position = match child_modifiers.self_alignment {
                    Alignment::Center => layout.content_center() - 0.5 * child_margin_size,
                    Alignment::Right => Vec2::new(
                        layout.content_position().x + layout.content_size().x - child_margin_size.x,
                        layout.content_center().y - 0.5 * child_margin_size.y,
                    ),
                    Alignment::TopRight => layout.content_position() + layout.content_size() - child_margin_size,
                    Alignment::Top => Vec2::new(
                        layout.content_center().x - 0.5 * child_margin_size.x,
                        layout.content_position().y + layout.content_size().y - child_margin_size.y,
                    ),
                    Alignment::TopLeft => Vec2::new(
                        layout.content_position().x,
                        layout.content_position().y + layout.content_size().y - child_margin_size.y,
                    ),
                    Alignment::Left => Vec2::new(
                        layout.content_position().x,
                        layout.content_center().y - 0.5 * child_margin_size.y,
                    ),
                    Alignment::BottomLeft => layout.content_position(),
                    Alignment::Bottom => Vec2::new(
                        layout.content_center().x - 0.5 * child_margin_size.x,
                        layout.content_position().y,
                    ),
                    Alignment::BottomRight => Vec2::new(
                        layout.content_position().x + layout.content_size().x - child_margin_size.x,
                        layout.content_position().y,
                    ),
                }
                .round();

                let child_layout = child_measurements.to_layout(child_margin_position);
                self.compute_layout_rec(c, child_layout)
            })
            .collect()
    }

    fn compute_column_children_layouts(
        &mut self,
        ColumnProps { children, .. }: &ColumnProps,
        layout: &Layout,
    ) -> Vec<UiNodeLayout> {
        let total_children_weight: f32 = children
            .iter()
            .map(|c| match c {
                UiNode::Box(props) => props.modifiers.weight,
                UiNode::Column(props) => props.modifiers.weight,
                UiNode::Row(props) => props.modifiers.weight,
                UiNode::Text(props) => props.modifiers.weight,
            })
            .sum();

        let total_children_height: f32 = children
            .iter()
            .map(|c| self.measure(c, layout.children_boundary_size()).margin_size.y)
            .sum();

        let extra_column_height = (layout.content_size().y - total_children_height).max(0.0);

        let mut remaining_column_height = extra_column_height;
        let mut remaining_children_weight = total_children_weight;

        let mut vertical_offset = 0.0;
        let column_top = layout.content_position().y + layout.content_size().y;
        children
            .iter()
            .map(|c| {
                let child_modifiers = match c {
                    UiNode::Box(props) => &props.modifiers,
                    UiNode::Column(props) => &props.modifiers,
                    UiNode::Row(props) => &props.modifiers,
                    UiNode::Text(props) => &props.modifiers,
                };

                let mut child_measurements = self.measure(c, layout.children_boundary_size());

                if remaining_children_weight > 0.0 {
                    let child_weight = child_modifiers.weight;
                    let child_extra_height = remaining_column_height * (child_weight / remaining_children_weight);
                    let child_extra_height = child_extra_height.ceil().min(remaining_column_height);
                    remaining_column_height -= child_extra_height;
                    remaining_children_weight -= child_weight;

                    child_measurements.margin_size.y += child_extra_height;
                    child_measurements.children_boundary_size = child_measurements.content_size();
                }

                let child_margin_size = child_measurements.margin_size;
                vertical_offset += child_margin_size.y;

                let child_margin_position = match child_modifiers.self_alignment {
                    Alignment::TopLeft | Alignment::Left | Alignment::BottomLeft => {
                        Vec2::new(layout.content_position().x, column_top - vertical_offset)
                    }
                    Alignment::Top | Alignment::Center | Alignment::Bottom => Vec2::new(
                        (layout.content_center().x - 0.5 * child_margin_size.x).round(),
                        column_top - vertical_offset,
                    ),
                    Alignment::TopRight | Alignment::Right | Alignment::BottomRight => Vec2::new(
                        layout.content_position().x + layout.content_size().x - child_margin_size.x,
                        column_top - vertical_offset,
                    ),
                };

                let child_layout = child_measurements.to_layout(child_margin_position);
                self.compute_layout_rec(c, child_layout)
            })
            .collect()
    }

    fn compute_row_children_layouts(
        &mut self,
        RowProps { children, .. }: &RowProps,
        layout: &Layout,
    ) -> Vec<UiNodeLayout> {
        let total_children_weight: f32 = children
            .iter()
            .map(|c| match c {
                UiNode::Box(props) => props.modifiers.weight,
                UiNode::Column(props) => props.modifiers.weight,
                UiNode::Row(props) => props.modifiers.weight,
                UiNode::Text(props) => props.modifiers.weight,
            })
            .sum();

        let total_children_width: f32 = children
            .iter()
            .map(|c| self.measure(c, layout.children_boundary_size()).margin_size.x)
            .sum();

        let extra_column_width = (layout.content_size().x - total_children_width).max(0.0);

        let mut remaining_column_width = extra_column_width;
        let mut remaining_children_weight = total_children_weight;

        let mut horizontal_offset = 0.0;
        children
            .iter()
            .map(|c| {
                let child_modifiers = match c {
                    UiNode::Box(props) => &props.modifiers,
                    UiNode::Column(props) => &props.modifiers,
                    UiNode::Row(props) => &props.modifiers,
                    UiNode::Text(props) => &props.modifiers,
                };

                let mut child_measurements = self.measure(c, layout.children_boundary_size());

                if remaining_children_weight > 0.0 {
                    let child_weight = child_modifiers.weight;
                    let child_extra_width = remaining_column_width * (child_weight / remaining_children_weight);
                    let child_extra_width = child_extra_width.ceil().min(remaining_column_width);
                    remaining_column_width -= child_extra_width;
                    remaining_children_weight -= child_weight;

                    child_measurements.margin_size.x += child_extra_width;
                    child_measurements.children_boundary_size = child_measurements.content_size();
                }

                let child_margin_size = child_measurements.margin_size;

                let child_margin_position = match child_modifiers.self_alignment {
                    Alignment::BottomLeft | Alignment::Bottom | Alignment::BottomRight => Vec2::new(
                        layout.content_position().x + horizontal_offset,
                        layout.content_position().y,
                    ),
                    Alignment::Left | Alignment::Center | Alignment::Right => Vec2::new(
                        layout.content_position().x + horizontal_offset,
                        (layout.content_center().y - 0.5 * child_margin_size.y).round(),
                    ),
                    Alignment::TopLeft | Alignment::Top | Alignment::TopRight => Vec2::new(
                        layout.content_position().x + horizontal_offset,
                        layout.content_position().y + layout.content_size().y - child_margin_size.y,
                    ),
                };

                horizontal_offset += child_margin_size.x;

                let child_layout = child_measurements.to_layout(child_margin_position);
                self.compute_layout_rec(c, child_layout)
            })
            .collect()
    }

    fn measure(&mut self, ui_node: &UiNode, boundary_size: Vec2) -> Measurements {
        match ui_node {
            UiNode::Text(props) => return self.measure_text(props, boundary_size),
            _ => (),
        }

        let modifiers = match ui_node {
            UiNode::Box(props) => &props.modifiers,
            UiNode::Column(props) => &props.modifiers,
            UiNode::Row(props) => &props.modifiers,
            UiNode::Text(props) => &props.modifiers,
        };

        let empty_vec = vec![];
        let children = match ui_node {
            UiNode::Box(props) => &props.children,
            UiNode::Column(props) => &props.children,
            UiNode::Row(props) => &props.children,
            UiNode::Text(_) => &empty_vec,
        };
        // We subtract it here as the only special case is Extent::FillParent.
        let boundary_size = (boundary_size - modifiers.margin.delta_size()).max(Vec2::ZERO);

        let preliminar_children_boundary_size = Vec2::new(
            match modifiers.width {
                Extent::FitContent => 0.0,
                Extent::FillParent => boundary_size.x,
                Extent::Px(px) => px.round(),
            },
            match modifiers.height {
                Extent::FitContent => 0.0,
                Extent::FillParent => boundary_size.y,
                Extent::Px(px) => px.round(),
            },
        );
        let preliminar_children_boundary_size = (preliminar_children_boundary_size
            - modifiers.border_thickness.delta_size()
            - modifiers.padding.delta_size())
        .max(Vec2::ZERO);
        // TODO: only compute when necessary
        let min_intrinsic_children_sizes: Vec<Measurements> =
            self.measure_children(preliminar_children_boundary_size, children);

        let mut children_boundary_size = preliminar_children_boundary_size;
        let computed_width = match modifiers.width {
            Extent::FillParent => boundary_size.x,
            Extent::FitContent => match ui_node {
                UiNode::Box(_) | UiNode::Column(_) => {
                    let max_child_width = min_intrinsic_children_sizes
                        .iter()
                        .map(|cs| cs.margin_size.x)
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(0.0);
                    children_boundary_size.x = max_child_width;
                    max_child_width + modifiers.padding.delta_size().x + modifiers.border_thickness.delta_size().x
                }
                UiNode::Row(_) => {
                    let min_intrinsic_width = min_intrinsic_children_sizes.iter().map(|cs| cs.margin_size.x).sum();
                    children_boundary_size.x = min_intrinsic_width;
                    let children_sizes = self.measure_children(children_boundary_size, children);
                    children_sizes.iter().map(|cs| cs.margin_size.x).sum::<f32>()
                        + modifiers.padding.delta_size().x
                        + modifiers.border_thickness.delta_size().x
                }
                UiNode::Text(_) => unreachable!(),
            },
            Extent::Px(px) => px.round(),
        };

        let computed_height = match modifiers.height {
            Extent::FillParent => boundary_size.y,
            Extent::FitContent => match ui_node {
                UiNode::Box(_) | UiNode::Row(_) => {
                    let max_child_height = min_intrinsic_children_sizes
                        .iter()
                        .map(|cs| cs.margin_size.y)
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(0.0);
                    children_boundary_size.y = max_child_height;
                    max_child_height + modifiers.padding.delta_size().y + modifiers.border_thickness.delta_size().y
                }
                UiNode::Column(_) => {
                    let min_intrinsic_height = min_intrinsic_children_sizes.iter().map(|cs| cs.margin_size.y).sum();
                    children_boundary_size.y = min_intrinsic_height;
                    let children_sizes = self.measure_children(children_boundary_size, children);
                    children_sizes.iter().map(|cs| cs.margin_size.y).sum::<f32>()
                        + modifiers.padding.delta_size().y
                        + modifiers.border_thickness.delta_size().y
                }
                UiNode::Text(_) => unreachable!(),
            },
            Extent::Px(px) => px.round(),
        };

        let border_size = Vec2::new(computed_width, computed_height);
        let margin_size = border_size + modifiers.margin.delta_size();

        Measurements {
            margin_size,
            margin: modifiers.margin,
            border_thickness: modifiers.border_thickness,
            padding: modifiers.padding,
            children_boundary_size,
        }
    }

    fn measure_children(&mut self, parent_size: Vec2, children: &[UiNode]) -> Vec<Measurements> {
        return children.iter().map(|c| self.measure(c, parent_size)).collect();
    }

    fn measure_text(
        &mut self,
        TextProps {
            content,
            font,
            font_size,
            line_height,
            modifiers,
        }: &TextProps,
        boundary_size: Vec2,
    ) -> Measurements {
        // TODO: support more than FitContent
        let boundary_size = boundary_size - modifiers.border_thickness.delta_size() - modifiers.padding.delta_size();

        let text_options = TextLayoutOptions {
            font,
            font_size: *font_size,
            line_height: *line_height,
            max_line_width: if boundary_size.y == 0.0 {
                f32::INFINITY
            } else {
                boundary_size.x
            },
        };

        let dimensions = self
            .font_engine
            .lay_out_text(self.image_manager, content, &text_options, |_| {});

        let content_size = dimensions;
        let padding_size = content_size + modifiers.padding.delta_size();
        let border_size = padding_size + modifiers.border_thickness.delta_size();
        let margin_size = border_size + modifiers.margin.delta_size();

        Measurements {
            margin_size,
            margin: modifiers.margin,
            border_thickness: modifiers.border_thickness,
            padding: modifiers.padding,
            children_boundary_size: content_size,
        }
    }

    fn emit_rectangle(
        &mut self,
        position: Vec2,
        size: Vec2,
        fill_color: Color,
        border_color: Color,
        border_thickness: BorderThickness,
        border_radius: BorderRadius,
    ) {
        let rectangle = DrawElement::Rectangle {
            bounds: Rectangle::from_position_size(position, size),
            fill_color,
            border_color,
            border_radius: border_radius.to_vec4(),
            border_width: border_thickness.to_vec4(),
        };

        self.draw_data.push(rectangle);
    }

    fn emit_text_draw_data(
        &mut self,
        TextProps {
            content,
            font,
            font_size,
            line_height,
            modifiers,
        }: &TextProps,
        layout: &Layout,
    ) {
        let origin = layout.content_position() + Vec2::new(0.0, layout.content_size().y);
        let options = TextLayoutOptions {
            font,
            font_size: *font_size,
            line_height: *line_height,
            max_line_width: if layout.content_size().y == 0.0 {
                f32::INFINITY
            } else {
                layout.content_size().x
            },
        };

        self.font_engine
            .lay_out_text(self.image_manager, content, &options, |glyph| {
                let texture = DrawElement::Texture {
                    bounds: Rectangle::from_position_size(origin + glyph.position, glyph.size),
                    uv_rectangle: glyph.atlas_uv_rectangle,
                    image_id: glyph.image_id,
                };

                self.draw_data.push(texture);
            });
    }
}

struct UiNodeLayout {
    layout: Layout,
    children: Vec<UiNodeLayout>,
}

struct Layout {
    margin_position: Vec2,
    margin_size: Vec2,
    children_boundary_size: Vec2,
    margin: Margin,
    border_thickness: BorderThickness,
    padding: Padding,
}

impl Layout {
    fn margin_position(&self) -> Vec2 {
        self.margin_position
    }

    fn border_position(&self) -> Vec2 {
        self.margin_position + self.margin.delta_position()
    }

    fn padding_position(&self) -> Vec2 {
        self.margin_position + self.margin.delta_position() + self.border_thickness.delta_position()
    }

    fn content_position(&self) -> Vec2 {
        self.margin_position
            + self.margin.delta_position()
            + self.border_thickness.delta_position()
            + self.padding.delta_position() // TODO: clamp
    }

    fn margin_size(&self) -> Vec2 {
        self.margin_size
    }

    fn border_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size()).max(Vec2::ZERO)
    }

    fn padding_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size()).max(Vec2::ZERO)
    }

    fn content_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size() - self.padding.delta_size())
            .max(Vec2::ZERO)
    }

    fn children_boundary_size(&self) -> Vec2 {
        self.children_boundary_size
    }

    fn content_center(&self) -> Vec2 {
        self.content_position() + (self.content_size() * 0.5)
    }
}

#[derive(Debug)]
struct Measurements {
    margin_size: Vec2,
    margin: Margin,
    border_thickness: BorderThickness,
    padding: Padding,
    children_boundary_size: Vec2,
}

impl Measurements {
    fn to_layout(&self, margin_position: Vec2) -> Layout {
        Layout {
            margin_position,
            margin_size: self.margin_size,
            children_boundary_size: self.children_boundary_size,
            margin: self.margin,
            border_thickness: self.border_thickness,
            padding: self.padding,
        }
    }

    fn border_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size()).max(Vec2::ZERO)
    }

    fn padding_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size()).max(Vec2::ZERO)
    }

    fn content_size(&self) -> Vec2 {
        (self.margin_size - self.margin.delta_size() - self.border_thickness.delta_size() - self.padding.delta_size())
            .max(Vec2::ZERO)
    }
}

pub fn example_ui() -> UiNode {
    return UiNode::Box(BoxProps {
        modifiers: Modifiers::new()
            .width(Extent::FillParent)
            .height(Extent::FillParent)
            .fill_color(Color::new(0.1, 0.1, 1.0, 1.0))
            .margin(Margin::all(8.0))
            .padding(Padding::all(16.0))
            .fill_color(Color::new(1.0, 1.0, 0.1, 0.25))
            .border_radius(BorderRadius::all(16.0))
            .padding(Padding::all(16.0)),
        children: vec![UiNode::Box(BoxProps {
            modifiers: Modifiers::new()
                .width(Extent::FillParent)
                .height(Extent::FillParent)
                .padding(Padding::all(0.0))
                .fill_color(Color::new(1.0, 0.1, 0.1, 0.25))
                .border_color(Color::new(1.0, 0.1, 0.1, 0.9))
                .border_thickness(BorderThickness::all(4.0))
                .border_radius(BorderRadius::all(8.0)),
            children: vec![
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Center)
                        .fill_color(Color::new(1.0, 1.0, 1.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .margin(Margin::all(4.0))
                        .self_alignment(Alignment::Right)
                        .fill_color(Color::new(1.0, 0.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::TopRight)
                        .fill_color(Color::new(1.0, 1.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::new(0.0, 8.0, 16.0, 24.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Top)
                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::new(4.0, 8.0, 12.0, 16.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::TopLeft)
                        .border_color(Color::new(1.0, 1.0, 1.0, 0.4))
                        .border_thickness(BorderThickness::all(2.0)),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .padding(Padding::all(8.0))
                                .border_color(Color::new(1.0, 0.0, 0.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0)),
                            children: vec![UiNode::Box(BoxProps {
                                modifiers: Modifiers::new().fill_color(Color::new(0.0, 1.0, 0.0, 0.4)),
                                children: vec![],
                            })],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .border_color(Color::new(0.0, 1.0, 0.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0))
                                .self_alignment(Alignment::BottomLeft),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .border_color(Color::new(0.0, 0.0, 1.0, 0.4))
                                .border_thickness(BorderThickness::all(2.0))
                                .self_alignment(Alignment::TopRight),
                            children: vec![],
                        }),
                    ],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Left)
                        .fill_color(Color::new(0.0, 0.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::BottomLeft)
                        .fill_color(Color::new(0.0, 0.0, 1.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::Bottom)
                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(80.0))
                        .height(Extent::Px(80.0))
                        .self_alignment(Alignment::BottomRight)
                        .fill_color(Color::new(1.0, 0.0, 1.0, 0.25))
                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                        .border_thickness(BorderThickness::all(1.0))
                        .border_radius(BorderRadius::all(4.0)),
                    children: vec![],
                }),
                UiNode::Row(RowProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .padding(Padding::all(8.0))
                        .border_thickness(BorderThickness::all(4.0))
                        .border_color(Color::new(1.0, 1.0, 1.0, 1.0)),
                    children: vec![
                        UiNode::Column(ColumnProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(256.0))
                                .height(Extent::FitContent)
                                .padding(Padding::all(16.0))
                                .border_thickness(BorderThickness::all(4.0))
                                .border_color(Color::new(1.0, 1.0, 1.0, 1.0)),
                            children: vec![
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .height(Extent::Px(24.0))
                                        .fill_color(Color::new(0.0, 1.0, 1.0, 0.5))
                                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(1.0))
                                        .border_radius(BorderRadius::all(8.0)),
                                    children: vec![],
                                }),
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .height(Extent::FillParent)
                                        .fill_color(Color::new(1.0, 0.0, 1.0, 0.5))
                                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(1.0))
                                        .border_radius(BorderRadius::all(8.0)),
                                    children: vec![],
                                }),
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .height(Extent::Px(32.0))
                                        .fill_color(Color::new(1.0, 0.0, 0.0, 0.5))
                                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(1.0))
                                        .border_radius(BorderRadius::all(8.0)),
                                    children: vec![],
                                }),
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .width(Extent::Px(96.0))
                                        .height(Extent::Px(64.0))
                                        .margin(Margin::all(8.0))
                                        .padding(Padding::all(8.0))
                                        .self_alignment(Alignment::Center)
                                        .fill_color(Color::new(1.0, 1.0, 0.0, 0.5))
                                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(4.0))
                                        .border_radius(BorderRadius::all(8.0)),
                                    children: vec![UiNode::Box(BoxProps {
                                        modifiers: Modifiers::new()
                                            .width(Extent::FillParent)
                                            .height(Extent::FillParent)
                                            .fill_color(Color::new(1.0, 1.0, 1.0, 0.5))
                                            .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                            .border_thickness(BorderThickness::all(1.0))
                                            .border_radius(BorderRadius::all(8.0)),
                                        children: vec![],
                                    })],
                                }),
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .height(Extent::Px(32.0))
                                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.5))
                                        .border_color(Color::new(0.1, 0.1, 0.1, 0.9))
                                        .border_thickness(BorderThickness::all(1.0))
                                        .border_radius(BorderRadius::all(8.0)),
                                    children: vec![],
                                }),
                            ],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.25, 0.25, 1.0, 0.5)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .fill_color(Color::new(0.25, 0.25, 1.0, 0.25)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.25, 1.0, 0.25, 0.5)),
                            children: vec![UiNode::Text(TextProps {
                                content: "ÓThis is a text!\nÓWith three lines\nÓThis is the last lineeeeeeeeee."
                                    .to_string(),
                                font: "jetbrainsmono-regular.ttf".to_string(),
                                font_size: 24.0,
                                line_height: 24.0 * 1.5,
                                modifiers: Modifiers::new().self_alignment(Alignment::TopLeft),
                            })],
                        }),
                    ],
                }),
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(256.0))
                        .self_alignment(Alignment::Left),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.4)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(2.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.4)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 0.0, 1.0, 0.4)),
                            children: vec![],
                        }),
                    ],
                }),
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new(),
                    children: vec![
                        UiNode::Row(RowProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(128.0))
                                .self_alignment(Alignment::Top),
                            children: vec![
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(1.0)
                                        .fill_color(Color::new(1.0, 0.0, 0.0, 0.4)),
                                    children: vec![],
                                }),
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(2.0)
                                        .fill_color(Color::new(0.0, 1.0, 0.0, 0.4)),
                                    children: vec![],
                                }),
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(1.0)
                                        .fill_color(Color::new(0.0, 0.0, 1.0, 0.4)),
                                    children: vec![UiNode::Text(TextProps {
                                        content: "HellÓowjdoqi129312893u!\nYegh".to_string(),
                                        font: "jetbrainsmono-regular.ttf".to_string(),
                                        font_size: 48.0,
                                        line_height: 48.0,
                                        modifiers: Modifiers::new().fill_color(Color::new(0.0, 0.0, 0.0, 1.0)),
                                    })],
                                }),
                            ],
                        }),
                        UiNode::Row(RowProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(128.0))
                                .self_alignment(Alignment::Top),
                            children: vec![
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(2.0)
                                        .fill_color(Color::new(1.0, 1.0, 0.0, 0.4)),
                                    children: vec![],
                                }),
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(1.0)
                                        .fill_color(Color::new(0.0, 1.0, 1.0, 0.4)),
                                    children: vec![],
                                }),
                                UiNode::Box(BoxProps {
                                    modifiers: Modifiers::new()
                                        .width(Extent::Px(0.0))
                                        .weight(3.0)
                                        .fill_color(Color::new(1.0, 0.0, 1.0, 0.4)),
                                    children: vec![UiNode::Text(TextProps {
                                        content: "HellÓowjdoqi129312893u!\nYegh".to_string(),
                                        font: "times.ttf".to_string(),
                                        font_size: 17.0,
                                        line_height: 17.0,
                                        modifiers: Modifiers::new()
                                            .fill_color(Color::new(0.0, 0.0, 0.0, 0.5))
                                            .border_radius(BorderRadius::all(8.0))
                                            .padding(Padding::all(8.0)),
                                    })],
                                }),
                            ],
                        }),
                    ],
                }),
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomRight),
                    children: (5..48)
                        .map(|i| {
                            UiNode::Text(TextProps {
                                content: "aAbBcCdDoOÓgfjpq".to_string(),
                                font: "tangerine-regular.ttf".to_string(),
                                font_size: i as f32,
                                line_height: i as f32,
                                modifiers: Modifiers::new()
                                    .self_alignment(Alignment::Left)
                                    .fill_color(if i % 2 == 0 {
                                        Color::new(1.0, 0.0, 0.0, 0.5)
                                    } else {
                                        Color::new(0.0, 1.0, 0.0, 0.5)
                                    }),
                            })
                        })
                        .collect(),
                }),
            ],
        })],
        ..Default::default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;
    use pretty_assertions::assert_eq;

    fn convert_to_draw_data(position: Vec2, size: Vec2, ui: UiNode) -> Vec<DrawElement> {
        let mut image_manager = ImageManager::new();
        let mut font_engine = FontEngine::new("");
        let mut draw_data = vec![];
        ui.to_draw_data(position, size, &mut image_manager, &mut font_engine, &mut draw_data);
        draw_data
    }

    fn test_converter(width: f32, height: f32, ui: UiNode, expected: &[DrawElement]) {
        let draw_data = convert_to_draw_data(Vec2::ZERO, Vec2::new(width, height), ui);
        assert_eq!(expected, &draw_data);
    }

    #[test]
    fn default_box() {
        test_converter(
            32.0,
            32.0,
            UiNode::Box(BoxProps {
                modifiers: Modifiers::new(),
                children: vec![],
            }),
            &[DrawElement::Rectangle {
                bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::ZERO,
                border_width: Vec4::ZERO,
            }],
        )
    }

    #[test]
    fn padding() {
        test_converter(
            32.0,
            32.0,
            UiNode::Box(BoxProps {
                modifiers: Modifiers::new().padding(Padding::all(8.0)),
                children: vec![UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                    children: vec![],
                })],
            }),
            &[
                DrawElement::Rectangle {
                    bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                    fill_color: Color::ZERO,
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
                DrawElement::Rectangle {
                    bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(16.0, 16.0)),
                    fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
            ],
        )
    }

    #[test]
    fn margin() {
        test_converter(
            32.0,
            32.0,
            UiNode::Box(BoxProps {
                modifiers: Modifiers::new().margin(Margin::all(8.0)),
                children: vec![],
            }),
            &[DrawElement::Rectangle {
                bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(16.0, 16.0)),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::ZERO,
                border_width: Vec4::ZERO,
            }],
        )
    }

    #[test]
    fn full_padding() {
        test_converter(
            32.0,
            32.0,
            UiNode::Box(BoxProps {
                modifiers: Modifiers::new().padding(Padding::all(16.0)),
                children: vec![UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                    children: vec![],
                })],
            }),
            &[
                DrawElement::Rectangle {
                    bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                    fill_color: Color::ZERO,
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
                DrawElement::Rectangle {
                    bounds: Rectangle::from_position_size(Vec2::new(16.0, 16.0), Vec2::new(0.0, 0.0)),
                    fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
            ],
        )
    }

    #[test]
    fn too_much_padding() {
        // TODO: restore this test.
        // test_converter(
        //     32.0,
        //     32.0,
        //     UiNode::Box(BoxProps {
        //         modifiers: Modifiers::new().padding(Padding::all(24.0)),
        //         children: vec![UiNode::Box(BoxProps {
        //             modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
        //             children: vec![],
        //         })],
        //     }),
        //     &[
        //         DrawElement::Rectangle {
        //             bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
        //             fill_color: Color::ZERO,
        //             border_color: Color::ZERO,
        //             border_radius: Vec4::ZERO,
        //             border_width: Vec4::ZERO,
        //         },
        //         DrawElement::Rectangle {
        //             bounds: Rectangle::from_position_size(Vec2::new(16.0, 16.0), Vec2::new(0.0, 0.0)),
        //             fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
        //             border_color: Color::ZERO,
        //             border_radius: Vec4::ZERO,
        //             border_width: Vec4::ZERO,
        //         },
        //     ],
        // )
    }

    #[test]
    fn self_alignment_basic() {
        fn test_self_alignment_basic(alignment: Alignment, expected_position: Vec2) {
            test_converter(
                32.0,
                32.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(8.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(alignment),
                    children: vec![],
                }),
                &[DrawElement::Rectangle {
                    bounds: Rectangle::from_position_size(expected_position, Vec2::new(8.0, 8.0)),
                    fill_color: Color::ZERO,
                    border_color: Color::ZERO,
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                }],
            )
        }

        test_self_alignment_basic(Alignment::Center, Vec2::new(12.0, 12.0));
        test_self_alignment_basic(Alignment::Right, Vec2::new(24.0, 12.0));
        test_self_alignment_basic(Alignment::TopRight, Vec2::new(24.0, 24.0));
        test_self_alignment_basic(Alignment::Top, Vec2::new(12.0, 24.0));
        test_self_alignment_basic(Alignment::TopLeft, Vec2::new(0.0, 24.0));
        test_self_alignment_basic(Alignment::Left, Vec2::new(0.0, 12.0));
        test_self_alignment_basic(Alignment::BottomLeft, Vec2::new(0.0, 0.0));
        test_self_alignment_basic(Alignment::Bottom, Vec2::new(12.0, 0.0));
        test_self_alignment_basic(Alignment::BottomRight, Vec2::new(24.0, 0.0));
    }

    #[test]
    fn subpixel_alignment() {
        let ui: UiNode = UiNode::Box(BoxProps {
            modifiers: Modifiers::new().width(Extent::Px(8.0)).height(Extent::Px(8.0)),
            children: vec![],
        });

        test_converter(
            15.0,
            15.0,
            ui.clone(),
            &[DrawElement::Rectangle {
                bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(8.0, 8.0)),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::splat(0.0),
                border_width: Vec4::splat(0.0),
            }],
        );

        test_converter(
            17.0,
            17.0,
            ui,
            &[DrawElement::Rectangle {
                bounds: Rectangle::from_position_size(Vec2::new(5.0, 5.0), Vec2::new(8.0, 8.0)),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::splat(0.0),
                border_width: Vec4::splat(0.0),
            }],
        );
    }

    #[test]
    fn self_alignment_with_parent_border_thickness() {
        fn test_self_alignment_basic(alignment: Alignment, expected_position: Vec2) {
            test_converter(
                32.0,
                32.0,
                UiNode::Box(BoxProps {
                    // Border thickness of 4.0 makes the parent container equivalent to a 24.0 size container.
                    modifiers: Modifiers::new().border_thickness(BorderThickness::all(4.0)),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .width(Extent::Px(8.0))
                            .height(Extent::Px(8.0))
                            .self_alignment(alignment),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::ZERO, Vec2::new(32.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::splat(4.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(expected_position, Vec2::new(8.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        test_self_alignment_basic(Alignment::Center, Vec2::new(12.0, 12.0));
        test_self_alignment_basic(Alignment::Right, Vec2::new(20.0, 12.0));
        test_self_alignment_basic(Alignment::TopRight, Vec2::new(20.0, 20.0));
        test_self_alignment_basic(Alignment::Top, Vec2::new(12.0, 20.0));
        test_self_alignment_basic(Alignment::TopLeft, Vec2::new(4.0, 20.0));
        test_self_alignment_basic(Alignment::Left, Vec2::new(4.0, 12.0));
        test_self_alignment_basic(Alignment::BottomLeft, Vec2::new(4.0, 4.0));
        test_self_alignment_basic(Alignment::Bottom, Vec2::new(12.0, 4.0));
        test_self_alignment_basic(Alignment::BottomRight, Vec2::new(20.0, 4.0));
    }

    mod boxes {
        use super::*;

        #[test]
        fn box_different_padding_values() {
            test_converter(
                32.0,
                32.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().padding(Padding::new(1.0, 2.0, 4.0, 8.0)),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 1.0), Vec2::new(32.0 - 10.0, 32.0 - 5.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn box_different_border_thickness_values() {
            test_converter(
                32.0,
                32.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().border_thickness(BorderThickness::new(1.0, 2.0, 4.0, 8.0)),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::new(1.0, 2.0, 4.0, 8.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0, 2.0), Vec2::new(32.0 - 5.0, 32.0 - 10.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn box_fit_content_with_fill_parent_child() {
            test_converter(
                128.0,
                128.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .self_alignment(Alignment::BottomLeft),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .self_alignment(Alignment::TopRight),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(8.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 128.0 - 64.0 - 8.0), Vec2::new(96.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn box_fit_content_with_fill_parent_child_all_children_with_borders() {
            /* In this case, children having border should not affect in any way the parent size. */
            test_converter(
                128.0,
                128.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .border_thickness(BorderThickness::all(8.0))
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .border_thickness(BorderThickness::all(8.0))
                                .self_alignment(Alignment::BottomLeft),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .border_thickness(BorderThickness::all(8.0))
                                .self_alignment(Alignment::TopRight),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(8.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 128.0 - 64.0 - 8.0), Vec2::new(96.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                ],
            );
        }

        #[test]
        fn box_fit_content_with_fill_parent_child_parent_with_border() {
            test_converter(
                128.0,
                128.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .border_thickness(BorderThickness::all(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .self_alignment(Alignment::BottomLeft),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .self_alignment(Alignment::TopRight),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0 + 16.0, 64.0 + 16.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(8.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(8.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0 + 64.0 - 8.0), Vec2::new(96.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn box_fit_content_with_fill_parent_child_parent_with_padding() {
            /* Should be functionally almost equivalent to box_fit_content_with_fill_parent_child_parent_with_border */
            test_converter(
                128.0,
                128.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .padding(Padding::all(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(8.0))
                                .height(Extent::Px(64.0))
                                .self_alignment(Alignment::BottomLeft),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(96.0))
                                .height(Extent::Px(8.0))
                                .self_alignment(Alignment::TopRight),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(96.0 + 16.0, 64.0 + 16.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(96.0, 64.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(8.0, 64.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0 + 64.0 - 8.0), Vec2::new(96.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }
    }

    mod columns {
        use super::*;

        #[test]
        fn basic_column() {
            test_converter(
                32.0,
                128.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new(),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new().height(Extent::Px(24.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new().height(Extent::Px(48.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 128.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 128.0 - 24.0), Vec2::new(32.0, 24.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0, 128.0 - 24.0 - 48.0),
                            Vec2::new(32.0, 48.0),
                        ),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn basic_column_child_with_padding() {
            test_converter(
                32.0,
                128.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new(),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new().height(Extent::Px(24.0)).padding(Padding::all(2.0)),
                            children: vec![UiNode::Box(BoxProps {
                                modifiers: Modifiers::new(),
                                children: vec![],
                            })],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new().height(Extent::Px(48.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 128.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 128.0 - 24.0), Vec2::new(32.0, 24.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0 + 2.0, 128.0 - 24.0 + 2.0),
                            Vec2::new(32.0 - 4.0, 24.0 - 4.0),
                        ),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0, 128.0 - 24.0 - 48.0),
                            Vec2::new(32.0, 48.0),
                        ),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn basic_column_single_child_with_margin() {
            test_converter(
                32.0,
                128.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new(),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().height(Extent::Px(16.0)).margin(Margin::all(2.0)),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 128.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0 + 2.0, 128.0 - 16.0 - 2.0),
                            Vec2::new(32.0 - 4.0, 16.0),
                        ),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn column_both_fit_content_with_fill_parent_child() {
            test_converter(
                256.0,
                256.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0, 32.0 + 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 32.0), Vec2::new(64.0, 32.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0, 32.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn column_fit_content_padding() {
            test_converter(
                256.0,
                256.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FillParent)
                        .height(Extent::FitContent)
                        .padding(Padding::all(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().height(Extent::Px(32.0)),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(256.0, 32.0 + 16.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(8.0, 8.0), Vec2::new(256.0 - 16.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn column_fit_content_single_child_fill_parent() {
            test_converter(
                32.0,
                128.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .self_alignment(Alignment::BottomLeft)
                        .width(Extent::FitContent)
                        .height(Extent::FitContent),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().width(Extent::FillParent).height(Extent::FillParent),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn column_fit_content_single_child_fill_parent_with_margin() {
            test_converter(
                32.0,
                128.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .self_alignment(Alignment::BottomLeft)
                        .width(Extent::FitContent)
                        .height(Extent::FitContent),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .width(Extent::FillParent)
                            .height(Extent::FillParent)
                            .margin(Margin::all(4.0)),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(8.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(0.0, 0.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }

        #[test]
        fn column_fit_content_hor_child_fill_parent() {
            test_converter(
                256.0,
                256.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .self_alignment(Alignment::BottomLeft)
                        .width(Extent::FitContent)
                        .height(Extent::Px(48.0)),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0, 48.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0, 48.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, -48.0), Vec2::new(64.0, 48.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }
    }

    mod rows {
        use super::*;

        #[test]
        fn row_both_fit_content_with_fill_parent_child() {
            test_converter(
                256.0,
                256.0,
                UiNode::Row(RowProps {
                    modifiers: Modifiers::new()
                        .width(Extent::FitContent)
                        .height(Extent::FitContent)
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 0.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(64.0))
                                .height(Extent::Px(32.0))
                                .fill_color(Color::new(0.0, 1.0, 0.0, 0.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0 + 64.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(64.0, 32.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(64.0, 0.0), Vec2::new(64.0, 32.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 0.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::splat(0.0),
                        border_width: Vec4::splat(0.0),
                    },
                ],
            );
        }
    }

    mod weight {
        use super::*;

        #[test]
        fn column_weight() {
            test_converter(
                256.0,
                256.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(100.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(20.0))
                                .weight(1.0)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(40.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 100.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0, 100.0 - 40.0),
                            Vec2::new(50.0, 20.0 + 20.0),
                        ),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 40.0 + 20.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn row_weight() {
            test_converter(
                256.0,
                256.0,
                UiNode::Row(RowProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(100.0))
                        .height(Extent::Px(50.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(20.0))
                                .weight(1.0)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::Px(40.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(20.0 + 20.0, 50.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(40.0, 0.0), Vec2::new(40.0 + 20.0, 50.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn column_weight_rounding() {
            test_converter(
                256.0,
                256.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(100.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![],
                        }),
                        UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .height(Extent::Px(0.0))
                                .weight(1.0)
                                .fill_color(Color::new(0.0, 0.0, 1.0, 1.0)),
                            children: vec![],
                        }),
                    ],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 100.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 100.0 - 34.0), Vec2::new(50.0, 34.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(
                            Vec2::new(0.0, 100.0 - 34.0 - 33.0),
                            Vec2::new(50.0, 33.0),
                        ),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 33.0)),
                        fill_color: Color::new(0.0, 0.0, 1.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn column_weight_respects_margin() {
            test_converter(
                256.0,
                256.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .margin(Margin::all(4.0)) // This margin should only allow for a height of 0.
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(50.0 - 8.0, 0.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn column_weight_respects_border_thickness() {
            test_converter(
                256.0,
                256.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .border_thickness(BorderThickness::all(3.0)) // This border thickness should only allow for a height of the child of 2.0.
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![],
                        })],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::splat(3.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(3.0, 3.0), Vec2::new(50.0 - 6.0, 2.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn column_weight_respects_padding() {
            test_converter(
                256.0,
                256.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .padding(Padding::all(3.0)) // This padding should only allow for a height of the child of 2.0.
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![],
                        })],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(3.0, 3.0), Vec2::new(50.0 - 6.0, 2.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn column_weight_transfers_to_nested_children_correctly() {
            test_converter(
                256.0,
                256.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(50.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .height(Extent::Px(0.0))
                            .weight(1.0)
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![UiNode::Box(BoxProps {
                            modifiers: Modifiers::new()
                                .width(Extent::FillParent)
                                .height(Extent::FillParent)
                                .fill_color(Color::new(0.0, 1.0, 0.0, 1.0)),
                            children: vec![UiNode::Box(BoxProps {
                                modifiers: Modifiers::new()
                                    .width(Extent::FillParent)
                                    .height(Extent::FillParent)
                                    .fill_color(Color::new(0.0, 0.0, 1.0, 1.0)),
                                children: vec![],
                            })],
                        })],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::new(0.0, 1.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 8.0)),
                        fill_color: Color::new(0.0, 0.0, 1.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }
    }

    mod rounding {
        use super::*;

        #[test]
        fn border_thickness_rounding() {
            test_converter(
                32.0,
                32.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().border_thickness(BorderThickness::all(3.5)), // Should all be rounded to 4.0
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::splat(4.0),
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(32.0 - 8.0, 32.0 - 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn padding_rounding() {
            test_converter(
                32.0,
                32.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().padding(Padding::all(3.5)), // Should all be rounded to 4.0
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(32.0 - 8.0, 32.0 - 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn margin_rounding() {
            test_converter(
                32.0,
                32.0,
                UiNode::Box(BoxProps {
                    modifiers: Modifiers::new().margin(Margin::all(3.5)), // Should all be rounded to 4.0
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new().fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(32.0 - 8.0, 32.0 - 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(4.0, 4.0), Vec2::new(32.0 - 8.0, 32.0 - 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }

        #[test]
        fn root_rounding() {
            /* This test checks that the position and the size passed to ui.to_draw_data() get correctly rounded. */
            let root_position = Vec2::splat(0.5); // Should be rounded to (1.0, 1.0)
            let root_size = Vec2::splat(31.5); // Should be rounded to (32.0, 32.0)

            let ui = UiNode::Box(BoxProps {
                modifiers: Modifiers::new(),
                children: vec![],
            });

            let draw_data = convert_to_draw_data(root_position, root_size, ui);

            let expected = vec![DrawElement::Rectangle {
                bounds: Rectangle::from_position_size(Vec2::new(1.0, 1.0), Vec2::new(32.0, 32.0)),
                fill_color: Color::ZERO,
                border_color: Color::ZERO,
                border_radius: Vec4::ZERO,
                border_width: Vec4::ZERO,
            }];

            pretty_assertions::assert_eq!(&expected, &draw_data);
        }

        #[test]
        fn column_horizontal_rounding() {
            test_converter(
                32.0,
                32.0,
                UiNode::Column(ColumnProps {
                    modifiers: Modifiers::new()
                        .width(Extent::Px(9.0))
                        .height(Extent::Px(8.0))
                        .self_alignment(Alignment::BottomLeft),
                    children: vec![UiNode::Box(BoxProps {
                        modifiers: Modifiers::new()
                            .width(Extent::Px(8.0))
                            .self_alignment(Alignment::Center)
                            .fill_color(Color::new(1.0, 0.0, 0.0, 1.0)),
                        children: vec![],
                    })],
                }),
                &[
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(0.0, 0.0), Vec2::new(9.0, 8.0)),
                        fill_color: Color::ZERO,
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                    DrawElement::Rectangle {
                        bounds: Rectangle::from_position_size(Vec2::new(1.0, 0.0), Vec2::new(8.0, 8.0)),
                        fill_color: Color::new(1.0, 0.0, 0.0, 1.0),
                        border_color: Color::ZERO,
                        border_radius: Vec4::ZERO,
                        border_width: Vec4::ZERO,
                    },
                ],
            )
        }
    }
}

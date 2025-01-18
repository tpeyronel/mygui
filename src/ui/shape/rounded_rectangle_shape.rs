use dyn_partial_eq::DynPartialEq;
use glam::Vec2;

use crate::{
    color::Color,
    ui::{
        border_radius::BorderRadius, border_thickness::BorderThickness, color_mesh_builder::ColorMeshBuilder, Layout,
        Modifiers,
    },
};

use super::{Shape, ShapeData};

#[derive(Debug, Clone, Hash, PartialEq, DynPartialEq)]
pub struct RoundedRectangleShape;

impl Shape for RoundedRectangleShape {
    fn to_shape_data(&self, layout: &Layout, modifiers: &Modifiers) -> ShapeData {
        let fill_color = modifiers.fill_color;
        let border_color = modifiers.border_color;
        let border_thickness = modifiers.border_thickness;
        let border_radius = modifiers.border_radius;

        create_rounded_rectangle(layout, fill_color, border_color, border_thickness, border_radius)
    }
}

fn create_rounded_rectangle(
    layout: &Layout,
    fill_color: Color,
    border_color: Color,
    border_thickness: BorderThickness,
    border_radius: BorderRadius,
) -> ShapeData {
    let mut bg_builder = ColorMeshBuilder::new();
    let mut fg_builder = ColorMeshBuilder::new();

    let bl = CornerVertices::new(
        layout.border_position(),
        border_radius.bottom_left(),
        Vec2::new(border_thickness.left(), border_thickness.bottom()),
        Vec2::new(1.0, 1.0),
        fill_color,
        border_color,
        &mut bg_builder,
        &mut fg_builder,
    );

    let br = CornerVertices::new(
        layout.border_position() + layout.border_size().with_y(0.0),
        border_radius.bottom_right(),
        Vec2::new(border_thickness.right(), border_thickness.bottom()),
        Vec2::new(-1.0, 1.0),
        fill_color,
        border_color,
        &mut bg_builder,
        &mut fg_builder,
    );

    let tr = CornerVertices::new(
        layout.border_position() + layout.border_size(),
        border_radius.top_right(),
        Vec2::new(border_thickness.right(), border_thickness.top()),
        Vec2::new(-1.0, -1.0),
        fill_color,
        border_color,
        &mut bg_builder,
        &mut fg_builder,
    );

    let tl = CornerVertices::new(
        layout.border_position() + layout.border_size().with_x(0.0),
        border_radius.top_left(),
        Vec2::new(border_thickness.left(), border_thickness.top()),
        Vec2::new(1.0, -1.0),
        fill_color,
        border_color,
        &mut bg_builder,
        &mut fg_builder,
    );

    // Fill inner center
    bg_builder.add_quad(bl.fill_hor.idx, br.fill_hor.idx, tr.fill_hor.idx, tl.fill_hor.idx);

    // Fill inner left side
    match (bl.fill_hor.idx != bl.fill_ver.idx, tl.fill_hor.idx != tl.fill_ver.idx) {
        (false, false) => (),
        (false, true) => bg_builder.add_triangle(bl.fill_hor.idx, tl.fill_hor.idx, tl.fill_ver.idx),
        (true, false) => bg_builder.add_triangle(bl.fill_ver.idx, bl.fill_hor.idx, tl.fill_hor.idx),
        (true, true) => bg_builder.add_quad(bl.fill_ver.idx, bl.fill_hor.idx, tl.fill_hor.idx, tl.fill_ver.idx),
    }

    // Fill inner right side
    match (br.fill_hor.idx != br.fill_ver.idx, tr.fill_hor.idx != tr.fill_ver.idx) {
        (false, false) => (),
        (false, true) => bg_builder.add_triangle(br.fill_hor.idx, tr.fill_ver.idx, tr.fill_hor.idx),
        (true, false) => bg_builder.add_triangle(br.fill_hor.idx, br.fill_ver.idx, tr.fill_hor.idx),
        (true, true) => bg_builder.add_quad(br.fill_hor.idx, br.fill_ver.idx, tr.fill_ver.idx, tr.fill_hor.idx),
    }

    /* Fill borders */
    /* TODO: optimize unnecessary vertices and triangles */

    // Bottom border
    fg_builder.add_quad(
        bl.border_outer_hor.idx,
        br.border_outer_hor.idx,
        br.border_inner_hor.idx,
        bl.border_inner_hor.idx,
    );

    // Right border
    fg_builder.add_quad(
        br.border_inner_ver.idx,
        br.border_outer_ver.idx,
        tr.border_outer_ver.idx,
        tr.border_inner_ver.idx,
    );

    // Top border
    fg_builder.add_quad(
        tl.border_inner_hor.idx,
        tr.border_inner_hor.idx,
        tr.border_outer_hor.idx,
        tl.border_outer_hor.idx,
    );

    // Left border
    fg_builder.add_quad(
        bl.border_outer_ver.idx,
        bl.border_inner_ver.idx,
        tl.border_inner_ver.idx,
        tl.border_outer_ver.idx,
    );

    /* Fill corners (with corner borders) */

    emit_rectangle_corners(
        &bl,
        border_radius.bottom_left(),
        &fill_color,
        &border_color,
        &mut bg_builder,
        &mut fg_builder,
    );

    emit_rectangle_corners(
        &br,
        border_radius.bottom_right(),
        &fill_color,
        &border_color,
        &mut bg_builder,
        &mut fg_builder,
    );

    emit_rectangle_corners(
        &tr,
        border_radius.top_right(),
        &fill_color,
        &border_color,
        &mut bg_builder,
        &mut fg_builder,
    );

    emit_rectangle_corners(
        &tl,
        border_radius.top_left(),
        &fill_color,
        &border_color,
        &mut bg_builder,
        &mut fg_builder,
    );

    ShapeData {
        foreground_mesh: fg_builder.build(),
        background_mesh: bg_builder.build(),
    }
}

fn emit_rectangle_corners(
    v: &CornerVertices,
    corner_radius: f32,
    fill_color: &Color,
    border_color: &Color,
    bg_builder: &mut ColorMeshBuilder,
    fg_builder: &mut ColorMeshBuilder,
) {
    if corner_radius > 0.0 {
        emit_rectangle_corners_rec(
            v,
            0.0,
            v.border_outer_ver.idx,
            v.border_inner_ver.idx,
            v.fill_ver.idx,
            std::f32::consts::FRAC_PI_2,
            v.border_outer_hor.idx,
            v.border_inner_hor.idx,
            v.fill_hor.idx,
            compute_corner_depth(corner_radius),
            fill_color,
            border_color,
            bg_builder,
            fg_builder,
        );
    }
}

fn compute_corner_depth(corner_radius: f32) -> u32 {
    const MAGIC: f32 = 0.5;
    (MAGIC * corner_radius.log2()).round() as u32
}

fn emit_rectangle_corners_rec(
    v: &CornerVertices,
    left_angle: f32,
    left_border_outer_idx: u32,
    left_border_inner_idx: u32,
    left_fill_idx: u32,
    right_angle: f32,
    right_border_outer_idx: u32,
    right_border_inner_idx: u32,
    right_fill_idx: u32,
    depth: u32,
    fill_color: &Color,
    border_color: &Color,
    bg_builder: &mut ColorMeshBuilder,
    fg_builder: &mut ColorMeshBuilder,
) {
    let alpha = (left_angle + right_angle) * 0.5;

    let (sin_alpha, cos_alpha) = alpha.sin_cos();

    let outer_pos = Vec2::new(
        lerp(v.border_outer_hor.pos.x, v.border_outer_ver.pos.x, cos_alpha),
        lerp(v.border_outer_ver.pos.y, v.border_outer_hor.pos.y, sin_alpha),
    );

    let inner_pos = Vec2::new(
        lerp(v.fill_hor.pos.x, v.fill_ver.pos.x, cos_alpha),
        lerp(v.fill_ver.pos.y, v.fill_hor.pos.y, sin_alpha),
    );

    let fill_idx = bg_builder.add_vertex(inner_pos, *fill_color);
    let border_outer_idx = fg_builder.add_vertex(outer_pos, *border_color);
    let border_inner_idx = fg_builder.add_vertex(inner_pos, *border_color);

    bg_builder.add_triangle(left_fill_idx, fill_idx, right_fill_idx);

    if depth > 0 {
        emit_rectangle_corners_rec(
            v,
            left_angle,
            left_border_outer_idx,
            left_border_inner_idx,
            left_fill_idx,
            alpha,
            border_outer_idx,
            border_inner_idx,
            fill_idx,
            depth - 1,
            fill_color,
            border_color,
            bg_builder,
            fg_builder,
        );
        emit_rectangle_corners_rec(
            v,
            alpha,
            border_outer_idx,
            border_inner_idx,
            fill_idx,
            right_angle,
            right_border_outer_idx,
            right_border_inner_idx,
            right_fill_idx,
            depth - 1,
            fill_color,
            border_color,
            bg_builder,
            fg_builder,
        );
    } else {
        // Emit border thickness
        fg_builder.add_triangle(left_border_inner_idx, border_inner_idx, border_outer_idx);
        fg_builder.add_triangle(left_border_inner_idx, border_outer_idx, left_border_outer_idx);
        fg_builder.add_triangle(border_inner_idx, right_border_inner_idx, right_border_outer_idx);
        fg_builder.add_triangle(border_inner_idx, right_border_outer_idx, border_outer_idx);
    }
}

fn lerp(x: f32, y: f32, a: f32) -> f32 {
    x * (1.0 - a) + y * a
}

struct CornerVertices {
    fill_hor: CornerVertex,
    fill_ver: CornerVertex,
    border_inner_hor: CornerVertex,
    border_inner_ver: CornerVertex,
    border_outer_hor: CornerVertex,
    border_outer_ver: CornerVertex,
}

impl CornerVertices {
    pub fn new(
        corner_pos: Vec2,
        corner_radius: f32,
        corner_thickness: Vec2,
        rotation: Vec2,
        fill_color: Color,
        border_color: Color,
        bg_builder: &mut ColorMeshBuilder,
        fg_builder: &mut ColorMeshBuilder,
    ) -> Self {
        let fill_hor_pos = corner_pos + Vec2::new(corner_radius.max(corner_thickness.x), corner_thickness.y) * rotation;
        let fill_hor_idx = bg_builder.add_vertex(fill_hor_pos, fill_color);

        let fill_ver_pos = corner_pos + Vec2::new(corner_thickness.x, corner_radius.max(corner_thickness.y)) * rotation;
        let fill_ver_idx = if fill_ver_pos == fill_hor_pos {
            fill_hor_idx
        } else {
            bg_builder.add_vertex(fill_ver_pos, fill_color)
        };

        let border_inner_hor_pos = fill_hor_pos;
        let border_inner_hor_idx = fg_builder.add_vertex(border_inner_hor_pos, border_color);

        let border_inner_ver_pos = fill_ver_pos;
        let border_inner_ver_idx = if border_inner_ver_pos == border_inner_hor_pos {
            border_inner_hor_idx
        } else {
            fg_builder.add_vertex(border_inner_ver_pos, border_color)
        };

        let border_outer_hor_pos = corner_pos + Vec2::new(corner_radius, 0.0) * rotation;
        let border_outer_hor_idx = if border_outer_hor_pos == border_inner_hor_pos {
            border_inner_hor_idx
        } else {
            fg_builder.add_vertex(border_outer_hor_pos, border_color)
        };

        let border_outer_ver_pos = corner_pos + Vec2::new(0.0, corner_radius) * rotation;
        let border_outer_ver_idx = if border_outer_ver_pos == border_outer_hor_pos {
            border_outer_hor_idx
        } else {
            fg_builder.add_vertex(border_outer_ver_pos, border_color)
        };

        Self {
            fill_hor: CornerVertex {
                pos: fill_hor_pos,
                idx: fill_hor_idx,
            },
            fill_ver: CornerVertex {
                pos: fill_ver_pos,
                idx: fill_ver_idx,
            },
            border_inner_hor: CornerVertex {
                pos: border_inner_hor_pos,
                idx: border_inner_hor_idx,
            },
            border_inner_ver: CornerVertex {
                pos: border_inner_ver_pos,
                idx: border_inner_ver_idx,
            },
            border_outer_hor: CornerVertex {
                pos: border_outer_hor_pos,
                idx: border_outer_hor_idx,
            },
            border_outer_ver: CornerVertex {
                pos: border_outer_ver_pos,
                idx: border_outer_ver_idx,
            },
        }
    }
}

struct CornerVertex {
    pos: Vec2,
    idx: u32,
}

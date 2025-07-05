use macroquad::prelude::*;

use crate::{
    mesh::{MMesh, PointId},
    util::mouse_position_point,
};

pub enum State {
    Idle(Option<PointId>),
    Drag(PointId),
}

pub struct Path {
    state: State,
}

impl Path {
    pub fn new() -> Self {
        Self {
            state: State::Idle(None),
        }
    }
    pub fn update(&mut self, mesh: &mut MMesh) {
        match &mut self.state {
            State::Idle(point_id) => {
                let mouse_position = mouse_position_point();
                if is_mouse_button_pressed(MouseButton::Left) {
                    *point_id = mesh
                        .closest_point(mouse_position, Some(3.))
                        .map(|(id, _)| id);

                    println!("point id {:?}", point_id);

                    if let Some(point_id) = *point_id {
                        self.state = State::Drag(point_id);
                    }
                }
            }
            State::Drag(point_id) => {
                if is_mouse_button_released(MouseButton::Left) {
                    self.state = State::Idle(Some(*point_id));
                } else {
                    let mouse_position = mouse_position_point();
                    mesh.set_point(*point_id, mouse_position);
                }
            }
        }
    }

    pub fn draw(&self, mesh: &MMesh) {
        match self.state {
            State::Idle(point_id) => {
                if let Some(point) = point_id.and_then(|id| mesh.get_point(id)) {
                    draw_circle(point.x as f32, point.y as f32, 3., SKYBLUE);
                }
            }
            State::Drag(point_id) => {
                if let Some(point) = mesh.get_point(point_id) {
                    draw_circle(point.x as f32, point.y as f32, 3., SKYBLUE);
                }
            }
        }
    }
}

use kurbo::{CubicBez, Line, Point, QuadBez};
use macroquad::prelude::*;

use crate::{
    mesh::{MMesh, PointId},
    util::{draw_bez, dvec2_to_point, mouse_position_dvec2, mouse_position_point, point_to_dvec2},
};

#[derive(Clone, Copy, Debug)]
enum State {
    Idle,
    DragStartPoint(PointId, Option<Point>),
    IdleStartPoint(PointId, Option<Point>),
    DragSecondPoint(PointId, Option<Point>, Option<Point>, PointId),
}

#[derive(Clone, Debug)]
pub struct Pen {
    state: State,
}

impl Pen {
    pub fn new() -> Pen {
        Pen { state: State::Idle }
    }

    pub fn update(&mut self, mesh: &mut MMesh) {
        match &mut self.state {
            State::Idle => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    // create a new point
                    let (x, y) = mouse_position();
                    let mouse_position = Point::new(x as f64, y as f64);

                    let point_id = mesh
                        .closest_point(mouse_position)
                        .and_then(|(id, point)| {
                            if point.distance(mouse_position) < 3. {
                                None
                            } else {
                                Some(id)
                            }
                        })
                        .unwrap_or(mesh.append_point(mouse_position));

                    // transition to the drag state
                    self.state = State::DragStartPoint(point_id, None);
                }
            }
            State::DragStartPoint(p1, p2) => {
                if is_key_pressed(KeyCode::Escape) {
                    mesh.remove_floating_point(*p1);
                    self.state = State::Idle;
                } else if is_mouse_button_released(MouseButton::Left) {
                    self.state = State::IdleStartPoint(*p1, *p2);
                } else {
                    // Get mouse position
                    let mouse_position = mouse_position_point();
                    // Calculate handle position
                    *p2 = if mesh
                        .get_point(*p1)
                        .is_some_and(|point| point.distance(mouse_position) > 3.)
                    {
                        Some(mouse_position)
                    } else {
                        None
                    };
                }
            }
            State::IdleStartPoint(p1, p2) => {
                let (x, y) = mouse_position();
                let mouse_position = Point::new(x as f64, y as f64);

                if is_key_pressed(KeyCode::Escape) {
                    mesh.remove_floating_point(*p1);
                    self.state = State::Idle;
                } else if is_mouse_button_pressed(MouseButton::Left) {
                    if mesh.get_point(*p1).unwrap().distance(mouse_position) < 5. {
                        self.state = State::DragStartPoint(*p1, Some(mouse_position));
                    } else {
                        // create a new endpoint
                        let p4 = mesh.append_point(mouse_position);
                        // transition to the drag state
                        self.state = State::DragSecondPoint(*p1, *p2, None, p4);
                    }
                }
            }
            State::DragSecondPoint(p1, p2, p3, p4) => {
                if is_key_pressed(KeyCode::Escape) {
                    mesh.remove_floating_point(*p1);
                    mesh.remove_floating_point(*p4);
                    self.state = State::Idle;
                } else if is_mouse_button_released(MouseButton::Left) {
                    let (p2, p3) = match (p2, p3) {
                        (Some(p2), Some(p3)) => {
                            (Some(mesh.append_point(*p2)), Some(mesh.append_point(*p3)))
                        }
                        (Some(p2), None) | (None, Some(p2)) => (Some(mesh.append_point(*p2)), None),
                        (None, None) => (None, None),
                    };
                    mesh.append_segment(*p1, p2, p3, *p4);
                    self.state = State::IdleStartPoint(*p4, Some(mouse_position_point()));
                } else {
                    // Get mouse position
                    let mouse_position = mouse_position_dvec2();
                    // Calculate handle position
                    let p4_pos = point_to_dvec2(mesh.get_point(*p4).unwrap());
                    *p3 = if p4_pos.distance(mouse_position) > 3. {
                        let p3 = 2. * p4_pos - mouse_position;
                        Some(dvec2_to_point(p3))
                    } else {
                        None
                    };
                }
            }
        }
    }

    pub fn draw(&self, mesh: &MMesh) {
        match self.state {
            State::Idle => {}
            State::DragStartPoint(p1, p2) => {
                let Some(p1) = mesh.get_point(p1) else {
                    return;
                };
                draw_circle(p1.x as f32, p1.y as f32, 3., SKYBLUE);

                if let Some(p2) = p2 {
                    draw_circle(p2.x as f32, p2.y as f32, 3., SKYBLUE);
                    draw_line(
                        p1.x as f32,
                        p1.y as f32,
                        p2.x as f32,
                        p2.y as f32,
                        1.,
                        SKYBLUE,
                    );
                }
            }
            State::IdleStartPoint(p1, p2) => {
                let p1 = mesh.get_point(p1).unwrap();
                draw_circle(p1.x as f32, p1.y as f32, 3., SKYBLUE);

                // If there is no handle then draw a line instead.
                let p2 = p2.unwrap_or(mouse_position_point());
                draw_circle(p2.x as f32, p2.y as f32, 3., SKYBLUE);
                draw_line(
                    p1.x as f32,
                    p1.y as f32,
                    p2.x as f32,
                    p2.y as f32,
                    1.,
                    SKYBLUE,
                );
                let quad_bez = QuadBez::new(p1, p2, mouse_position_point());
                draw_bez(quad_bez);
            }
            State::DragSecondPoint(p1, p2, p3, p4) => {
                let p1 = mesh.get_point(p1).unwrap();
                let p4 = mesh.get_point(p4).unwrap();

                draw_circle(p1.x as f32, p1.y as f32, 3., SKYBLUE);
                draw_circle(p4.x as f32, p4.y as f32, 3., SKYBLUE);

                match (p2, p3) {
                    (Some(p2), Some(p3)) => draw_bez(CubicBez::new(p1, p2, p3, p4)),
                    (Some(p2), None) | (None, Some(p2)) => draw_bez(QuadBez::new(p1, p2, p4)),
                    (None, None) => {
                        draw_bez(Line::new(p1, p4));
                    }
                }
            }
        }
    }
}

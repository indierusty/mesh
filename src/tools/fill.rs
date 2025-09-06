use kurbo::Point;
use macroquad::prelude::*;

use crate::mesh::DynamicMesh;

const COLOR_BOX_SIZE: f32 = 25.0;

pub const COLORS: [Color; 14] = [
    LIGHTGRAY, GRAY, GOLD, ORANGE, PINK, DARKGREEN, RED, SKYBLUE, BLUE, PURPLE, BEIGE, BROWN,
    WHITE, BLACK,
];

pub enum State {
    // Option of index of color in the COLORS array
    Idle(Option<usize>),
}

pub struct FillTool {
    state: State,
    pos_x: f32,
    pos_y: f32,
}

impl FillTool {
    pub fn new() -> Self {
        Self {
            state: State::Idle(None),
            pos_x: 5.,
            pos_y: 50.,
        }
    }

    fn colors_position(&self) -> Vec<(usize, f32, f32)> {
        (0..COLORS.len())
            .into_iter()
            .map(|i| (i, self.pos_x, self.pos_y + i as f32 * COLOR_BOX_SIZE))
            .collect()
    }

    pub fn update(&mut self, mesh: &mut DynamicMesh) {
        match self.state {
            State::Idle(color_idx) => {
                let (mouse_x, mouse_y) = mouse_position();
                let colors_position = self.colors_position();
                let selection = colors_position.into_iter().find(|&(_i, x, y)| {
                    Rect::new(x, y, COLOR_BOX_SIZE, COLOR_BOX_SIZE)
                        .contains(Vec2::new(mouse_x, mouse_y))
                });

                if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some((i, _, _)) = selection {
                        self.state = State::Idle(Some(i));
                    } else {
                        if let Some(color_idx) = &color_idx {
                            mesh.dynamic_data.apply_style(
                                Some(COLORS[*color_idx]),
                                Point::new(mouse_x as f64, mouse_y as f64),
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn draw(&self) {
        match self.state {
            State::Idle(color_idx) => {
                let (x, y) = mouse_position();
                draw_circle_lines(x, y, 5., 2., BLACK);
                match color_idx {
                    Some(color_idx) => draw_circle(x, y, 5., COLORS[color_idx]),
                    None => draw_circle(x, y, 5., BLANK),
                }

                for (i, x, y) in self.colors_position() {
                    draw_rectangle(x, y, COLOR_BOX_SIZE, COLOR_BOX_SIZE, COLORS[i]);
                }
            }
        }
    }
}

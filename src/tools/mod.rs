use fill::FillTool;
use macroquad::input::{KeyCode, is_key_pressed};
use path::PathTool;
use pen::PenTool;

use crate::mesh::DynamicMesh;

mod fill;
mod path;
mod pen;

enum ToolType {
    Pen,
    Path,
    Fill,
}

pub struct Tools {
    active_tool: ToolType,
    pen: PenTool,
    path: PathTool,
    fill: FillTool,
}

impl Tools {
    pub fn new() -> Self {
        Self {
            active_tool: ToolType::Pen,
            pen: PenTool::new(),
            path: PathTool::new(),
            fill: FillTool::new(),
        }
    }

    pub fn update(&mut self, mesh: &mut DynamicMesh) {
        if is_key_pressed(KeyCode::Z) {
            self.active_tool = ToolType::Pen;
        }
        if is_key_pressed(KeyCode::X) {
            self.active_tool = ToolType::Path;
        }
        if is_key_pressed(KeyCode::C) {
            self.active_tool = ToolType::Fill;
        }
        match self.active_tool {
            ToolType::Pen => self.pen.update(mesh),
            ToolType::Path => self.path.update(mesh),
            ToolType::Fill => self.fill.update(mesh),
        }
    }

    pub fn draw(&self, mesh: &DynamicMesh) {
        match self.active_tool {
            ToolType::Pen => self.pen.draw(mesh),
            ToolType::Path => self.path.draw(mesh),
            ToolType::Fill => self.fill.draw(),
        }
    }
}

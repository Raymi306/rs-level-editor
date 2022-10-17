use eframe::egui;
use eframe::egui::Rect;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct HashableVec2 {
    pub x: i64,
    pub y: i64,
}

impl From<egui::widgets::plot::PlotPoint> for HashableVec2 {
    fn from(point: egui::widgets::plot::PlotPoint) -> HashableVec2 {
        HashableVec2 {
            x: point.x as i64,
            y: point.y as i64,
        }
    }
}

impl From<HashableVec2> for egui::widgets::plot::PlotPoint {
    fn from(point: HashableVec2) -> egui::widgets::plot::PlotPoint {
        egui::widgets::plot::PlotPoint {
            x: point.x as f64,
            y: point.y as f64,
        }
    }
}

pub struct SpritesheetInfo {
    pub sprite_size: u16,
    pub num_rows: u8,
    pub num_cols: u8,
}

impl Default for SpritesheetInfo {
    fn default() -> Self {
        Self {
            sprite_size: 32,
            num_rows: 10,
            num_cols: 10,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Mode {
    DrawBackground,
    DrawForeground,
    Collision,
    Entity,
}

#[derive(PartialEq, Debug)]
pub enum ColumnOrientation {
    Major,
    Minor,
}

#[derive(Debug, Clone)]
pub enum Action {
    ClickForeground(HashableVec2, Rect, Option<Rect>, bool),
    ClickBackground(HashableVec2, Rect, Option<Rect>, bool),
    ClickCollision(HashableVec2),
    ClickEntity(HashableVec2, Option<String>),
}

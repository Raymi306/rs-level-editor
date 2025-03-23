use std::collections::{BTreeSet, HashMap, HashSet};

use eframe::egui;
use eframe::egui::Rect;

mod file;
use file::{pick_file_to, save_file_to};

mod types;
use types::*;

mod plot_panel;
mod popups;
mod side_panel;
mod top_panel;

struct MyApp {
    spritesheet_info: SpritesheetInfo,
    spritesheet_handle: Option<egui::TextureHandle>,
    spritesheet_col_orientation: ColumnOrientation,
    foreground_plotted_tiles: HashMap<HashableVec2, Rect>,
    background_plotted_tiles: HashMap<HashableVec2, Rect>,
    collision_tiles: HashSet<HashableVec2>,
    entity_tiles: HashMap<HashableVec2, String>,
    selected_uv: Option<Rect>,
    selected_entity: Option<HashableVec2>,
    entity_description: String,
    prev_entity_description: String,
    entity_descriptions: BTreeSet<String>,
    current_mode: Mode,
    undo_queue: Vec<Action>,
    redo_queue: Vec<Action>,
    show_entity_popup: bool,
    show_clear_confirmation: bool,
    show_foreground: bool,
    show_background: bool,
    show_collision: bool,
    show_entity: bool,
    show_grid: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            spritesheet_info: SpritesheetInfo::default(),
            spritesheet_handle: None,
            spritesheet_col_orientation: ColumnOrientation::Minor,
            foreground_plotted_tiles: HashMap::new(),
            background_plotted_tiles: HashMap::new(),
            collision_tiles: HashSet::new(),
            entity_tiles: HashMap::new(),
            selected_uv: None,
            selected_entity: None,
            entity_description: "".to_string(),
            prev_entity_description: "".to_string(),
            entity_descriptions: BTreeSet::new(),
            current_mode: Mode::DrawBackground,
            undo_queue: Vec::new(),
            redo_queue: Vec::new(),
            show_entity_popup: false,
            show_clear_confirmation: false,
            show_foreground: true,
            show_background: true,
            show_collision: true,
            show_entity: true,
            show_grid: true,
        }
    }
}

impl MyApp {
    fn clear(&mut self) {
        self.show_clear_confirmation = false;
        self.foreground_plotted_tiles.clear();
        self.background_plotted_tiles.clear();
        self.collision_tiles.clear();
        self.entity_descriptions.clear();
        self.entity_tiles.clear();
        self.undo_queue.clear();
        self.redo_queue.clear();
    }
    fn handle_undo_redo(&mut self, is_undo: bool) {
        let queue = if is_undo {
            &mut self.undo_queue
        } else {
            &mut self.redo_queue
        };
        if let Some(action) = queue.pop() {
            let mut cloned_action = action.clone();
            match action {
                Action::ClickForeground(point, uv, old_uv_maybe, is_drag) => {
                    if let Some(old_uv) = old_uv_maybe {
                        if !is_drag {
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                self.foreground_plotted_tiles.entry(point)
                            {
                                e.insert(old_uv);
                            } else {
                                self.foreground_plotted_tiles.remove(&point);
                            }
                        } else {
                            self.foreground_plotted_tiles.insert(point, old_uv);
                            cloned_action =
                                Action::ClickForeground(point, old_uv, Some(uv), is_drag)
                        }
                    } else if let std::collections::hash_map::Entry::Vacant(e) =
                        self.foreground_plotted_tiles.entry(point)
                    {
                        e.insert(uv);
                    } else {
                        self.foreground_plotted_tiles.remove(&point);
                    }
                }
                Action::ClickBackground(point, uv, old_uv_maybe, is_drag) => {
                    if let Some(old_uv) = old_uv_maybe {
                        if !is_drag {
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                self.background_plotted_tiles.entry(point)
                            {
                                e.insert(old_uv);
                            } else {
                                self.background_plotted_tiles.remove(&point);
                            }
                        } else {
                            self.background_plotted_tiles.insert(point, old_uv);
                            cloned_action =
                                Action::ClickBackground(point, old_uv, Some(uv), is_drag)
                        }
                    } else if let std::collections::hash_map::Entry::Vacant(e) =
                        self.background_plotted_tiles.entry(point)
                    {
                        e.insert(uv);
                    } else {
                        self.background_plotted_tiles.remove(&point);
                    }
                }
                Action::ClickCollision(point) => {
                    if self.collision_tiles.contains(&point) {
                        self.collision_tiles.remove(&point);
                    } else {
                        self.collision_tiles.insert(point);
                    }
                }
                Action::ClickEntity(point, attached_label) => {
                    if let Some(label) = self.entity_tiles.get(&point) {
                        let label_clone = label.clone();
                        self.entity_descriptions.remove(label);
                        self.entity_tiles.remove(&point);
                        cloned_action = Action::ClickEntity(point, Some(label_clone));
                    } else if let Some(label) = attached_label {
                        self.entity_tiles.insert(point, label.clone());
                        self.entity_descriptions.insert(label.clone());
                    }
                }
            };
            if is_undo {
                self.redo_queue.push(cloned_action);
            } else {
                self.undo_queue.push(cloned_action);
            }
        }
    }
    fn toggle_current_mode(&mut self) {
        self.current_mode = match self.current_mode {
            Mode::DrawBackground => Mode::DrawForeground,
            Mode::DrawForeground => Mode::Collision,
            Mode::Collision => Mode::Entity,
            Mode::Entity => Mode::DrawBackground,
        }
    }
    fn handle_toplevel_input(&mut self, ctx: &egui::Context) {
        if self.show_clear_confirmation || self.show_entity_popup {
            return;
        }
        for event in &ctx.input().events {
            if let egui::Event::Key {
                key,
                pressed,
                modifiers,
            } = event
            {
                if !*pressed {
                    match (key, modifiers) {
                        (egui::Key::M, _) => self.toggle_current_mode(),
                        (egui::Key::Z, egui::Modifiers { ctrl, .. }) => {
                            if *ctrl {
                                self.handle_undo_redo(true);
                            }
                        }
                        (egui::Key::R, egui::Modifiers { ctrl, .. }) => {
                            if *ctrl {
                                self.handle_undo_redo(false);
                            }
                        }
                        (egui::Key::O, egui::Modifiers { ctrl, .. }) => {
                            if *ctrl {
                                let mut open_path = None;
                                pick_file_to(&mut open_path, ("Level", &["lvl"]));
                                if let Some(path) = open_path {
                                    self.open(path);
                                }
                            }
                        }
                        (egui::Key::S, egui::Modifiers { ctrl, .. }) => {
                            if *ctrl {
                                let mut save_path = None;
                                save_file_to(&mut save_path, ("Level", &["lvl"]));
                                if let Some(path) = save_path {
                                    self.save(path);
                                }
                            }
                        }
                        _ => (),
                    };
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_clear_confirmation_popup(ctx);
        self.handle_entity_popup(ctx);
        self.handle_toplevel_input(ctx);
        self.top_panel(ctx);
        self.side_panel(ctx);
        self.plot_panel(ctx);
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Level Editor",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

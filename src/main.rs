use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;

use eframe;
use eframe::egui;
use eframe::egui::{Pos2, Rect, Vec2};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct HashableVec2 {
    x: i64,
    y: i64,
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

struct SpritesheetInfo {
    sprite_size: u16,
    num_rows: u8,
    num_cols: u8,
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
enum Mode {
    DrawBackground,
    DrawForeground,
    Collision,
    Entity,
}

#[derive(Debug, Clone)]
enum Action {
    ClickForeground(HashableVec2, Rect),
    ClickBackground(HashableVec2, Rect),
    ClickCollision(HashableVec2),
    ClickEntity(HashableVec2, Option<String>),
}

struct MyApp {
    spritesheet_info: SpritesheetInfo,
    spritesheet_handle: Option<egui::TextureHandle>,
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
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            spritesheet_info: SpritesheetInfo::default(),
            spritesheet_handle: None,
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
        }
    }
}

fn pick_file_to(var: &mut Option<PathBuf>, filter: (&str, &[&str])) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(filter.0, filter.1)
        .pick_file()
    {
        *var = Some(path);
    }
}
fn save_file_to(var: &mut Option<PathBuf>, filter: (&str, &[&str])) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(filter.0, filter.1)
        .save_file()
    {
        *var = Some(path);
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
    fn save(&mut self, path: PathBuf) {
        let display = path.display();
        let mut file = match File::create(&path) {
            Err(why) => {
                println!("Couldn't create {}: {}", display, why);
                return;
            }
            Ok(f) => f,
        };
        if self.background_plotted_tiles.len() == 0 || self.spritesheet_handle.is_none() {
            return;
        }
        let handle = self.spritesheet_handle.as_ref().unwrap();
        let handle_size = handle.size_vec2();
        let len_bg = (self.background_plotted_tiles.len() * 20) as u64;
        let len_fg = (self.foreground_plotted_tiles.len() * 20) as u64;
        let len_collision = (self.collision_tiles.len() * 16) as u64;
        let len_entity = (self.entity_tiles.len() * 64) as u64; // we don't know label length
        let mut buffer: Vec<u8> = Vec::with_capacity((len_bg + len_fg + len_collision + len_entity) as usize);
        buffer.extend_from_slice(&len_bg.to_le_bytes()); // 36
        buffer.extend_from_slice(&len_fg.to_le_bytes()); // 56
        buffer.extend_from_slice(&len_collision.to_le_bytes()); // 72
        for (point, uv) in self.background_plotted_tiles.iter() {
            let x = point.x.to_le_bytes();
            let y = point.y.to_le_bytes();
            let row = ((uv.min.x * handle_size.x) as i16).to_le_bytes();
            let col = ((uv.min.y * handle_size.y) as i16).to_le_bytes();
            buffer.extend_from_slice(&x);
            buffer.extend_from_slice(&y);
            buffer.extend_from_slice(&row);
            buffer.extend_from_slice(&col);
        }
        for (point, uv) in self.foreground_plotted_tiles.iter() {
            let x = point.x.to_le_bytes();
            let y = point.y.to_le_bytes();
            let row = ((uv.min.x * handle_size.x) as i16).to_le_bytes();
            let col = ((uv.min.y * handle_size.y) as i16).to_le_bytes();
            buffer.extend_from_slice(&x);
            buffer.extend_from_slice(&y);
            buffer.extend_from_slice(&row);
            buffer.extend_from_slice(&col);
        }
        for point in self.collision_tiles.iter() {
            let x = point.x.to_le_bytes();
            let y = point.y.to_le_bytes();
            buffer.extend_from_slice(&x);
            buffer.extend_from_slice(&y);
        }
        for (point, label) in self.entity_tiles.iter() {
            let x = point.x.to_le_bytes();
            let y = point.y.to_le_bytes();
            let label_len = (label.len() as u64).to_le_bytes();
            let label = label.as_bytes();
            buffer.extend_from_slice(&x);
            buffer.extend_from_slice(&y);
            buffer.extend_from_slice(&label_len);
            buffer.extend_from_slice(&label);
        }
        file.write_all(&buffer).unwrap();
        file.flush().unwrap();
    }
    fn open(&mut self, path: PathBuf) {
        if self.spritesheet_handle.is_none() {
            return;
        }
        self.clear();
        let handle = self.spritesheet_handle.as_ref().unwrap();
        let handle_size = handle.size_vec2();

        let display = path.display();
        let mut file = match File::open(&path) {
            Err(why) => {
                println!("Couldn't create {}: {}", display, why);
                return;
            }
            Ok(f) => f,
        };
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let mut index = 0;
        let len_bg = u64::from_le_bytes(buf[index..index + 8].try_into().unwrap());
        index += 8;
        let len_fg = u64::from_le_bytes(buf[index..index + 8].try_into().unwrap());
        index += 8;
        let len_collision = u64::from_le_bytes(buf[index..index + 8].try_into().unwrap());
        index += 8;
        let bg_fg_stride_len = (8 + 8 + 2 + 2) as usize;
        let background_bytes = &buf[index..index + len_bg as usize];
        for chunk in background_bytes.chunks_exact(bg_fg_stride_len) {
            let x = i64::from_le_bytes(chunk[0..8].try_into().unwrap());
            let y = i64::from_le_bytes(chunk[8..16].try_into().unwrap());
            let row = i16::from_le_bytes(chunk[16..18].try_into().unwrap());
            let col = i16::from_le_bytes(chunk[18..20].try_into().unwrap());
            let uv_min_x = row as f32 / handle_size.x;
            let uv_min_y = col as f32 / handle_size.y;
            let uv_max_x = (row as f32 + handle_size.x / self.spritesheet_info.num_rows as f32) / handle_size.x;
            let uv_max_y = (col as f32 + handle_size.y / self.spritesheet_info.num_cols as f32) / handle_size.y;
            let uv = Rect {
                min: Pos2 {
                    x: uv_min_x,
                    y: uv_min_y,
                },
                max: Pos2 {
                    x: uv_max_x,
                    y: uv_max_y,
                }
            };
            self.background_plotted_tiles.insert(HashableVec2 { x, y }, uv);
        }
        index += len_bg as usize;
        let foreground_bytes = &buf[index..index + len_fg as usize];
        for chunk in foreground_bytes.chunks_exact(bg_fg_stride_len) {
            let x = i64::from_le_bytes(chunk[0..8].try_into().unwrap());
            let y = i64::from_le_bytes(chunk[8..16].try_into().unwrap());
            let row = i16::from_le_bytes(chunk[16..18].try_into().unwrap());
            let col = i16::from_le_bytes(chunk[18..20].try_into().unwrap());
            let uv_min_x = row as f32 / handle_size.x;
            let uv_min_y = col as f32 / handle_size.y;
            let uv_max_x = (row as f32 + handle_size.x / self.spritesheet_info.num_rows as f32) / handle_size.x;
            let uv_max_y = (col as f32 + handle_size.y / self.spritesheet_info.num_cols as f32) / handle_size.y;
            let uv = Rect {
                min: Pos2 {
                    x: uv_min_x,
                    y: uv_min_y,
                },
                max: Pos2 {
                    x: uv_max_x,
                    y: uv_max_y,
                }
            };
            self.foreground_plotted_tiles.insert(HashableVec2 { x, y }, uv);
        }
        index += len_fg as usize;
        let collision_stride_len = (8 + 8) as usize;
        let collision_bytes = &buf[index..index + len_collision as usize];
        for chunk in collision_bytes.chunks_exact(collision_stride_len) {
            let x = i64::from_le_bytes(chunk[0..8].try_into().unwrap());
            let y = i64::from_le_bytes(chunk[8..16].try_into().unwrap());
            self.collision_tiles.insert(HashableVec2 { x, y });
        }
        index += len_collision as usize;
        let len = buf.len();
        while index < len {
            let x = i64::from_le_bytes(buf[index..index + 8].try_into().unwrap());
            index += 8;
            let y = i64::from_le_bytes(buf[index..index + 8].try_into().unwrap());
            index += 8;
            let label_len = u64::from_le_bytes(buf[index..index + 8].try_into().unwrap());
            index += 8;
            let label = String::from_utf8(buf[index..index + label_len as usize].try_into().unwrap()).unwrap();
            self.entity_tiles.insert(HashableVec2 { x, y }, label);
            index += label_len as usize;
        }
    }
    fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("my_top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut save_path = None;
                let mut open_path = None;
                if ui.small_button("Save").on_hover_text("Ctrl + S").clicked() {
                    save_file_to(&mut save_path, ("Level", &["lvl"]));
                    if let Some(path) = save_path {
                        self.save(path);
                    }
                }
                if ui.small_button("Open").on_hover_text("Ctrl + O").clicked() {
                    pick_file_to(&mut open_path, ("Level", &["lvl"]));
                    if let Some(path) = open_path {
                        self.open(path);
                    }
                }
                ui.separator();
                if ui.small_button("Clear").clicked() {
                    self.show_clear_confirmation = true;
                }
                ui.separator();
                ui.radio_value(&mut self.current_mode, Mode::DrawBackground, "Background")
                    .on_hover_text("M");
                ui.radio_value(&mut self.current_mode, Mode::DrawForeground, "Foreground")
                    .on_hover_text("M");
                ui.radio_value(&mut self.current_mode, Mode::Collision, "Collision")
                    .on_hover_text("M");
                ui.radio_value(&mut self.current_mode, Mode::Entity, "Entity")
                    .on_hover_text("M");
                ui.separator();
                ui.label("View Filter");
                ui.checkbox(&mut self.show_foreground, "Foreground");
                ui.checkbox(&mut self.show_background, "Background");
                ui.checkbox(&mut self.show_collision, "Collision");
                ui.checkbox(&mut self.show_entity, "Entity");
            });
        });
    }

    fn side_panel_settings(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Settings", |ui| {
            ui.label("Sprite Size");
            ui.add(
                egui::DragValue::new(&mut self.spritesheet_info.sprite_size)
                    .clamp_range(1..=1024)
                    .suffix("px"),
            );
            ui.label("Num Rows");
            ui.add(egui::DragValue::new(&mut self.spritesheet_info.num_rows).clamp_range(1..=255));
            ui.label("Num Columns");
            ui.add(egui::DragValue::new(&mut self.spritesheet_info.num_cols).clamp_range(1..=255));
        });
    }

    fn side_panel_sprite_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if let Some(handle) = &self.spritesheet_handle {
                let handle_size = handle.size_vec2();
                for x in (0..handle_size.x as u32)
                    .step_by((handle_size.x / self.spritesheet_info.num_rows as f32) as usize)
                {
                    for y in (0..handle_size.y as u32)
                        .step_by((handle_size.y / self.spritesheet_info.num_cols as f32) as usize)
                    {
                        let normalized_x = x as f32 / handle_size.x;
                        let normalized_y = y as f32 / handle_size.y;
                        let max_x = (x as f32
                            + handle_size.x / self.spritesheet_info.num_rows as f32)
                            / handle_size.x;
                        let max_y = (y as f32
                            + handle_size.y / self.spritesheet_info.num_cols as f32)
                            / handle_size.y;
                        let uv = Rect {
                            min: Pos2 {
                                x: normalized_x,
                                y: normalized_y,
                            },
                            max: Pos2 { x: max_x, y: max_y },
                        };
                        let mut img_btn = egui::widgets::ImageButton::new(
                            handle,
                            Vec2 {
                                x: handle_size.x / self.spritesheet_info.num_rows as f32,
                                y: handle_size.y / self.spritesheet_info.num_cols as f32,
                            },
                        )
                        .uv(uv);
                        if self.selected_uv == Some(uv) {
                            img_btn = img_btn.selected(true);
                        }
                        if ui.add(img_btn).clicked() {
                            self.selected_uv = Some(uv);
                        };
                    }
                }
            }
        });
    }

    fn side_panel_spritesheet_preview(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        spritesheet_path: Option<PathBuf>,
    ) {
        if let Some(path) = spritesheet_path {
            let image_result = image::io::Reader::open(path).unwrap().decode();
            if image_result.is_err() {
                return;
            }
            let image = image_result.unwrap();
            let size = [image.width() as usize, image.height() as usize];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            self.spritesheet_handle =
                Some(ctx.load_texture("example-image", color_image, egui::TextureFilter::Nearest));
        }
        if let Some(handle) = &self.spritesheet_handle {
            ui.image(handle, handle.size_vec2());
        }
    }
    fn side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("my_right_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut spritesheet_path = None;
                if ui.button("Open Spritesheet").clicked() {
                    pick_file_to(
                        &mut spritesheet_path,
                        ("image", &["webp", "png", "bmp", "jpg", "jpeg"]),
                    );
                }
                self.side_panel_spritesheet_preview(ctx, ui, spritesheet_path);
                self.side_panel_settings(ui);
                ui.separator();
                self.side_panel_sprite_selector(ui);
            });
        });
    }
    fn handle_plot_clicks(
        &mut self,
        plot_ui: &egui::plot::PlotUi,
        primary_clicked: bool,
        secondary_clicked: bool,
        is_drag: bool,
    ) {
        if !(primary_clicked || secondary_clicked || is_drag) {
            return;
        }
        if let Some(coord) = plot_ui.pointer_coordinate() {
            let coord_x = coord.x.floor();
            let coord_y = coord.y.floor();
            let point = egui::widgets::plot::PlotPoint {
                x: coord_x,
                y: coord_y,
            };
            let plot_bounds = plot_ui.plot_bounds();
            let min = plot_bounds.min();
            let max = plot_bounds.max();
            let hashable_point = HashableVec2::from(point);
            if !(coord.x < min[0] || coord.x > max[0] || coord.y < min[1] || coord.y > max[1])
                && !self.show_clear_confirmation
                && !self.show_entity_popup
            // stop when pop ups are open
            {
                match self.current_mode {
                    Mode::DrawForeground | Mode::DrawBackground => {
                        self.handle_plot_fg_bg_clicks(
                            primary_clicked,
                            secondary_clicked,
                            is_drag,
                            hashable_point,
                        );
                    }
                    Mode::Collision => {
                        self.handle_plot_collision_clicks(primary_clicked, is_drag, hashable_point);
                    }
                    Mode::Entity => {
                        self.handle_plot_entity_clicks(
                            primary_clicked,
                            secondary_clicked,
                            is_drag,
                            hashable_point,
                        );
                    }
                }
            }
        }
    }
    fn handle_plot_fg_bg_clicks(
        &mut self,
        primary_clicked: bool,
        secondary_clicked: bool,
        is_drag: bool,
        hashable_point: HashableVec2,
    ) {
        if let Some(selected_uv) = self.selected_uv {
            let (layer_plotted_tiles, action) = match self.current_mode {
                Mode::DrawBackground => (
                    &mut self.background_plotted_tiles,
                    Action::ClickBackground(hashable_point, selected_uv),
                ),
                Mode::DrawForeground => (
                    &mut self.foreground_plotted_tiles,
                    Action::ClickForeground(hashable_point, selected_uv),
                ),
                _ => unreachable!(),
            };
            if primary_clicked || is_drag {
                if !is_drag {
                    if let None = layer_plotted_tiles.remove(&hashable_point) {
                        layer_plotted_tiles.insert(hashable_point, selected_uv);
                        self.undo_queue.push(action);
                        self.redo_queue.clear();
                    }
                } else {
                    if let None = layer_plotted_tiles.insert(hashable_point, selected_uv) {
                        self.undo_queue.push(action);
                        self.redo_queue.clear();
                    }
                }
            }
        }
        if secondary_clicked {
            match self.current_mode {
                Mode::DrawBackground => {
                    if let Some(uv) = self.background_plotted_tiles.get(&hashable_point) {
                        self.selected_uv = Some(*uv);
                    }
                }
                Mode::DrawForeground => {
                    if let Some(uv) = self.foreground_plotted_tiles.get(&hashable_point) {
                        self.selected_uv = Some(*uv);
                    }
                }
                _ => unreachable!(),
            };
        }
    }
    fn handle_plot_collision_clicks(
        &mut self,
        primary_clicked: bool,
        is_drag: bool,
        hashable_point: HashableVec2,
    ) {
        if primary_clicked || is_drag {
            if !is_drag {
                if !self.collision_tiles.remove(&hashable_point) {
                    self.collision_tiles.insert(hashable_point);
                    self.undo_queue.push(Action::ClickCollision(hashable_point));
                    self.redo_queue.clear();
                }
            } else {
                if self.collision_tiles.insert(hashable_point) {
                    self.undo_queue.push(Action::ClickCollision(hashable_point));
                    self.redo_queue.clear();
                }
            }
        }
    }
    fn handle_plot_entity_clicks(
        &mut self,
        primary_clicked: bool,
        secondary_clicked: bool,
        is_drag: bool,
        hashable_point: HashableVec2,
    ) {
        if primary_clicked || is_drag {
            if !is_drag {
                let mut action_label = None;
                if let Some(label) = self.entity_tiles.remove(&hashable_point) {
                    action_label = Some(label.clone());
                    self.entity_descriptions.remove(&label);
                } else {
                    self.entity_tiles.insert(hashable_point, "".to_owned());
                    self.show_entity_popup = true;
                    self.prev_entity_description = "".to_owned();
                    self.selected_entity = Some(hashable_point);
                }
                self.undo_queue.push(Action::ClickEntity(hashable_point, action_label));
                self.redo_queue.clear();
            }
        } else if secondary_clicked {
            if let Some(description) = self.entity_tiles.get(&hashable_point) {
                self.show_entity_popup = true;
                self.entity_description = description.clone();
                self.prev_entity_description = self.entity_description.clone();
                self.selected_entity = Some(hashable_point);
            }
        }
    }
    fn draw_sprites_on_plot(
        &self,
        plot_ui: &mut egui::plot::PlotUi,
        handle: &egui::TextureHandle,
        plotted_tiles: &HashMap<HashableVec2, Rect>,
    ) {
        let handle_size = handle.size_vec2();
        for (point, uv) in plotted_tiles {
            let final_coord = egui::widgets::plot::PlotPoint {
                x: point.x as f64 + 0.5,
                y: point.y as f64 + 0.5,
            };
            let img = egui::widgets::plot::PlotImage::new(
                handle,
                final_coord,
                Vec2 {
                    x: handle_size.x / self.spritesheet_info.num_rows as f32,
                    y: handle_size.y / self.spritesheet_info.num_cols as f32,
                } / self.spritesheet_info.sprite_size as f32,
            )
            .uv(*uv);
            plot_ui.image(img);
        }
    }
    fn draw_on_plot(&mut self, plot_ui: &mut egui::plot::PlotUi) {
        // if we want to draw sprites, we need a spritesheet
        if let Some(handle) = &self.spritesheet_handle {
            if self.show_background {
                self.draw_sprites_on_plot(plot_ui, handle, &self.background_plotted_tiles);
            }
            if self.show_foreground {
                self.draw_sprites_on_plot(plot_ui, handle, &self.foreground_plotted_tiles);
            }
        }
        // can draw these without spritesheet
        if self.show_collision {
            let collision_plot_points: Vec<[f64; 2]> = self
                .collision_tiles
                .iter()
                .map(|point| [point.x as f64 + 0.5, point.y as f64 + 0.5])
                .collect();
            let collision_points = egui::plot::Points::new(collision_plot_points)
                .filled(false)
                .radius(10.0)
                .shape(egui::plot::MarkerShape::Square)
                .color(egui::Color32::from_rgb(255, 0, 0));
            plot_ui.points(collision_points);
        }
        if self.show_entity {
            let entity_plot_points: Vec<[f64; 2]> = self
                .entity_tiles
                .iter()
                .map(|(point, _)| [point.x as f64 + 0.5, point.y as f64 + 0.5])
                .collect();
            let entity_points = egui::plot::Points::new(entity_plot_points)
                .filled(false)
                .radius(10.0)
                .shape(egui::plot::MarkerShape::Diamond)
                .color(egui::Color32::from_rgb(0, 255, 255));
            plot_ui.points(entity_points);
        }
    }
    fn plot_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::plot::Plot::new("level_plot")
                .data_aspect(1.0)
                .x_grid_spacer(egui::widgets::plot::uniform_grid_spacer(|_| {
                    [100.0, 25.0, 1.0]
                }))
                .y_grid_spacer(egui::widgets::plot::uniform_grid_spacer(|_| {
                    [100.0, 25.0, 1.0]
                }))
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .allow_double_click_reset(false)
                .show(ui, |plot_ui| {
                    let ctx = plot_ui.ctx();
                    //plot_ui.translate(Vec2 {x: 1.0, y: 0.0});
                    let mut primary_clicked = false;
                    let mut secondary_clicked = false;
                    for event in &ctx.input().events {
                        if let egui::Event::PointerButton {
                            button, pressed, ..
                        } = event
                        {
                            if *pressed {
                                match button {
                                    egui::PointerButton::Primary => primary_clicked = true,
                                    egui::PointerButton::Secondary => secondary_clicked = true,
                                    _ => (),
                                }
                            }
                        }
                    }
                    let drag_delta = plot_ui.pointer_coordinate_drag_delta();
                    let is_drag;

                    // attempting to stop user from clicking with a slight drag on mouse getting
                    // detected as a drag within a single square, thus performing 2 actions at once
                    // TODO a better alternative would be to add the delta with the position and see if
                    // it steps into another tile
                    if !(drag_delta.x > -0.05 && drag_delta.x < 0.05)
                        || !(drag_delta.y > -0.05 && drag_delta.y < 0.05)
                    {
                        is_drag = true;
                    } else {
                        is_drag = false;
                    }
                    self.handle_plot_clicks(plot_ui, primary_clicked, secondary_clicked, is_drag);
                    // Draw sprites, background then foreground
                    self.draw_on_plot(plot_ui);
                });
        });
    }
    fn handle_clear_confirmation(&mut self, ctx: &egui::Context) {
        if self.show_clear_confirmation {
            egui::Window::new("Are you sure you want to erase everything? This cannot be undone")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_clear_confirmation = false;
                        }
                        if ui.button("Ok").clicked() {
                            self.clear();
                        }
                    });
                });
        }
    }
    // TODO maybe pass the entity key in rather than getting it inside
    fn entity_description_is_ok(&self) -> (bool, HashableVec2) {
        if let Some(entity_key) = self.selected_entity {
            return (
                !self.entity_descriptions.contains(&self.entity_description)
                    && self.entity_description.len() != 0,
                entity_key,
            );
        }
        unreachable!(); // boy I hope so
    }
    fn do_entity_ok(&mut self, entity_key: HashableVec2) {
        if self.prev_entity_description != self.entity_description {
            self.entity_descriptions
                .remove(&self.prev_entity_description);
        }
        self.show_entity_popup = false;
        self.entity_tiles
            .insert(entity_key, self.entity_description.clone());
        self.entity_descriptions
            .insert(self.entity_description.clone());
        self.entity_description = "".to_string();
        self.redo_queue.clear(); // very unhappy with the state of undo/redo for entity editing but
    }
    fn handle_entity_popup(&mut self, ctx: &egui::Context) {
        if self.show_entity_popup {
            egui::Window::new("Entity Label Editor")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    let response = ui.add(egui::TextEdit::singleline(&mut self.entity_description));
                    self.entity_description = self.entity_description.trim().to_string();
                    if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                        let (description_is_ok, entity_key) = self.entity_description_is_ok();
                        if description_is_ok {
                            self.do_entity_ok(entity_key);
                        }
                    }
                    response.request_focus();
                    ui.label("Existing Entities");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for label in &self.entity_descriptions {
                                ui.label(label);
                            }
                        });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            let (description_is_ok, entity_key) = self.entity_description_is_ok();
                            if (!description_is_ok && !self.prev_entity_description.len() == 0)
                                || self.prev_entity_description.len() == 0
                            {
                                self.entity_tiles.remove(&entity_key);
                            }
                            self.show_entity_popup = false;
                            self.entity_description = "".to_string();
                        }
                        let (description_is_ok, entity_key) = self.entity_description_is_ok();
                        if description_is_ok {
                            if ui.button("Ok").clicked() {
                                self.do_entity_ok(entity_key);
                            }
                        } else {
                            ui.label("You must choose a unique label");
                        }
                    });
                });
        }
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
                Action::ClickForeground(point, uv) => {
                    if self.foreground_plotted_tiles.contains_key(&point) {
                        self.foreground_plotted_tiles.remove(&point);
                    } else {
                        self.foreground_plotted_tiles.insert(point, uv);
                    }
                },
                Action::ClickBackground(point, uv) => {
                    if self.background_plotted_tiles.contains_key(&point) {
                        self.background_plotted_tiles.remove(&point);
                    } else {
                        self.background_plotted_tiles.insert(point, uv);
                    }
                },
                Action::ClickCollision(point) => {
                    if self.collision_tiles.contains(&point) {
                        self.collision_tiles.remove(&point);
                    } else {
                        self.collision_tiles.insert(point);
                    }
                },
                Action::ClickEntity(point, attached_label) => {
                    if let Some(label) = self.entity_tiles.get(&point) {
                        let label_clone = label.clone();
                        self.entity_descriptions.remove(label);
                        self.entity_tiles.remove(&point);
                        cloned_action = Action::ClickEntity(point, Some(label_clone));
                    } else {
                        if let Some(label) = attached_label {
                            self.entity_tiles.insert(point, label.clone());
                            self.entity_descriptions.insert(label.clone());
                        }
                    }
                },
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
                        (egui::Key::Z, egui::Modifiers {ctrl, ..}) => {
                            if *ctrl {
                                self.handle_undo_redo(true);
                            }
                        },
                        (egui::Key::R, egui::Modifiers {ctrl, ..}) => {
                            if *ctrl {
                                self.handle_undo_redo(false);
                            }
                        },
                        _ => (),
                    };
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_clear_confirmation(ctx);
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

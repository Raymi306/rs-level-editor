use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::{HashableVec2, MyApp};
use eframe::egui::{Pos2, Rect};

pub fn pick_file_to(var: &mut Option<PathBuf>, filter: (&str, &[&str])) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(filter.0, filter.1)
        .pick_file()
    {
        *var = Some(path);
    }
}

pub fn save_file_to(var: &mut Option<PathBuf>, filter: (&str, &[&str])) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(filter.0, filter.1)
        .save_file()
    {
        *var = Some(path);
    }
}

impl MyApp {
    pub(crate) fn save(&mut self, path: PathBuf) {
        let display = path.display();
        let mut file = match File::create(&path) {
            Err(why) => {
                println!("Couldn't create {}: {}", display, why);
                return;
            }
            Ok(f) => f,
        };
        if self.background_plotted_tiles.is_empty() || self.spritesheet_handle.is_none() {
            return;
        }
        let handle = self.spritesheet_handle.as_ref().unwrap();
        let handle_size = handle.size_vec2();
        let len_bg = (self.background_plotted_tiles.len() * 20) as u64;
        let len_fg = (self.foreground_plotted_tiles.len() * 20) as u64;
        let len_collision = (self.collision_tiles.len() * 16) as u64;
        let len_entity = (self.entity_tiles.len() * 64) as u64; // we don't know label length
        let mut buffer: Vec<u8> =
            Vec::with_capacity((len_bg + len_fg + len_collision + len_entity) as usize);
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

    pub(crate) fn open(&mut self, path: PathBuf) {
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
            let uv_max_x = (row as f32 + handle_size.x / self.spritesheet_info.num_rows as f32)
                / handle_size.x;
            let uv_max_y = (col as f32 + handle_size.y / self.spritesheet_info.num_cols as f32)
                / handle_size.y;
            let uv = Rect {
                min: Pos2 {
                    x: uv_min_x,
                    y: uv_min_y,
                },
                max: Pos2 {
                    x: uv_max_x,
                    y: uv_max_y,
                },
            };
            self.background_plotted_tiles
                .insert(HashableVec2 { x, y }, uv);
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
            let uv_max_x = (row as f32 + handle_size.x / self.spritesheet_info.num_rows as f32)
                / handle_size.x;
            let uv_max_y = (col as f32 + handle_size.y / self.spritesheet_info.num_cols as f32)
                / handle_size.y;
            let uv = Rect {
                min: Pos2 {
                    x: uv_min_x,
                    y: uv_min_y,
                },
                max: Pos2 {
                    x: uv_max_x,
                    y: uv_max_y,
                },
            };
            self.foreground_plotted_tiles
                .insert(HashableVec2 { x, y }, uv);
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
            let label =
                String::from_utf8(buf[index..index + label_len as usize].try_into().unwrap())
                    .unwrap();
            self.entity_tiles.insert(HashableVec2 { x, y }, label);
            index += label_len as usize;
        }
    }
}

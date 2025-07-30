use std::io;

use crate::app::{
    model::{
        file::get_current_file,
        miller::positions::{get_position, update_dir_position},
    },
    state::{Mode, State},
};

pub trait Navigation {
    fn navigate_to_child(&mut self) -> io::Result<()>;
    fn navigate_to_parent(&mut self) -> io::Result<()>;
    fn navigate_up(&mut self) -> io::Result<()>;
    fn navigate_down(&mut self) -> io::Result<()>;
}

impl<'a> Navigation for State<'a> {
    fn navigate_to_parent(&mut self) -> io::Result<()> {
        if let Some(parent) = &self.dirs[0].dir_name {
            self.current_dir = parent.to_path_buf();
            let position_id = get_position(&self.positions_map, &self.current_dir);
            let _ = self.reset_state(position_id);
        }
        Ok(())
    }

    fn navigate_to_child(&mut self) -> io::Result<()> {
        if let Some(child) = &self.dirs[2].dir_name {
            self.current_dir = child.to_path_buf();
            let position_id = get_position(&self.positions_map, &self.current_dir);
            let _ = self.reset_state(position_id);
        }
        Ok(())
    }

    fn navigate_up(&mut self) -> io::Result<()> {
        let position_id = get_position(&self.positions_map, &self.current_dir);
        let new_position_id = position_id.saturating_sub(1);

        if self.mode == Mode::Visual {
            let current_file =
                get_current_file(&self.positions_map, &self.current_dir, &self.files[1]);
            if let Some(file) = current_file {
                let found_files = &self.selected.iter().find(|e| e.to_string() == file.name);
                if let Some(file) = found_files {
                    self.selected.push(current_file);
                }
            }
        }

        update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
        let _ = self.reset_state(new_position_id);
        Ok(())
    }

    fn navigate_down(&mut self) -> io::Result<()> {
        let position_id = get_position(&self.positions_map, &self.current_dir);
        if position_id < self.files[1].len().saturating_sub(1) {
            let new_position_id = position_id + 1;
            update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
            let _ = self.reset_state(new_position_id);
        }
        Ok(())
    }
}

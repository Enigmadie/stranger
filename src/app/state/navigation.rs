use std::io;

use crate::app::{
    model::miller::positions::{get_position, update_dir_position},
    state::State,
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
        if position_id > 0 {
            let new_position_id = position_id.saturating_sub(1);

            update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
            self.mark_item();
            let _ = self.reset_state(new_position_id);
        }
        Ok(())
    }

    fn navigate_down(&mut self) -> io::Result<()> {
        let position_id = get_position(&self.positions_map, &self.current_dir);
        if position_id < self.files[1].len().saturating_sub(1) {
            let new_position_id = position_id + 1;

            update_dir_position(&mut self.positions_map, &self.current_dir, new_position_id);
            self.mark_item();
            let _ = self.reset_state(new_position_id);
        }
        Ok(())
    }
}

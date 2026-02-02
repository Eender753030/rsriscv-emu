#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editting
}

impl InputMode {
    pub fn edit(&mut self) {
        *self = InputMode::Editting;
    }

    pub fn normal(&mut self) {
        *self = InputMode::Normal;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmuInput {
    pub mode: InputMode,
    pub chars: String,
    pub cursor: usize,
}

impl EmuInput {
    pub fn enter_char(&mut self, new_char: char) {
        if self.chars.len() < 10 {
            let idx = self.byte_index();
            self.chars.insert(idx, new_char);
            self.move_cursor_right();
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved = self.cursor.saturating_sub(1);
        self.cursor = self.clamp_cursor(cursor_moved);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved = self.cursor.saturating_add(1);
        self.cursor = self.clamp_cursor(cursor_moved);
    }

    pub fn delete_char(&mut self) {
        if self.cursor != 2 {
            let target = self.cursor;
            let left_to_target = target - 1;


            let before_delete = self.chars.chars().take(left_to_target);
            let after_delete = self.chars.chars().skip(target);

            self.chars = before_delete.chain(after_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn submit(&mut self) -> Option<u32> {
        let res = u32::from_str_radix(&self.chars[2..], 16).ok();
        self.clear();
        res
    } 

    pub fn clear(&mut self) {
        self.chars = "0x".to_string();
        self.cursor = 2;
    }

    fn byte_index(&self) -> usize {
        self.chars
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor)
            .unwrap_or(self.chars.len())    
    }

    fn clamp_cursor(&self, cursor_pos: usize) -> usize {
        cursor_pos.clamp(2, self.chars.chars().count())
    }
}

impl Default for EmuInput {
    fn default() -> Self {
        let mode = InputMode::default();
        let chars = "0x".to_string();

        EmuInput { mode, chars, cursor: 2 } 
    }
}
use ratatui::widgets::ListState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListStateRecord<T> {
    pub list: Vec<T>,
    pub list_state: ListState,
    pub current_select: usize,
}

impl <T> ListStateRecord<T> {
    pub fn new(list: Vec<T>) -> Self {
        ListStateRecord { 
            list,
            ..Default::default()
        }
    }

    pub fn select(&mut self, select: usize) {
        self.list_state.select(Some(select));
    }

    pub fn select_curr(&mut self) {
        self.list_state.select(Some(self.current_select));
    }

    pub fn no_select(&mut self) {
        self.list_state.select(None);
    }

    pub fn next(&mut self) {
        self.current_select = match self.current_select >= self.list.len() - 1 {
            true => 0,
            false => self.current_select + 1
        };
        self.select_curr();
    }

    pub fn prev(&mut self) {
        self.current_select = match self.current_select == 0 {
            true => self.list.len() - 1,
            false => self.current_select - 1
        };
        self.select_curr();
    }
}

impl<T> Default for ListStateRecord<T> {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        ListStateRecord {
            list: Vec::new(),
            list_state, 
            current_select: 0, 
        }
    }
}
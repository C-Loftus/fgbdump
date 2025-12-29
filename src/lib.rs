use ratatui::widgets::TableState;

pub struct ColumnsTableState {
    pub state: TableState,
}

impl ColumnsTableState {
    pub fn new() -> Self {
        Self {
            state: TableState::default().with_selected(Some(0)),
        }
    }

    pub fn next(&mut self, len: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self, len: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
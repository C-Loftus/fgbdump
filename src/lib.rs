use ratatui::{
    style::Color,
    widgets::{
        Block, Borders, TableState, Widget,
        canvas::{Canvas, Map, MapResolution},
    },
};

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

pub fn map_with_bbox_overlay(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> impl Widget {
    Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Extent of Data"),
        )
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
        .paint(move |ctx| {
            ctx.draw(&Map {
                color: Color::Red,
                resolution: MapResolution::High,
            });
            ctx.draw(&ratatui::widgets::canvas::Rectangle {
                x: xmin,
                y: ymin,
                width: xmax - xmin,
                height: ymax - ymin,
                color: Color::Green,
            });
        })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectedTab {
    Metadata,
    Columns,
    Map,
}

impl SelectedTab {
    pub fn next(self) -> Self {
        match self {
            Self::Metadata => Self::Columns,
            Self::Columns => Self::Map,
            Self::Map => Self::Metadata,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Metadata => Self::Map,
            Self::Columns => Self::Metadata,
            Self::Map => Self::Columns,
        }
    }

    pub fn titles() -> Vec<&'static str> {
        vec!["Metadata", "Columns", "Map"]
    }
}

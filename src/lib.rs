pub mod cli;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, TableState, Tabs, Widget,
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
    const MAX_LONGITUDE_RANGE: [f64; 2] = [-180.0, 180.0];
    const MAX_LATITUDE_RANGE: [f64; 2] = [-90.0, 90.0];
    Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Extent of Data in EPSG:4326"),
        )
        .x_bounds(MAX_LONGITUDE_RANGE)
        .y_bounds(MAX_LATITUDE_RANGE)
        .paint(move |ctx| {
            // draw section that isn't included in the dataset
            ctx.draw(&Map {
                color: Color::Red,
                resolution: MapResolution::High,
            });
            // make all the section that contains the dataset
            // enveloped in green to show it is included
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

pub fn make_tabs(selected_tab: SelectedTab) -> impl Widget {
    let tabs_titles = SelectedTab::titles();
    Tabs::new(tabs_titles)
        .select(selected_tab as usize)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Header Categories"),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )
}

pub fn info_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(value.to_string()),
    ])
}

pub struct Column<'a, T> {
    pub header: &'a str,
    pub value: Box<dyn Fn(&T) -> String + 'a>,
}

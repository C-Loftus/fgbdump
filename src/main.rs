//! FlatGeobuf header viewer with tabs and bounding box map
//!
//! This example uses ratatui 0.3.x with tabs and a Canvas map tab for FlatGeobuf bounding box visualization.

use std::io::stdout;

use argh::FromArgs;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use flatgeobuf::HttpFgbReader;
use ratatui::layout::Constraint;
use ratatui::widgets::{Cell, Row, Table};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::TableState,
    widgets::{
        Block, Borders, Paragraph, Tabs,
        canvas::{Canvas, Map, MapResolution},
    },
};

#[derive(FromArgs, Debug)]
/// Print info about a FlatGeobuf file
struct TopLevel {
    #[argh(subcommand)]
    cmd: Command,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Command {
    Header(Header),
    Query(Query),
}

#[derive(FromArgs, Debug)]
/// Display info about the FlatGeobuf header
#[argh(subcommand, name = "header")]
struct Header {
    #[argh(option, description = "path or URL to the FlatGeobuf file")]
    file: String,

    #[argh(switch, description = "print to stdout instead of the TUI")]
    stdout: bool,
}

#[derive(FromArgs, Debug)]
/// Query by a bounding box
#[argh(subcommand, name = "query")]
struct Query {
    #[argh(option, description = "path or URL to the FlatGeobuf file")]
    file: String,

    #[argh(option, description = "bounding box as xmin,ymin,xmax,ymax")]
    bbox: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: TopLevel = argh::from_env();

    match args.cmd {
        Command::Header(cmd) => {
            let fgb = HttpFgbReader::open(&cmd.file).await?;
            let header = fgb.header();
            if cmd.stdout {
                println!("{:#?}", header);
                return Ok(());
            }
            render_header_tui(&header)?;
        }
        Command::Query(cmd) => {
            let _fgb = HttpFgbReader::open(&cmd.file).await?;
            println!("BBox query: {}", cmd.bbox);
        }
    }

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectedTab {
    Metadata,
    Columns,
    Map,
}

impl SelectedTab {
    fn next(self) -> Self {
        match self {
            Self::Metadata => Self::Columns,
            Self::Columns => Self::Map,
            Self::Map => Self::Metadata,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Metadata => Self::Map,
            Self::Columns => Self::Metadata,
            Self::Map => Self::Columns,
        }
    }

    fn titles() -> Vec<&'static str> {
        vec!["Metadata", "Columns", "Map"]
    }
}

struct ColumnsTableState {
    state: TableState,
}

impl ColumnsTableState {
    fn new() -> Self {
        Self {
            state: TableState::default().with_selected(Some(0)),
        }
    }

    fn next(&mut self, len: usize) {
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

    fn previous(&mut self, len: usize) {
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

fn render_header_tui(header: &flatgeobuf::Header) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut selected_tab = SelectedTab::Metadata;

    // Extract bbox as Vec<f64>
    let bbox: [f64; 4] = header
        .envelope()
        .map(|v| [v.get(0), v.get(1), v.get(2), v.get(3)])
        .unwrap_or([0.0, 0.0, 0.0, 0.0]);

    let mut columns_table_state = ColumnsTableState::new();

    loop {
        terminal.draw(|f| {
            let size = f.area();

            // Render tabs
            let tabs_titles = SelectedTab::titles();
            let tabs = Tabs::new(tabs_titles)
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
                        .add_modifier(Modifier::BOLD),
                );
            f.render_widget(
                tabs,
                Rect {
                    x: 0,
                    y: 0,
                    width: size.width,
                    height: 3,
                },
            );

            // Content area
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(size);
            let content_area = chunks[1];

            match selected_tab {
                SelectedTab::Metadata => {
                    let column_count = header.columns().map(|c| c.len()).unwrap_or(0);
                    let crs = match header.crs() {
                        Some(crs) => format!("{}:{}", crs.org().unwrap_or("UNDEFINED_CRS_ORG"), crs.code()),
                        None => "Undefined".to_string(),
                    };
                    let envelope = header
                        .envelope()
                        .map_or("Undefined".to_string(), |e| format!("{:?}", e));

                    
                    let index_node_size = match header.index_node_size() {
                        0 => "No Spatial Index".to_string(),
                        _ => format!("{}", header.index_node_size()),
                    };

                    let body = Paragraph::new(vec![
                        info_line("Name", header.name().unwrap_or("")),
                        // Not clear if anything uses the title, commenting it out
                        // info_line("Title", header.title().unwrap_or("")),
                        info_line("Description", header.description().unwrap_or("")),
                        info_line("Features", &header.features_count().to_string()),
                        info_line("Bounds", &envelope),
                        info_line("Geometry Type", &format!("{:?}", header.geometry_type())),
                        info_line("Columns", &column_count.to_string()),
                        info_line("CRS", &crs),
                        info_line("Metadata", &format!("{:?}", header.metadata())),
                        info_line("Spatial Index R-Tree Node Size", &index_node_size),
                        info_line("Has M Dimension", &header.has_m().to_string()),
                        info_line("Has Z Dimension", &header.has_z().to_string()),
                        info_line("Has T Dimension", &header.has_t().to_string()),
                        info_line("Has TM Dimension", &header.has_tm().to_string()),
                    ])
                    .block(Block::default().borders(Borders::ALL).title("Metadata"));

                    f.render_widget(body, content_area);
                }
                SelectedTab::Columns => {
                    let columns_data = header.columns().unwrap_or_default();

                    let header_cells = [
                        "Name",
                        "Type",
                        "Description",
                        "Nullable",
                        "Primary Key",
                        "Unique",
                        "Precision",
                        "Scale",
                        "Width",
                    ]
                    .iter()
                    .map(|h| Cell::from(*h))
                    .collect::<Vec<_>>();
                    let table_header = Row::new(header_cells).height(1);

                    let rows = columns_data.iter().map(|c| {
                        let cells = vec![
                            Cell::from(c.name()),
                            Cell::from(format!("{:?}", c.type_())),
                            Cell::from(c.description().unwrap_or("—")),
                            Cell::from(c.nullable().to_string()),
                            Cell::from(c.primary_key().to_string()),
                            Cell::from(c.unique().to_string()),
                            Cell::from(c.precision().to_string()),
                            Cell::from(c.scale().to_string()),
                            Cell::from(c.width().to_string()),
                        ];
                        Row::new(cells).height(1)
                    });

                    let widths = &[
                        Constraint::Length(20),
                        Constraint::Length(10),
                        Constraint::Length(25),
                        Constraint::Length(10),
                        Constraint::Length(12),
                        Constraint::Length(8),
                        Constraint::Length(10),
                        Constraint::Length(8),
                        Constraint::Length(8),
                    ];

                    let table = Table::new(rows, widths)
                        .header(table_header)
                        .block(Block::default().borders(Borders::ALL).title("Columns"))
                        .highlight_style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol(">> ");

                    f.render_stateful_widget(table, content_area, &mut columns_table_state.state);
                }
                SelectedTab::Map => {
                    let xmin = bbox[0];
                    let ymin = bbox[1];
                    let xmax = bbox[2];
                    let ymax = bbox[3];

                    let canvas = Canvas::default()
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Bounding Box Map"),
                        )
                        .x_bounds([-180.0, 180.0])
                        .y_bounds([-90.0, 90.0])
                        .paint(|ctx: &mut ratatui::widgets::canvas::Context<'_>| {
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
                        });

                    f.render_widget(canvas, content_area);
                }
            }
        })?;

        // Event handling
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        }) = event::read()?
        {
            match code {
                KeyCode::Right => selected_tab = selected_tab.next(),
                KeyCode::Left => selected_tab = selected_tab.previous(),
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break,
                KeyCode::Char('c') => {
                    if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        break;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected_tab == SelectedTab::Columns {
                        columns_table_state.next(header.columns().unwrap_or_default().len());
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected_tab == SelectedTab::Columns {
                        columns_table_state.previous(header.columns().unwrap_or_default().len());
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn info_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(value.to_string()),
    ])
}

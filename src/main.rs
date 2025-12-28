//! FlatGeobuf header viewer with tabs and bounding box map
//!
//! This example uses ratatui 0.3.x with tabs and a Canvas map tab for FlatGeobuf bounding box visualization.

use std::io::stdout;

use argh::FromArgs;
use crossterm::{
    execute,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use flatgeobuf::HttpFgbReader;
use ratatui::{
    Frame, Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Tabs, Widget, canvas::{Canvas, Map, MapResolution}}
};
use flatbuffers::Vector;

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

// ----------------- TUI -----------------

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

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Render tabs
            let tabs_titles = SelectedTab::titles();
            let tabs = Tabs::new(tabs_titles)
                .select(selected_tab as usize)
                .block(Block::default().borders(Borders::ALL).title("Header Categories"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
            f.render_widget(tabs, Rect { x: 0, y: 0, width: size.width, height: 3 });

            // Content area
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(size);
            let content_area = chunks[1];

            match selected_tab {
                SelectedTab::Metadata => {
                    let column_count = header.columns().map(|c| c.len()).unwrap_or(0);
                    let crs = header.crs().map_or("Undefined".to_string(), |c| format!("{:?}", c));
                    let envelope = header.envelope().map_or("Undefined".to_string(), |e| format!("{:?}", e));

                    let body = Paragraph::new(vec![
                        info_line("Name", header.name().unwrap_or("—")),
                        info_line("Features", &header.features_count().to_string()),
                        info_line("Bounds", &envelope),
                        info_line("Geometry Type", &format!("{:?}", header.geometry_type())),
                        info_line("Columns", &column_count.to_string()),
                        info_line("CRS", &crs),
                    ])
                    .block(Block::default().borders(Borders::ALL).title("Metadata"));

                    f.render_widget(body, content_area);
                }
                SelectedTab::Columns => {
                    let lines: Vec<Line> = header
                        .columns()
                        .unwrap_or_default()
                        .iter()
                        .map(|c| info_line(c.name(), &format!("{:?}", c.type_())))
                        .collect();

                    let body = Paragraph::new(lines)
                        .block(Block::default().borders(Borders::ALL).title("Columns"));
                    f.render_widget(body, content_area);
                }
                SelectedTab::Map => {
                    let xmin = bbox[0];
                    let ymin = bbox[1];
                    let xmax = bbox[2];
                    let ymax = bbox[3];

                    let canvas = Canvas::default()
                        .block(Block::default().borders(Borders::ALL).title("Bounding Box Map"))
                        .x_bounds([xmin - 1.0, xmax + 1.0])
                        .y_bounds([ymin - 1.0, ymax + 1.0])
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
        if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) = event::read()? {
            match code {
                KeyCode::Right => selected_tab = selected_tab.next(),
                KeyCode::Left => selected_tab = selected_tab.previous(),
                KeyCode::Esc | KeyCode::Char('q') => break,
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
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::raw(value.to_string()),
    ])
}

use argh::FromArgs;
use flatgeobuf::HttpFgbReader;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io::stdout;

#[derive(FromArgs, Debug)]
/// Print info about a FlatGeobuf file; Created by Colton Loftus
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
    #[argh(option)]
    /// the path or URL to the FlatGeobuf file
    file: String,

    #[argh(switch)]
    /// print to stdout instead of the TUI
    stdout: bool,
}

#[derive(FromArgs, Debug)]
/// Query by a bounding box
#[argh(subcommand, name = "query")]
struct Query {
    #[argh(option)]
    /// the path or URL to the FlatGeobuf file
    file: String,

    #[argh(option)]
    /// the bounding box "xmin,ymin,xmax,ymax"
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
enum Tab {
    Metadata,
    Columns,
}

fn render_header_tui(header: &flatgeobuf::Header) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut active_tab = Tab::Metadata;
    let tabs: [Tab; 2] = [Tab::Metadata, Tab::Columns];

    loop {
        terminal.draw(|f| {
            let area = f.area();

            // Split area: top row for tabs, rest for content
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area);

            // Render tabs as a Paragraph
            let tab_spans: Vec<Span> = tabs
                .iter()
                .enumerate()
                .map(|(_, t)| {
                    let text = match t {
                        Tab::Metadata => " Metadata ",
                        Tab::Columns => " Columns ",
                    };
                    if *t == active_tab {
                        Span::styled(
                            text,
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        )
                        
                    } else {
                        Span::raw(text)
                    }
                })
                .collect();

            let tabs_paragraph = Paragraph::new(Line::from(tab_spans))
                .block(Block::default().borders(Borders::BOTTOM));

            f.render_widget(tabs_paragraph, layout[0]);

            // Render tab content
            match active_tab {
                Tab::Metadata => {
                    let column_count = header.columns().map(|c| c.len()).unwrap_or(0);
                    let crs = header.crs().map_or("Undefined".to_string(), |c| format!("{:?}", c));
                    let envelope = header.envelope().map_or("Undefined".to_string(), |e| format!("{:?}", e));

                    let body = Paragraph::new(vec![
                        info_line("Name", header.name().unwrap_or("—")),
                        info_line("Features", &header.features_count().to_string()),
                        info_line("Bounds", &envelope),
                        info_line("Geometry Type", &format!("{:?}", header.geometry_type())),
                        info_line("Columns", &column_count.to_string()),
                        Line::from(""),
                        Line::from(vec![
                            Span::styled("Index Node Size ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                            Span::raw(header.index_node_size().to_string()),
                        ]),
                        info_line("CRS", &crs),
                    ])
                    .wrap(Wrap { trim: true })
                    .block(Block::default().borders(Borders::ALL).title("Metadata"));

                    f.render_widget(body, layout[1]);
                }
                Tab::Columns => {
                    let column_lines: Vec<Line> = header
                        .columns()
                        .unwrap_or_default()
                        .iter()
                        .map(|c| info_line(&c.name(), ""))
                        .collect();

                    let body = Paragraph::new(column_lines)
                        .wrap(Wrap { trim: true })
                        .block(Block::default().borders(Borders::ALL).title("Columns"));

                    f.render_widget(body, layout[1]);
                }
            }
        })?;

        // Handle key events
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            match code {
                KeyCode::Right => active_tab = Tab::Columns,
                KeyCode::Left => active_tab = Tab::Metadata,
                KeyCode::Esc | KeyCode::Backspace => break,
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => break,
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

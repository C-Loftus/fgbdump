//! FlatGeobuf header viewer with tabs and bounding box map
//!
//! This example uses ratatui 0.3.x with tabs and a Canvas map tab for FlatGeobuf bounding box visualization.

use std::io::stdout;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fgbdump::{
    Column, ColumnsTableState, SelectedTab,
    cli::{Command, TopLevel},
    info_line, make_tabs, map_with_bbox_overlay,
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
    widgets::{Block, Borders, Paragraph, Tabs},
};

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

fn render_header_tui(header: &flatgeobuf::Header) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

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

            let tabs = make_tabs(selected_tab);
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
                    let envelope = header
                        .envelope()
                        .map_or("Undefined".to_string(), |e| format!("{:?}", e));

                    let index_node_size = match header.index_node_size() {
                        0 => "No Spatial Index".to_string(),
                        _ => format!("{}", header.index_node_size()),
                    };

                    let mut lines = vec![
                        info_line("Name", header.name().unwrap_or("")),
                        // Not clear if anything uses the title, commenting it out
                        // info_line("Title", header.title().unwrap_or("")),
                        info_line("Description", header.description().unwrap_or("")),
                        info_line("Features", &header.features_count().to_string()),
                        info_line("Bounds", &envelope),
                        info_line("Geometry Type", &format!("{:?}", header.geometry_type())),
                        info_line("Columns", &column_count.to_string()),
                        info_line("Spatial Index R-Tree Node Size", &index_node_size),
                    ];

                    for line in [
                        Line::default(),
                        info_line("Has M Dimension", &header.has_m().to_string()),
                        info_line("Has Z Dimension", &header.has_z().to_string()),
                        info_line("Has T Dimension", &header.has_t().to_string()),
                        info_line("Has TM Dimension", &header.has_tm().to_string()),
                    ] {
                        lines.push(line);
                    }

                    let crs = header.crs();

                    if let Some(crs) = crs {
                        // separator
                        lines.push(Line::default());
                        // code; name; code string; description; org; wkt
                        lines.push(info_line("CRS Code", &crs.code().to_string()));
                        lines.push(info_line("CRS Name", &crs.name().unwrap_or_default()));
                        lines.push(info_line(
                            "CRS Code String",
                            &crs.code_string().unwrap_or_default(),
                        ));
                        lines.push(info_line(
                            "CRS Description",
                            &crs.description().unwrap_or_default(),
                        ));
                        lines.push(info_line("CRS Authority", &crs.org().unwrap_or_default()));
                        lines.push(info_line("CRS WKT", &crs.wkt().unwrap_or_default()));
                    } else {
                        lines.push(info_line("CRS", "Undefined"));
                    }

                    // separator
                    lines.push(Line::default());
                    lines.push(info_line(
                        "Custom Metadata",
                        &format!("{:?}", header.metadata()),
                    ));

                    let body = Paragraph::new(lines)
                        .block(Block::default().borders(Borders::ALL).title("Metadata"));

                    f.render_widget(body, content_area);
                }
                SelectedTab::Columns => {
                    let columns_data = header.columns().unwrap_or_default();

                    // Declare table columns in one place
                    let columns: Vec<Column<_>> = vec![
                        Column {
                            header: "Name",
                            value: Box::new(|c: &flatgeobuf::Column| c.name().to_string()),
                        },
                        Column {
                            header: "Type",
                            value: Box::new(|c| format!("{:?}", c.type_())),
                        },
                        Column {
                            header: "Description",
                            value: Box::new(|c| c.description().unwrap_or("—").to_string()),
                        },
                        Column {
                            header: "Nullable",
                            value: Box::new(|c| c.nullable().to_string()),
                        },
                        Column {
                            header: "Primary Key",
                            value: Box::new(|c| c.primary_key().to_string()),
                        },
                        Column {
                            header: "Unique",
                            value: Box::new(|c| c.unique().to_string()),
                        },
                    ];

                    // Build table header
                    let header_cells = columns
                        .iter()
                        .map(|col| Cell::from(col.header))
                        .collect::<Vec<_>>();

                    let table_header = Row::new(header_cells).height(1);

                    // Build table rows
                    let rows = columns_data.iter().map(|c| {
                        let cells = columns
                            .iter()
                            .map(|col| Cell::from((col.value)(&c)))
                            .collect::<Vec<_>>();

                        Row::new(cells).height(1)
                    });

                    // Compute column widths based on max(header, content)
                    let widths = columns
                        .iter()
                        .enumerate()
                        .map(|(i, col)| {
                            let max_content_len = columns_data
                                .iter()
                                .map(|c| (col.value)(&c).len())
                                .max()
                                .unwrap_or(0);

                            let width = col.header.len().max(max_content_len) as u16 + 2;
                            Constraint::Length(width)
                        })
                        .collect::<Vec<_>>();

                    // Build table
                    let table = Table::new(rows, &widths)
                        .header(table_header)
                        .block(Block::default().borders(Borders::ALL).title("Columns"))
                        .row_highlight_style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol(">> ");

                    // Render
                    f.render_stateful_widget(table, content_area, &mut columns_table_state.state);
                }
                SelectedTab::Map => {
                    let xmin = bbox[0];
                    let ymin = bbox[1];
                    let xmax = bbox[2];
                    let ymax = bbox[3];

                    let canvas = map_with_bbox_overlay(xmin, ymin, xmax, ymax);
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

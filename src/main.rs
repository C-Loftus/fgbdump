// Copyright 2025 Colton Loftus
// SPDX-License-Identifier: Apache-2.0

use bytesize::ByteSize;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fgbdump::{
    Column, ColumnsTableState, SelectedTab, cli::Args, info_line, is_remote_file,
    make_map_with_bbox_overlay, make_tabs, projection::Bbox,
};
use flatgeobuf::{FgbReader, HttpFgbReader};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::scrollbar,
    text::Line,
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table,
    },
};
use reqwest::header::CONTENT_LENGTH;
use std::{
    fs::File,
    io::{BufReader, IsTerminal, stdout},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();

    if !args.stdout && !stdout().is_terminal() {
        eprintln!(
            "To render the TUI, this program must be run in a terminal. To dump text to stdout, use the --stdout flag."
        );
        std::process::exit(1);
    }

    if is_remote_file(&args.file) {
        // Remote file: use HTTP HEAD to get content length
        let client = reqwest::Client::new();
        let resp = client.head(&args.file).send().await?;
        let content_length = resp
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        let fgb = HttpFgbReader::open(&args.file).await?;
        let header = fgb.header();
        if args.stdout {
            println!("{:#?}", header);
            return Ok(());
        }
        render_header_tui(&header, content_length)?;
    } else {
        // Local file: use metadata to get size
        let metadata = std::fs::metadata(&args.file)?;
        let file_size = metadata.len();

        let mut filein = BufReader::new(File::open(&args.file)?);
        let fgb = FgbReader::open(&mut filein)?;
        let header = fgb.header();
        if args.stdout {
            println!("{:#?}", header);
            return Ok(());
        }
        render_header_tui(&header, Some(file_size))?;
    }

    Ok(())
}

fn render_header_tui(
    header: &flatgeobuf::Header,
    byte_size: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut selected_tab = SelectedTab::Metadata;

    // Scroll state for Metadata tab
    let mut metadata_scroll: usize = 0;
    let mut metadata_scroll_state = ScrollbarState::default();

    // Extract bbox
    let envelope = header
        .envelope()
        .ok_or("No bbox extent envelope founbd in the flatgeobuf metadata")?;
    let bbox = Bbox::from_flatgeobuf_envelope(&envelope)?;

    let crs = header
        .crs()
        .ok_or("No crs found in the flatgeobuf metadata; no map can be rendered".to_string())?;

    let src_proj_crs_string = format!(
        "{}:{}",
        crs.org()
            .ok_or("No crs org found in the flatgeobuf metadata; no map can be rendered ")?,
        crs.code()
    );

    let mut columns_table_state = ColumnsTableState::new();
    let mut columns_scroll_state = ScrollbarState::default();

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

                    let byte_size_str = byte_size.map(|s| ByteSize(s).to_string());
                    let mut lines = vec![
                        info_line("Name", header.name().unwrap_or("")),
                        info_line("File Size", byte_size_str.as_deref().unwrap_or("Unknown")),
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

                    if let Some(crs) = header.crs() {
                        lines.push(Line::default());
                        lines.push(info_line("CRS Code", &crs.code().to_string()));
                        lines.push(info_line("CRS Name", crs.name().unwrap_or_default()));
                        lines.push(info_line(
                            "CRS Code String",
                            crs.code_string().unwrap_or_default(),
                        ));
                        lines.push(info_line(
                            "CRS Description",
                            crs.description().unwrap_or_default(),
                        ));
                        lines.push(info_line("CRS Organization", crs.org().unwrap_or_default()));
                        lines.push(info_line("CRS WKT", crs.wkt().unwrap_or_default()));
                    } else {
                        lines.push(info_line("CRS", "Undefined"));
                    }

                    lines.push(Line::default());
                    lines.push(info_line(
                        "Custom Metadata",
                        &format!("{:?}", header.metadata()),
                    ));

                    let max_scroll = lines.len();

                    metadata_scroll = metadata_scroll.min(max_scroll);
                    metadata_scroll_state = metadata_scroll_state
                        .content_length(max_scroll + 1)
                        .position(metadata_scroll);

                    let body = Paragraph::new(lines)
                        .wrap(ratatui::widgets::Wrap { trim: true }) // enable wrapping
                        .scroll((metadata_scroll as u16, 0))
                        .block(Block::default().borders(Borders::ALL).title("Metadata"));

                    f.render_widget(body, content_area);

                    f.render_stateful_widget(
                        Scrollbar::new(ScrollbarOrientation::VerticalRight)
                            .symbols(scrollbar::VERTICAL)
                            .begin_symbol(Some("↑"))
                            .end_symbol(Some("↓")),
                        content_area,
                        &mut metadata_scroll_state,
                    );
                }

                SelectedTab::Columns => {
                    let columns_data = header.columns().unwrap_or_default();

                    let total_rows = columns_data.len();

                    const TABLE_CHROME_ROWS: u16 = 3; // top border + header + bottom border
                    let visible_rows =
                        content_area.height.saturating_sub(TABLE_CHROME_ROWS) as usize;

                    let max_scroll = total_rows.saturating_sub(visible_rows);

                    let selected = columns_table_state.state.selected().unwrap_or(0);
                    let scroll_pos = selected.min(max_scroll);

                    columns_scroll_state = columns_scroll_state
                        .content_length(max_scroll + 1)
                        .position(scroll_pos);

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

                    let header_cells = columns
                        .iter()
                        .map(|c| Cell::from(c.header))
                        .collect::<Vec<_>>();

                    let table_header = Row::new(header_cells).height(1);

                    let rows = columns_data.iter().map(|c| {
                        let cells = columns
                            .iter()
                            .map(|col| Cell::from((col.value)(&c)))
                            .collect::<Vec<_>>();
                        Row::new(cells).height(1)
                    });

                    let widths = columns
                        .iter()
                        .map(|col| {
                            let max_len = columns_data
                                .iter()
                                .map(|c| (col.value)(&c).len())
                                .max()
                                .unwrap_or(0);
                            Constraint::Length((col.header.len().max(max_len) + 2) as u16)
                        })
                        .collect::<Vec<_>>();

                    let table = Table::new(rows, &widths)
                        .header(table_header)
                        .block(Block::default().borders(Borders::ALL).title(format!(
                            "Columns (Focused {} of {})",
                            selected + 1,
                            total_rows
                        )))
                        .row_highlight_style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol(">> ");

                    f.render_stateful_widget(table, content_area, &mut columns_table_state.state);
                    f.render_stateful_widget(
                        Scrollbar::new(ScrollbarOrientation::VerticalRight)
                            .symbols(scrollbar::VERTICAL)
                            .begin_symbol(Some("↑"))
                            .end_symbol(Some("↓")),
                        content_area,
                        &mut columns_scroll_state,
                    );
                }

                SelectedTab::Map => {
                    let (bbox, message) = bbox
                        .project_to_ratatui_map_crs(&src_proj_crs_string)
                        .unwrap();
                    let canvas = make_map_with_bbox_overlay(&message, &bbox);
                    f.render_widget(canvas, content_area);
                }
            }
        })?;

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
                KeyCode::Char('c')
                    if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    break;
                }
                KeyCode::Down | KeyCode::Char('j') => match selected_tab {
                    SelectedTab::Metadata => {
                        metadata_scroll = metadata_scroll.saturating_add(1);
                        metadata_scroll_state = metadata_scroll_state.position(metadata_scroll);
                    }
                    SelectedTab::Columns => {
                        columns_table_state.next(header.columns().unwrap_or_default().len());
                    }
                    _ => {}
                },
                KeyCode::Up | KeyCode::Char('k') => match selected_tab {
                    SelectedTab::Metadata => {
                        metadata_scroll = metadata_scroll.saturating_sub(1);
                        metadata_scroll_state = metadata_scroll_state.position(metadata_scroll);
                    }
                    SelectedTab::Columns => {
                        columns_table_state.previous(header.columns().unwrap_or_default().len());
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

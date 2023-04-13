use crossterm::event::KeyCode::Tab;
use crossterm::event::{self, Event, KeyCode, MouseEventKind};
use std::ops::Add;
use tui::layout::Direction;
use tui::text::{Span, Spans};
use tui::widgets::Paragraph;
use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

const STYLE_DEFAULT: Style = Style {
    fg: None,
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
const STYLE_HEADER: Style = Style {
    fg: Some(Color::Blue),
    bg: Some(Color::Gray),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
const STYLE_INFO: Style = Style {
    fg: Some(Color::Cyan),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
const STYLE_WARNING: Style = Style {
    fg: Some(Color::LightYellow),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
const STYLE_VIOLATION: Style = Style {
    fg: Some(Color::LightRed),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const HEADERS: [&str; 5] = ["BYTE", "TYPE", "CH", "MESSAGE", "DATA"];

struct App<'a> {
    table_state: TableState,
    analysis: Vec<Vec<&'a str>>,
    messages: Vec<Vec<&'a str>>,
    viewport: u16,
    /// When `true` the table should automatically scroll to the bottom as
    /// new entries are added
    follow: bool,
}

impl<'a> App<'a> {
    pub(crate) fn new() -> App<'a> {
        App {
            table_state: TableState::default(),
            analysis: vec![
                vec![" 90", "STATUS", " 1", "Note On", "-"],
                vec![" 3C", "DATA  ", " 1", "Note On (Note)", "60"],
                vec![" F8", "STATUS", " -", "Timing Clock", "-"],
                vec![" 7F", "DATA  ", " 1", "Note On (Velocity)", "127"],
                vec![" 9F", "STATUS", "16", "Note On", "-"],
                vec![" 3C", "DATA  ", "16", "Note On (Note)", "60"],
                vec![" F8", "STATUS", " -", "Timing Clock", "-"],
                vec![" 7F", "DATA  ", "16", "Note On (Velocity)", "127"],
                vec![" 3E", "DATA  ", "16", "Note On (Note)", "62"],
                vec![" 7F", "DATA  ", "16", "Note On (Velocity)", "127"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" FE", "STATUS", " -", "Active Sense", "-"],
                vec![" 90", "STATUS", " 1", "Note On", "-"],
            ],
            messages: vec![],
            viewport: 0,
            follow: true,
        }
    }

    pub fn previous(&mut self) {
        self.follow = false;
        self.table_state.select(
            self.table_state
                .selected()
                .unwrap_or(0)
                .checked_sub(self.viewport as usize),
        );
    }
    pub fn next(&mut self) {
        self.follow = false;
        self.table_state.select(
            self.table_state
                .selected()
                .unwrap_or(self.analysis.len())
                .checked_add(self.viewport as usize),
        );
    }
    pub fn last(&mut self) {
        self.follow = true;
        self.table_state.select(Some(self.analysis.len() as usize));
    }
}

pub(crate) fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), anyhow::Error> {
    let mut app = App::new();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::PageDown => app.follow = true,
                KeyCode::End => app.follow = true,
                KeyCode::ScrollLock => app.follow = !app.follow,
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => app.previous(),
                MouseEventKind::ScrollDown => app.next(),
                _ => {}
            },
            _ => {}
        }
    }
}

fn ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    let size = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .margin(0)
        .split(frame.size());
    app.viewport = chunks[0].height.checked_sub(1).unwrap_or(0);

    // Menu bar
    let menu_bar = Table::new(vec![])
        .header(Row::new(vec![
            Cell::from(Spans::from(vec![
                Span::styled("F1", STYLE_HEADER),
                Span::styled(" FILTER", STYLE_DEFAULT),
            ])),
            Cell::from(Spans::from(vec![
                Span::styled("F2", STYLE_HEADER),
                Span::styled(" LOAD", STYLE_DEFAULT),
            ])),
            Cell::from(Spans::from(vec![
                Span::styled("F3", STYLE_HEADER),
                Span::styled(" SAVE", STYLE_DEFAULT),
            ])),
            Cell::from(Spans::from(vec![
                Span::styled("Q", STYLE_HEADER),
                Span::styled(" QUIT", STYLE_DEFAULT),
            ])),
        ]))
        .widths(&[
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ]);
    frame.render_widget(menu_bar, chunks[2]);

    // Table header
    let header_cells = HEADERS.iter().map(|h| Cell::from(*h).style(STYLE_HEADER));
    let header = Row::new(header_cells)
        .style(STYLE_HEADER)
        .height(1)
        .bottom_margin(0);

    // Table rows
    let rows = app.analysis.iter().map(|item| {
        let cells = item.iter().map(|c| Cell::from(*c));
        Row::new(cells)
            .height(1)
            .bottom_margin(0)
            .style(STYLE_DEFAULT)
    });

    // Table
    let table_widths = [
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(6),
        // Constraint::Min(10),
        Constraint::Length(size.width.checked_sub(40).unwrap_or(8).max(8)),
        Constraint::Length(6),
    ];
    let table = Table::new(rows)
        .header(header)
        // .block(Block::default().borders(Borders::ALL).title("MIDI In Raw"))
        .widths(&table_widths)
        .highlight_symbol("*")
        .column_spacing(1);
    if app.follow {
        app.table_state.select(app.analysis.len().checked_sub(1));
    }
    frame.render_stateful_widget(table, chunks[0], &mut app.table_state);
}

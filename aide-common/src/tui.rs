use std::io::Write;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

pub trait ToStringVec {
    fn to_string_vec(&self) -> Vec<String>;
}

pub trait ToListState {
    fn to_state(&self) -> ListState;
}

pub trait GetTitle {
    fn get_title(&self) -> &str;
}

pub fn draw_list<B: Backend, T: ToStringVec + ToListState + GetTitle>(
    f: &mut Frame<B>,
    object: &mut T,
) {
    let sv = object.to_string_vec();
    let list = sv
        .to_list()
        .block(
            Block::default()
                .title(object.get_title())
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, f.size(), &mut object.to_state())
}

pub fn tui_setup() -> Result<Terminal<impl Backend + Write>, std::io::Error> {
    enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    Ok(terminal)
}

pub fn tui_teardown<B: Backend + Write>(terminal: &mut Terminal<B>) -> Result<(), std::io::Error> {
    disable_raw_mode()?;
    // leave the alternate screen, restoring the original terminal
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;
    Ok(())
}
trait ToList {
    fn to_list(&self) -> List;
}

impl ToList for Vec<String> {
    fn to_list(&self) -> List<'_> {
        List::new(
            self.iter()
                .map(|s| ListItem::new(s.as_str()))
                .collect::<Vec<_>>(),
        )
    }
}

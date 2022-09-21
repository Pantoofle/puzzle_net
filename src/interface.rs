use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame, Terminal,
};

use crossterm::{
    event,
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{io, time::Duration};
use tui::layout::Margin;
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::canvas::{Canvas, Context};

use unicode_segmentation::UnicodeSegmentation;

use crate::game::Game;


pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut game: Game
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &game))?;

        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    /*
                    KeyCode::Down => {
                        app.y += 1.0;
                    }
                    KeyCode::Up => {
                        app.y -= 1.0;
                    }
                    KeyCode::Right => {
                        app.x += 1.0;
                    }
                    KeyCode::Left => {
                        app.x -= 1.0;
                    }
                    */
                    _ => {}
                }
            }
        }
    }
}

fn paint_game(ctx: &mut Context, game: &Game){
    let symbols = "╵╶╷╴└┌┐┘│─│─┬┤┴├";

    for y in 0..game.height {
        for x in 0..game.width {
            if let Some(cell) = game.get_cell(x, y){
                let symbol = symbols.graphemes(true)
                                    .nth(cell.version as usize * 4 + cell.orientation as usize)
                                    .expect(format!("No char at position {}", cell.version as usize * 4 + cell.orientation as usize).as_str());
                let fg = if cell.powered {Color::LightBlue} else {Color::White};
                let bg = if cell.locked {Color::DarkGray} else {Color::Black};

                ctx.print(x as f64, y as f64,Span::styled(symbol, Style::default().fg(fg).bg(bg)));
            } else {
                ctx.print(x as f64, y as f64, "X");
            }

        }

   }


}


fn ui<B: Backend>(f: &mut Frame<B>, game: &Game) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(game.width as u16 + 4), Constraint::Min(0)].as_ref())
        .split(f.size());

    let board_area = chunks[0].inner(&Margin{vertical: 1, horizontal: 1});

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Board"))
        .x_bounds([-2.0, board_area.width as f64 - 2.0])
        .y_bounds([-1.0, board_area.height as f64 - 1.0])
        .paint(|ctx| { paint_game(ctx, game) });
    f.render_widget(canvas, chunks[0]);
}


pub fn run(mut game: Game) -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal, game)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

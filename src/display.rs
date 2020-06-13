use std::fmt;

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset},
    Terminal,
};

use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use termion::input::TermRead;



pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
#[allow(dead_code)]
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            //tick_rate: Duration::from_millis(250),
            tick_rate: Duration::from_millis(100),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                        if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                            return;
                        }
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                tx.send(Event::Tick).unwrap();
                thread::sleep(config.tick_rate);
            })
        };
        Events {
            rx,
            ignore_exit_key,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }
}


struct App {
    signal: Vec<(f64, f64)>,
    data: Vec<(f64, f64)>,
    window: [f64; 2],
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let signal_head = &self.signal[..2];
        let signal_tail = &self.signal[(self.signal.len()-2)..];
        let data_head = &self.data[..2];
        let data_tail = &self.data[(self.data.len()-2)..];
        write!(
            f,
            "App:\n  signal_head: {:?} ... {:?},\n  data_head: {:?} ... {:?},\n  window: {:?}",
            signal_head, signal_tail, data_head, data_tail, self.window
        )
    }
}

impl App {
    fn new(data: Vec<i16>) -> App {
        let mut signal: Vec<(f64, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64, *x as f64))
            .collect();
        let data = signal.drain(..200).collect::<Vec<(f64, f64)>>();
        App {
            signal,
            data,
            window: [0.0, 20.0],
        }
    }

    fn update(&mut self) {
        for _ in 0..5 {
            self.data.remove(0);
        }
        self.data.extend(self.signal.drain(..5));
        self.window[0] += 5.0;
        self.window[1] += 5.0;
    }
}

pub fn run(signal: Vec<i16>) -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    let mut app = App::new(signal);

    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Ratio(1, 3),].as_ref(),)
                .split(size);
            let x_labels = [
                format!("{}", app.window[0]),
                format!("{}", (app.window[0] + app.window[1]) / 2.0),
                format!("{}", app.window[1]),
            ];
            let datasets = [
                Dataset::default()
                    .name("wav")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&app.data[..]),
            ];
            let chart = Chart::default()
                .block(
                    Block::default()
                        .title("Chart 1")
                        .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::BOLD))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("X Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::ITALIC))
                        .bounds(app.window)
                        .labels(&x_labels),
                )
                .y_axis(
                    Axis::default()
                        .title("Y Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::ITALIC))
                        .bounds([-30_000.0, 30_000.0])
                        .labels(&["-30_000", "0", "30_000"]),
                )
                .datasets(&datasets);
            f.render_widget(chart, chunks[0]);

        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == Key::Char('q') {
                    break;
                }
            }
            Event::Tick => {
                app.update();
            }
        }
    }

    Ok(())
}


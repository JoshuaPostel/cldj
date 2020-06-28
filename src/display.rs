use std::fmt;

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, BarChart},
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

use super::transform::fourier_transform;



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
    signal_buf: Vec<(f64, f64)>,
    window: [f64; 2],
    frequency: Vec<(String, u64)>,
    max: f64,
    min: f64,
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let signal_head = &self.signal[..2];
        let signal_tail = &self.signal[(self.signal.len()-2)..];
        let signal_buf_head = &self.signal_buf[..2];
        let signal_buf_tail = &self.signal_buf[(self.signal_buf.len()-2)..];
        write!(
            f,
            "App:\n  signal_head: {:?} ... {:?},\n  data_head: {:?} ... {:?},\n  window: {:?}",
            signal_head, signal_tail, signal_buf_head, signal_buf_tail, self.window
        )
    }
}

impl App {
    fn new(data: Vec<i16>) -> App {
        let max = *data.iter().max().expect("could not get max") as f64;
        let min = *data.iter().min().expect("could not get min") as f64;
        let mut signal: Vec<(f64, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64, *x as f64))
            .collect();
        let signal_buf = signal.drain(..200).collect::<Vec<(f64, f64)>>();
        let freq = fourier_transform(data);
        let _f2: Vec<(String, u64)> = freq
            .iter()
            .enumerate()
            .map(|(i, f)| (i.to_string(), f.norm() as u64))
            .collect();

        let frequency: Vec<(String, u64)> = freq
            .iter()
            .enumerate()
            .map(|(i, f)| (i.to_string(), f.norm() as u64))
            .collect();

        App {
            signal,
            signal_buf,
            window: [0.0, 100.0],
            frequency: frequency,
            max,
            min,
        }
    }

    fn update(&mut self) {
        for _ in 0..5 {
            self.signal_buf.remove(0);
        }
        self.signal_buf.extend(self.signal.drain(..5));
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

            //TODO open an issue on rust-tui about unerganomic type
            let frequency_retyped: Vec<(&str, u64)> = app
                .frequency
                .iter()
                .map(|(x, y)| (x.as_str(), *y))
                .collect();

            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                //.constraints([Constraint::Ratio(1, 2),].as_ref(),)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),)
                .split(size);
            let x_labels = [
                format!("{}", app.window[0]),
                format!("{}", (app.window[0] + app.window[1]) / 2.0),
                format!("{}", app.window[1]),
            ];
            let y_labels = [
                format!("{}", app.min),
                format!("{}", (app.min + app.max) / 2.0),
                format!("{}", app.max),
            ];
            let datasets = [
                Dataset::default()
                    .name("wav")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&app.signal_buf[..]),
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
                        .bounds([app.min, app.max])
                        .labels(&y_labels),
                )
                .datasets(&datasets);
            f.render_widget(chart, chunks[0]);

            let barchart = BarChart::default()
                .block(Block::default().title("Data1").borders(Borders::ALL))
                .data(&frequency_retyped)
                .bar_width(2)
                .style(Style::default().fg(Color::Yellow))
                .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));
            f.render_widget(barchart, chunks[1]);
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


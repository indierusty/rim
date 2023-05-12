use std::{
    io::{stdout, Write},
    time::Duration,
};

mod buffer;

use buffer::Buffer;
use crossterm::{
    cursor,
    event::{self, poll, Event, KeyEvent},
    style,
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
    QueueableCommand, Result,
};

fn main() -> Result<()> {
    let mut rim = Rim::new();
    rim.init();
    rim.run()?;
    Ok(())
}

pub type Window = [Row; 100];
pub type Row = [char; 1000];
/* A rect frame inside the main Window */
#[derive(Clone, Copy)]
pub struct Frame {
    start_row: u16, /* where to start printing */
    start_col: u16,
    end_row: u16, /* where the row ends exclusive */
    end_col: u16,
}

impl Frame {
    pub fn new(start_row: u16, start_col: u16, end_row: u16, end_col: u16) -> Self {
        Self {
            start_row,
            start_col,
            end_row,
            end_col,
        }
    }
}

struct Rim {
    buf: Buffer, /* content of file to edit */
    window_width: u16,
    window_height: u16,
    exit: bool,
}

impl Rim {
    pub fn new() -> Self {
        let terminal_size = size().unwrap();

        Self {
            buf: Buffer::new(Some("test.txt".to_string())),
            window_width: terminal_size.0,
            window_height: terminal_size.1,
            exit: false,
        }
    }

    pub fn init(&self) {
        enable_raw_mode().unwrap();
    }

    fn process_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            event::KeyCode::Backspace => self.buf.delete_char(),
            event::KeyCode::Enter => self.buf.insert_nl(),
            event::KeyCode::Left => self.buf.move_left(),
            event::KeyCode::Right => self.buf.move_right(),
            event::KeyCode::Up => _ = self.buf.move_up(),
            event::KeyCode::Down => _ = self.buf.move_down(),
            event::KeyCode::Char(c) => self.buf.insert_char(c),
            event::KeyCode::Esc => self.exit = true,
            _ => {}
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        let mut stdout = stdout();
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;

        let mut window: Window = [[' '; 1000]; 100];
        let frame = Some(Frame::new(0, 0, self.window_height, self.window_width));

        loop {
            // `poll()` waits for an `Event` for a given time period
            if poll(Duration::from_millis(500)).unwrap() {
                match event::read().unwrap() {
                    Event::Key(key_event) => self.process_key(key_event),
                    Event::Resize(cols, rows) => {
                        self.window_width = cols;
                        self.window_height = rows;
                    }
                    _ => {}
                }
            } else {
                // Timeout expired and no `Event` is available
            }

            if let Some(frame) = frame {
                self.buf.print(&mut window, frame)?;

                stdout.queue(cursor::MoveTo(0, 0))?;
                stdout.queue(cursor::Hide)?;
                for row in 0..self.window_height as usize {
                    for col in 0..self.window_width as usize {
                        stdout.queue(style::Print(window[row][col]))?;
                    }
                    stdout.queue(cursor::MoveToNextLine(1))?;
                }
                stdout.queue(cursor::MoveTo(
                    frame.start_col as u16 + self.buf.col_in_frame(),
                    frame.start_row as u16 + self.buf.row_in_frame(),
                ))?;
            }

            stdout.queue(cursor::Show)?;

            stdout.flush()?; // important to execute all cmd in queue
            if self.exit {
                break;
            }
        }
        Ok(())
    }
}

impl Drop for Rim {
    fn drop(&mut self) {
        let mut stdout = stdout();
        stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
        stdout.queue(cursor::MoveTo(0, 0)).unwrap();
        stdout.queue(cursor::Show).unwrap();
        disable_raw_mode().unwrap();
        stdout.flush().unwrap();
    }
}

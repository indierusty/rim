use std::{
    io::{stdout, StdoutLock, Write},
    time::Duration,
};

use crate::buffer::Buffer;

use crossterm::{
    cursor,
    event::{self, poll, Event, KeyEvent, KeyModifiers},
    style,
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
    QueueableCommand,
};

const MAX_COLS: usize = 1000; // max chars in a line
const MAX_ROWS: usize = 100; // max line

pub type Window = [Row; MAX_ROWS];
pub type Row = [char; MAX_COLS];

/* A rect frame inside the main Window */
#[derive(Clone, Copy)]
pub struct Frame {
    pub start_row: u16, /* where to start printing */
    pub start_col: u16,
    pub end_row: u16, /* where the row ends exclusive */
    pub end_col: u16,
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

pub struct Rim<'a> {
    buf: Buffer,
    window: Window,
    window_width: u16,
    window_height: u16,
    exit: bool,
    stdout: StdoutLock<'a>,
}

impl<'a> Rim<'a> {
    pub fn new(file_path: &str) -> Self {
        let terminal_size = size().unwrap();

        Self {
            buf: Buffer::new(Some(file_path.to_string())),
            window: [[' '; MAX_COLS]; MAX_ROWS],
            window_width: terminal_size.0,
            window_height: terminal_size.1,
            exit: false,
            stdout: stdout().lock(),
        }
    }

    fn init(&self) {
        enable_raw_mode().unwrap();
    }

    fn process_key(&mut self, key_event: KeyEvent) {
        // NOTE: KeyModifiers are bitfields
        // if only control is pressed [among keymodifiers {SHIFT, CAPSLOCK, etc}]
        if key_event.modifiers == KeyModifiers::CONTROL {
            match key_event.code {
                // ctrl + s => for save
                event::KeyCode::Char('s') => self.buf.save(),
                event::KeyCode::Char('q') => self.exit = true,
                _ => {}
            }
        } else {
            // NOTE: on Shift + char, Char is also Uppercase, hence no extra work for it
            match key_event.code {
                event::KeyCode::Backspace => self.buf.delete_char(),
                event::KeyCode::Enter => self.buf.insert_nl(),
                event::KeyCode::Left => self.buf.move_left(),
                event::KeyCode::Right => self.buf.move_right(),
                event::KeyCode::Up => _ = self.buf.move_up(),
                event::KeyCode::Down => _ = self.buf.move_down(),
                event::KeyCode::Char(c) => self.buf.insert_char(c),
                _ => {}
            }
        }
    }

    pub fn run(mut self) -> std::io::Result<()> {
        self.init();
        // draw window buffer at start
        self.refresh_screen()?;

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
                // on any event refresh the screen / window
                self.refresh_screen()?;
            } else {
                // Timeout expired and no `Event` is available
            }

            if self.exit {
                break;
            }
        }

        self.exit();

        Ok(())
    }

    fn refresh_screen(&mut self) -> std::io::Result<()> {
        let frame = Some(Frame::new(0, 0, self.window_height, self.window_width));

        if let Some(frame) = frame {
            self.buf.print(&mut self.window, frame)?;

            self.stdout.queue(cursor::MoveTo(0, 0))?;
            self.stdout.queue(cursor::Hide)?;
            for row in 0..self.window_height as usize {
                for col in 0..self.window_width as usize {
                    self.stdout.queue(style::Print(self.window[row][col]))?;
                }
                self.stdout.queue(cursor::MoveToNextLine(1))?;
            }
            self.stdout.queue(cursor::MoveTo(
                frame.start_col as u16 + self.buf.col_in_frame(),
                frame.start_row as u16 + self.buf.row_in_frame(),
            ))?;
        }

        self.stdout.queue(cursor::Show)?;

        self.stdout.flush()?; // important to execute all cmd in queue
        Ok(())
    }

    fn exit(&mut self) {
        // on exit
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
        self.stdout.queue(cursor::MoveTo(0, 0)).unwrap();
        self.stdout.queue(cursor::Show).unwrap();
        self.stdout.flush().unwrap();
    }
}

impl<'a> Drop for Rim<'a> {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}

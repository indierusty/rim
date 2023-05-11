use std::{
    io::{stdout, Write},
    time::Duration,
};

mod buffer;

use buffer::Buffer;
use crossterm::{
    cursor,
    event::{self, poll, Event, KeyEvent, KeyModifiers},
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
    QueueableCommand, Result,
};

fn main() -> Result<()> {
    let mut rim = Rim::new();
    rim.init();
    rim.run()?;
    Ok(())
}

struct Rim {
    buf: Buffer, /* content of file to edit */
    win_size_x: usize,
    win_size_y: usize,
    exit: bool,
}

impl Rim {
    pub fn new() -> Self {
        let terminal_size = size().unwrap();

        Self {
            buf: Buffer::new(Some("test.txt".to_string())),
            win_size_x: terminal_size.0 as usize,
            win_size_y: terminal_size.1 as usize,
            exit: false,
        }
    }

    pub fn init(&self) {
        enable_raw_mode().unwrap();
    }

    fn process_key(&mut self, key_event: KeyEvent) {
        if key_event == KeyEvent::new(event::KeyCode::Esc, KeyModifiers::NONE) {
            self.exit = true;
        }

        match key_event.code {
            // event::KeyCode::Backspace => todo!(),
            // event::KeyCode::Enter => todo!(),
            event::KeyCode::Left => self.buf.move_left(),
            event::KeyCode::Right => self.buf.move_right(),
            event::KeyCode::Up => _ = self.buf.move_up(),
            event::KeyCode::Down => _ = self.buf.move_down(),
            // event::KeyCode::Char(_) => todo!(),
            // event::KeyCode::Esc => todo!(),
            _ => {}
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        let mut stdout = stdout();
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;

        loop {
            // `poll()` waits for an `Event` for a given time period
            if poll(Duration::from_millis(200)).unwrap() {
                match event::read().unwrap() {
                    Event::Key(key_event) => self.process_key(key_event),
                    Event::Resize(x, y) => {
                        self.win_size_x = x as usize;
                        self.win_size_y = y as usize;
                    }
                    _ => {}
                }
            } else {
                // Timeout expired and no `Event` is available
            }
            self.buf.print(&mut stdout)?;
            stdout.queue(cursor::MoveTo(self.buf.x(), self.buf.y()))?;

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

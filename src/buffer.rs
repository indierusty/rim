use std::fs;
use std::io::Stdout;
use std::path::Path;

use crossterm::{cursor, style, terminal, QueueableCommand};

pub struct Buffer {
    data: Vec<Vec<char>>,
    name: String, /* path to file to edit */
    x: usize,     /* cursor pointing to row in data */
    y: usize,     /* cursor pointing to col in data */
}

impl Buffer {
    pub fn new(name: Option<String>) -> Self {
        let name = if let Some(name) = name {
            name
        } else {
            "".to_string()
        };

        let data = if Path::new(&name).is_file() {
            read_buffer(&name)
        } else {
            empty_buffer()
        };

        Self {
            data,
            name,
            x: 0,
            y: 0,
        }
    }

    pub fn print(&mut self, stdout: &mut Stdout) -> std::io::Result<()> {
        stdout.queue(cursor::MoveTo(0, 0))?;
        stdout.queue(cursor::Hide)?;

        for col in 0..self.data.len() {
            stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
            for row in 0..self.data[col].len() {
                stdout.queue(style::Print(self.data[col][row]))?;
            }
            stdout.queue(cursor::MoveToNextLine(1))?;
        }
        stdout.queue(cursor::Show)?;
        Ok(())
    }

    pub fn insert_nl(&mut self) {
        // make newline by copying all element of current line starting at cursor x
        let newline = (self.data[self.y][self.x..]).to_vec();
        self.data.insert(self.y + 1, newline);

        // trim the current line and push delimenter at end
        self.data[self.y].truncate(self.x);
        self.data[self.y].push('\0');

        // move the cursor to point first char of next line
        self.move_down();
        self.move_start_of_line();
    }

    pub fn join_line(&mut self) {
        if self.y == 0 {
            return; // no line above to join current line
        }

        // remove delemeter of line above current line
        self.data[self.y - 1].pop().unwrap();
        let above_line_len = self.data[self.y - 1].len();
        // join current line to above line and remove current line
        let mut line_to_join = self.data.remove(self.y);
        self.data[self.y - 1].append(&mut line_to_join);

        // reset cursor position
        self.move_up();
        self.x = above_line_len;
    }

    pub fn insert_char(&mut self, c: char) {
        self.data[self.y].insert(self.x, c);
        self.move_right();
    }

    pub fn delete_char(&mut self) {
        if self.x > 0 {
            self.data[self.y].remove(self.x - 1);
            self.move_left();
        } else {
            // join current line to ablove and delete current line
            self.join_line()
        }
    }
}

/* Cursor Movement */
impl Buffer {
    fn reset_x(&mut self) {
        // reset cursor x to point at min of current line len - 1 and previous cursor x
        self.x = self.x.min(self.data[self.y].len() - 1);
    }

    fn move_start_of_line(&mut self) {
        self.x = 0;
    }

    fn move_end_of_line(&mut self) {
        self.x = self.data[self.y].len() - 1;
    }

    pub fn move_up(&mut self) -> bool {
        // if moved a line up return true
        if self.y > 0 {
            self.y -= 1;
            self.reset_x();
            return true;
        }

        false
    }

    pub fn move_down(&mut self) -> bool {
        // if moved a line down return true
        if self.y < self.data.len() - 1 {
            self.y += 1;
            self.reset_x();
            return true;
        }

        false
    }

    pub fn move_right(&mut self) {
        if self.x < self.data[self.y].len() - 1 {
            self.x += 1;
        } else {
            if self.move_down() {
                self.move_start_of_line();
            }
        }
    }

    pub fn move_left(&mut self) {
        if self.x > 0 {
            self.x -= 1;
        } else {
            if self.move_up() {
                self.move_end_of_line();
            }
        }
    }
}

/* Getter */
impl Buffer {
    pub fn x(&self) -> u16 {
        self.x as u16
    }
    pub fn y(&self) -> u16 {
        self.y as u16
    }
}

/* Helper Functions */
fn read_buffer(file_path: &str) -> Vec<Vec<char>> {
    /* read each line of file separated by either '\n' || '\r'
    and add '\0' at end of each line */
    fs::read_to_string(&file_path)
        .unwrap()
        .split(|c| c == '\n' || c == '\r')
        .map(|line| {
            let mut line = line.chars().collect::<Vec<char>>();
            line.push('\0');
            line
        })
        .collect()
}

/*at least one empty line for buf_x and buf_y to point here x: 0, y: 0
  pointing at first line and last char which is just delimeter.
* this avoid index out of bound. */
fn empty_buffer() -> Vec<Vec<char>> {
    vec![vec!['\0']]
}

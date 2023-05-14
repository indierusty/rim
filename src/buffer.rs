use crate::rim::{Frame, Window};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub struct Buffer {
    data: Vec<Vec<char>>,
    name: String,   /* path to file to edit */
    cur_col: usize, /* cursor pointing to col/char in data */
    cur_row: usize, /* cursor pointing to row/line in data */
    buf_row: usize, /* starting row to print */
    buf_col: usize, /* starting col to print */
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
            cur_col: 0,
            cur_row: 0,
            buf_row: 0,
            buf_col: 0,
        }
    }

    pub fn save(&mut self) {
        let last_line_index = self.data.len() - 1;
        let byte_string: String = self
            .data
            .iter()
            .enumerate()
            .fold(vec![], |mut acc, (i, line)| {
                acc.extend_from_slice(line.as_slice());
                acc.pop(); // removes delimeter at end of each line in self.data
                if i != last_line_index {
                    acc.push('\n'); // nl at the end of every line except last line
                }
                return acc;
            })
            .into_iter()
            .collect();

        let save_to_file = |mut f: File| {
            f.write(byte_string.as_bytes()).unwrap();
        };

        let path = Path::new(&self.name);
        match File::create(path) {
            Ok(f) => save_to_file(f),
            Err(e) => eprintln!("Err creating file: {e}"),
        }
    }

    pub fn print(&mut self, window: &mut Window, frame: Frame) -> std::io::Result<()> {
        // Important to shift buffer position so cursor remain inside frame
        self.confine_frame(frame);

        let mut data_row = self.buf_col; /* row pointing in self.data */
        let mut row = frame.start_row; /* row pointing in window frame */
        //
        while row < frame.end_row {
            //
            if data_row < self.data.len() {
                // print row
                let mut data_col = self.buf_row;
                let mut col = frame.start_col;

                while col < frame.end_col {
                    if data_col < self.data[data_row].len() {
                        window[row as usize][col as usize] = self.data[data_row][data_col];
                    } else {
                        window[row as usize][col as usize] = ' ';
                    }
                    data_col += 1;
                    col += 1;
                }
            } else {
                // line is empty hence print a tilde at start
                window[row as usize][frame.start_col as usize] = '~';
                // and the rest of line is empty, important erase previous garbage chars
                let mut col = frame.start_col + 1;
                while col < frame.end_col {
                    window[row as usize][col as usize] = ' ';
                    col += 1;
                }
            }

            data_row += 1;
            row += 1;
        }
        Ok(())
    }

    pub fn insert_nl(&mut self) {
        // make newline by copying all element of current line starting at cursor x
        let newline = (self.data[self.cur_row][self.cur_col..]).to_vec();
        self.data.insert(self.cur_row + 1, newline);

        // trim the current line and push delimenter at end
        self.data[self.cur_row].truncate(self.cur_col);
        self.data[self.cur_row].push('\0');

        // move the cursor to point first char of next line
        self.move_down();
        self.move_start_of_line();
    }

    pub fn join_line(&mut self) {
        if self.cur_row == 0 {
            return; // no line above to join current line
        }

        // remove delemeter of line above current line
        self.data[self.cur_row - 1].pop().unwrap();
        let above_line_len = self.data[self.cur_row - 1].len();
        // join current line to above line and remove current line
        let mut line_to_join = self.data.remove(self.cur_row);
        self.data[self.cur_row - 1].append(&mut line_to_join);

        // reset cursor position
        self.move_up();
        self.cur_col = above_line_len;
    }

    /// insert char at cursor and shifts the cursor right
    pub fn insert_char(&mut self, c: char) {
        self.data[self.cur_row].insert(self.cur_col, c);
        self.move_right();
    }

    /// delete char just behind the cursor and shifts the cursor left
    pub fn delete_char(&mut self) {
        if self.cur_col > 0 {
            self.data[self.cur_row].remove(self.cur_col - 1);
            self.move_left();
        } else {
            // join current line to above and delete current line
            self.join_line()
        }
    }
}

/// Cursor Movement
impl Buffer {
    /// shifts row and col cursor until inside of frame window
    fn confine_frame(&mut self, frame: Frame) {
        let frame_height = frame.end_row - frame.start_row;
        let frame_width = frame.end_col - frame.start_col;

        // row
        while self.cur_row < self.buf_col {
            self.buf_col -= 1;
        }

        while self.cur_row > self.buf_col + frame_height as usize - 1 {
            self.buf_col += 1;
        }

        // col
        while self.cur_col < self.buf_row {
            self.buf_row -= 1;
        }

        while self.cur_col > self.buf_row + frame_width as usize - 1 {
            self.buf_row += 1;
        }
    }

    /// reset cursor x to point at min of current line len - 1 and previous cursor x
    fn reset_x(&mut self) {
        self.cur_col = self.cur_col.min(self.data[self.cur_row].len() - 1);
    }

    /// shifts cursor to first col in current row
    fn move_start_of_line(&mut self) {
        self.cur_col = 0;
    }

    /// shifts cursor to end col in current row
    fn move_end_of_line(&mut self) {
        self.cur_col = self.data[self.cur_row].len() - 1;
    }

    /// shifts cursor up a row
    pub fn move_up(&mut self) -> bool {
        // if moved a line up return true
        if self.cur_row > 0 {
            self.cur_row -= 1;
            self.reset_x();
            return true;
        }

        false
    }

    /// shifts cursor down a row
    pub fn move_down(&mut self) -> bool {
        // if moved a line down return true
        if self.cur_row < self.data.len() - 1 {
            self.cur_row += 1;
            self.reset_x();
            return true;
        }

        false
    }

    /// shifts cursor right a column
    pub fn move_right(&mut self) {
        if self.cur_col < self.data[self.cur_row].len() - 1 {
            self.cur_col += 1;
        } else {
            if self.move_down() {
                self.move_start_of_line();
            }
        }
    }

    /// shifts cursor left a column
    pub fn move_left(&mut self) {
        if self.cur_col > 0 {
            self.cur_col -= 1;
        } else {
            if self.move_up() {
                self.move_end_of_line();
            }
        }
    }
}

/// Getters
impl Buffer {
    pub fn col_in_frame(&self) -> u16 {
        (self.cur_col - self.buf_row) as u16
    }
    pub fn row_in_frame(&self) -> u16 {
        (self.cur_row - self.buf_col) as u16
    }
}

/// Helper Function
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

/// At least one empty line for buf_x and buf_y to point here x: 0, y: 0
/// pointing at first line and last char which is just delimeter.
/// This avoid index out of bound
fn empty_buffer() -> Vec<Vec<char>> {
    vec![vec!['\0']]
}

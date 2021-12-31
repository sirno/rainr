use std::io::{BufWriter, Stdout, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{style, PrintStyledContent, StyledContent};

pub struct Screen {
    pub width: usize,
    pub height: usize,
    buffer: Vec<StyledContent<char>>,
}

impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![style(' '); width * height],
        }
    }

    pub fn clear(&mut self) {
        for character in &mut self.buffer {
            *character = style(' ');
        }
    }

    pub fn write_at_pos(&mut self, c: StyledContent<char>, row: usize, col: usize) {
        self.buffer[col + self.width * row] = c;
    }

    pub fn draw(&self, stdout: &mut BufWriter<Stdout>) -> crossterm::Result<()> {
        for (idx, c) in self.buffer.iter().enumerate() {
            queue!(
                stdout,
                MoveTo(
                    u16::try_from(idx % self.width).unwrap(),
                    u16::try_from(idx / self.width).unwrap(),
                ),
                PrintStyledContent(*c),
            )?;
        }
        stdout.flush()?;
        Ok(())
    }
}

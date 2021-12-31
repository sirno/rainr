use crossterm::{
    style::{style, Color, Stylize},
    terminal,
};
use ezemoji::{EZEmoji, Japanese};
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::screen::Screen;

#[derive(Debug)]
pub struct Droplet {
    row: u16,
    column: u16,
    length: u16,
    speed: Duration,
    last_update: Instant,
    trace: VecDeque<char>,
}

impl Droplet {
    pub fn new(row: u16, column: u16, length: u16, speed: Duration, rng: &mut ThreadRng) -> Self {
        let mut trace = VecDeque::with_capacity(length.into());
        trace.push_back(Droplet::draw_letter(rng));
        Self {
            row,
            column,
            length,
            speed,
            last_update: Instant::now(),
            trace,
        }
    }

    pub fn write(&self, screen: &mut Screen) {
        for (idx, character) in self.trace.iter().rev().enumerate() {
            let row: usize = self.row.abs_diff(idx.try_into().unwrap()).into();

            if row >= screen.height {
                continue;
            }

            let fac =
                f32::from(self.length.abs_diff(idx.try_into().unwrap())) / f32::from(self.length);
            let base = if idx == 0 { 200 } else { 0 };

            screen.write_at_pos(
                style(*character).with(Color::Rgb {
                    r: base,
                    g: (fac * 255.) as u8,
                    b: base,
                }),
                row,
                self.column.into(),
            );
        }
    }

    pub fn tick(&mut self, rng: &mut ThreadRng) -> crossterm::Result<()> {
        if self.last_update.elapsed() < self.speed {
            return Ok(());
        }

        self.row = self.row + 1;

        if self.is_full() {
            self.trace.pop_front();
        }

        if !self.is_running_out()? {
            self.trace.push_back(Droplet::draw_letter(rng));
        }

        self.last_update = Instant::now();

        Ok(())
    }

    pub fn draw_letter(rng: &mut ThreadRng) -> char {
        *Japanese.as_vec_char().choose(rng).unwrap()
    }

    pub fn is_running_out(&self) -> crossterm::Result<bool> {
        let (_, height) = terminal::size()?;
        Ok(self.row >= height)
    }

    pub fn is_invisible(&self) -> crossterm::Result<bool> {
        let (_, height) = terminal::size()?;
        Ok(self.row >= height + self.length)
    }

    pub fn is_full(&self) -> bool {
        self.trace.len() >= self.length.into()
    }
}

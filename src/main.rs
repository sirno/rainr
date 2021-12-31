#![feature(int_abs_diff)]
mod droplet;
mod screen;

use droplet::Droplet;
use screen::Screen;

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use crossterm::{self, cursor, execute};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::cell::RefCell;
use std::collections::vec_deque::VecDeque;
use std::io::{stdout, BufWriter, Write};
use std::time::Duration;

fn main() -> crossterm::Result<()> {
    let mut stdout = BufWriter::with_capacity(100, stdout());

    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    terminal::enable_raw_mode()?;

    let (width, height) = terminal::size()?;
    let mut screen: Screen = Screen::new(usize::from(width), usize::from(height));

    let mut rng = thread_rng();

    let rate_distribution: Uniform<f64> = Uniform::new(0., 1.);
    let columns_distribution = Uniform::new(0u16, width);
    let length_distribution = Uniform::new(1u16, height / 2);
    let speed_distribution = Uniform::new(50u64, 2000u64);

    let mut droplets: VecDeque<Box<RefCell<Droplet>>> = VecDeque::new();

    droplets.push_back(Box::new(RefCell::new(Droplet::new(
        rng.sample::<u16, _>(columns_distribution),
        rng.sample::<u16, _>(length_distribution),
        Duration::from_millis(rng.sample(speed_distribution)),
        &mut rng,
    ))));

    stdout.flush()?;

    loop {
        if poll(Duration::from_millis(20))? {
            match read() {
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                })) => break,
                _ => (),
            }
        }

        if rng.sample::<f64, _>(rate_distribution) < 0.7 {
            droplets.push_back(Box::new(RefCell::new(Droplet::new(
                rng.sample::<u16, _>(columns_distribution),
                rng.sample::<u16, _>(length_distribution),
                Duration::from_millis(rng.sample(speed_distribution)),
                &mut rng,
            ))));
        }

        if droplets.is_empty() {
            break;
        }

        screen.clear();

        let mut remove_droplets: Vec<*const _> = Vec::new();
        for droplet_ref in &droplets {
            let mut droplet = droplet_ref.as_ref().borrow_mut();
            droplet.write(&mut screen);
            droplet.tick(&mut rng)?;
            if droplet.is_invisible()? {
                remove_droplets.push(droplet_ref as *const _);
            }
        }

        screen.draw(&mut stdout)?;

        while {
            let front_droplet_ref = droplets.pop_front().unwrap();
            let front_droplet_raw = &front_droplet_ref as *const _;

            let mut remove_item = false;
            for droplet in &remove_droplets {
                if front_droplet_raw == *droplet {
                    remove_item = true;
                    break;
                }
            }

            if !remove_item {
                droplets.push_front(front_droplet_ref);
            } else {
                drop(front_droplet_ref);
            }

            remove_item && !droplets.is_empty()
        } {}

        stdout.flush()?;
    }
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}

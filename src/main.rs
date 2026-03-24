mod droplet;
mod screen;

use droplet::Droplet;
use screen::Screen;

use clap::Parser;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use crossterm::{self, cursor, execute};
use rand_distr::{Exp, Distribution, Uniform};
use std::collections::BTreeMap;
use std::io::{stdout, BufWriter, Write};
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(
        long,
        default_value_t = 0.7,
        help = "Create new droplet at rate `density`."
    )]
    density: f64,

    #[clap(long, default_value_t = 1000, help = "Max speed in ms.")]
    max_speed: u64,

    #[clap(long, default_value_t = 50, help = "Min speed in ms.")]
    min_speed: u64,

    #[clap(
        long,
        default_value_t = 0.,
        help = "Average starting row; exponentially distributed."
    )]
    average_start: f64,

    #[clap(
        long,
        default_value_t = 0.5,
        help = "Max length of droplet relative to screen height."
    )]
    max_length: f64,
}

fn main() -> crossterm::Result<()> {
    let args = Args::parse();
    let mut stdout = BufWriter::with_capacity(100, stdout());

    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    terminal::enable_raw_mode()?;

    let (width, height) = terminal::size()?;
    let mut screen: Screen = Screen::new(usize::from(width), usize::from(height));

    let mut rng = rand::rng();

    let rate_distribution: Uniform<f64> = Uniform::new(0., 1.).unwrap();
    let delay_distribution = Exp::new(1. / 0.5).unwrap();
    let columns_distribution = Uniform::new(0u16, width).unwrap();
    let row_distribution = rate_distribution.map(|val| -args.average_start * (1. - val).ln());
    let length_distribution = Uniform::new(1u16, (args.max_length * height as f64) as u16).unwrap();
    let speed_distribution = Uniform::new(args.min_speed, args.max_speed).unwrap();

    let mut droplets: BTreeMap<Instant, Droplet> = BTreeMap::new();
    let mut create_droplet_time = Instant::now() + Duration::from_millis((delay_distribution.sample(&mut rng) * 1000.) as u64);

    stdout.flush()?;

    loop {
        if poll(Duration::from_millis(20))? {
            if let Ok(Event::Key(KeyEvent {
                       code: KeyCode::Char('q'),
                       modifiers: KeyModifiers::NONE,
                   })) = read() { break }
        }

        if create_droplet_time <= Instant::now() {
            let new_droplet = Droplet::new(
                row_distribution.sample(&mut rng) as u16,
                columns_distribution.sample(&mut rng),
                length_distribution.sample(&mut rng),
                Duration::from_millis(speed_distribution.sample(&mut rng) as u64),
                &mut rng,
            );
            droplets.insert(Instant::now(), new_droplet);
            create_droplet_time = Instant::now() + Duration::from_millis((delay_distribution.sample(&mut rng) * 1000.) as u64);
        }

        if droplets.is_empty() {
            continue;
        }

        screen.clear();

        // Write all the droplets to the screen
        for droplet in droplets.values_mut() {
            droplet.write(&mut screen);
            droplet.tick(&mut rng)?;
        }

        droplets.retain(|_, droplet| !droplet.is_invisible().unwrap_or(false));

        let next_droplet_update = droplets.keys()
                .copied()
                .min()
                .unwrap_or_else(|| Instant::now() + Duration::from_millis(20));

        screen.draw(&mut stdout)?;

        // Sleep until the next droplet needs to be ticked
        let time_to_next_tick = next_droplet_update.saturating_duration_since(Instant::now());

        if time_to_next_tick.is_zero() {
            continue;
        }

        std::thread::sleep(time_to_next_tick);

        stdout.flush()?;
    }
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}

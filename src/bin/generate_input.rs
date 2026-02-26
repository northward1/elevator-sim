use anyhow::Result;
use clap::Parser;
use rand::SeedableRng;
use rand::distr::{Distribution, Uniform};
use rand_distr::Poisson;
use rand_pcg::Pcg64;
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Parser)]
struct Args {
    /// Start seed
    start: u64,
    /// End seed
    end: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Constant parameters as per README
    let n = 10;
    let m = 3;
    let c = 10;
    let t = 100;
    let lambda = 0.1;

    std::fs::create_dir_all("in")?;

    for seed in args.start..=args.end {
        let mut rng = Pcg64::seed_from_u64(seed);
        let poi = Poisson::new(lambda)?;
        let target_dist = Uniform::new(0, n)?;

        let path = format!("in/{:04}.txt", seed);
        let mut writer = BufWriter::new(File::create(path)?);
        // Header
        writeln!(writer, "{} {} {} {} {}", n, m, c, t, lambda)?;

        for i in 0..n {
            for turn in 0..t {
                let count: u32 = poi.sample(&mut rng) as u32;
                write!(writer, "{}", count)?;
                for _ in 0..count {
                    let mut target = target_dist.sample(&mut rng);
                    while target == i {
                        target = target_dist.sample(&mut rng);
                    }
                    write!(writer, " {}", target)?;
                }
                if turn == t - 1 {
                    writeln!(writer)?;
                } else {
                    write!(writer, " ")?;
                }
            }
        }
    }

    Ok(())
}

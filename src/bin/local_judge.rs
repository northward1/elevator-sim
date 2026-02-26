use anyhow::{Context, Result};
use clap::Parser;
use elevator_sim::{Passenger, SimulationState};
use proconio::input;
use proconio::source::once::OnceSource;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[derive(Parser)]
struct Args {
    input_file: String,
    command: String,
    #[clap(trailing_var_arg = true)]
    args: Vec<String>,
}

#[allow(clippy::needless_range_loop)]
fn main() -> Result<()> {
    let args = Args::parse();

    let input_content = std::fs::read_to_string(&args.input_file)
        .with_context(|| format!("Failed to read input file: {}", args.input_file))?;
    let mut source = OnceSource::from(input_content.as_str());

    input! {
        from &mut source,
        n: usize, m: usize, c: usize, t: usize, lambda: f64,
    }

    let mut passenger_source: Vec<Vec<Vec<Passenger>>> = vec![vec![vec![]; t]; n];
    let mut next_passenger_id = 0;

    for i in 0..n {
        for turn in 0..t {
            input! {
                from &mut source,
                count: usize,
                targets: [usize; count],
            }
            for target_floor in targets {
                passenger_source[i][turn].push(Passenger {
                    id: next_passenger_id,
                    arrival_turn: turn,
                    target_floor,
                });
                next_passenger_id += 1;
            }
        }
    }

    let mut state = SimulationState::new(n, m, c, t);

    let mut child = Command::new(&args.command)
        .args(&args.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to spawn agent process")?;

    let mut stdin = child.stdin.take().context("Failed to open stdin")?;
    let mut stdout = BufReader::new(child.stdout.take().context("Failed to open stdout")?);

    writeln!(stdin, "{} {} {} {} {}", n, m, c, t, lambda)?;
    stdin.flush()?;

    for turn in 0..t {
        state.turn = turn;
        for i in 0..n {
            for p in passenger_source[i][turn].drain(..) {
                state.waiting_passengers[i].push(p);
            }
        }

        // Send state to agent
        let h_line = state
            .elevators
            .iter()
            .map(|e| e.floor.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(stdin, "{}", h_line)?;

        for e in &state.elevators {
            write!(stdin, "{}", e.passengers.len())?;
            for p in &e.passengers {
                write!(stdin, " {} {}", p.target_floor, turn - p.arrival_turn)?;
            }
            writeln!(stdin)?;
        }

        for i in 0..n {
            write!(stdin, "{}", state.waiting_passengers[i].len())?;
            for p in &state.waiting_passengers[i] {
                write!(stdin, " {} {}", p.target_floor, turn - p.arrival_turn)?;
            }
            writeln!(stdin)?;
        }
        stdin.flush()?;

        // Process agent actions
        for i in 0..m {
            let mut action_line = String::new();
            if stdout.read_line(&mut action_line)? == 0 {
                anyhow::bail!(
                    "Agent process terminated unexpectedly at turn {} for elevator {}",
                    turn,
                    i
                );
            }
            let parts: Vec<&str> = action_line.split_whitespace().collect();
            if parts.is_empty() {
                anyhow::bail!("Empty action line at turn {} for elevator {}", turn, i);
            }
            let action = parts[0];
            let mut picks = vec![];
            if action == "OPEN" {
                for &p_idx_str in &parts[1..] {
                    picks.push(
                        p_idx_str
                            .parse::<usize>()
                            .context("Invalid passenger index format")?,
                    );
                }
            }
            state
                .apply_action(i, action, &picks)
                .with_context(|| format!("Turn {}: Invalid action by elevator {}", turn, i))?;
        }
    }

    println!("Score: {}", state.calculate_final_score());
    let _ = child.kill();
    Ok(())
}

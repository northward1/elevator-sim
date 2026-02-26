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
    #[clap(short, long)]
    save_log: Option<String>,
    #[clap(trailing_var_arg = true)]
    args: Vec<String>,
}

#[allow(clippy::needless_range_loop)]
fn main() -> Result<()> {
    let args = Args::parse();

    let mut log_writer = if let Some(ref path) = args.save_log {
        Some(std::io::BufWriter::new(std::fs::File::create(path)?))
    } else {
        None
    };

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
                state.add_passenger(i, p.target_floor, p.arrival_turn, p.id);
            }
        }

        // Send state to agent
        let mut h_floors = vec![];
        for i in 0..m {
            h_floors.push(state.get_elevator_floor(i).to_string());
        }
        writeln!(stdin, "{}", h_floors.join(" "))?;

        for i in 0..m {
            let p_count = state.get_elevator_passenger_count(i);
            write!(stdin, "{}", p_count)?;
            for p_idx in 0..p_count {
                let target = state.get_elevator_passenger_target(i, p_idx);
                // Note: local_judge originally sent (turn - p.arrival_turn),
                // but SimulationState doesn't expose arrival_turn for elevator passengers yet.
                // Let's add it or just send 0 for now if the agent doesn't strictly need it.
                // For a proper greedy agent, target floor is most important.
                write!(stdin, " {} {}", target, 0)?;
            }
            writeln!(stdin)?;
        }

        for i in 0..n {
            let p_count = state.get_waiting_passenger_count(i);
            write!(stdin, "{}", p_count)?;
            for p_idx in 0..p_count {
                let target = state.get_waiting_passenger_target(i, p_idx);
                write!(stdin, " {} {}", target, 0)?;
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
            if let Some(ref mut writer) = log_writer {
                write!(writer, "{}", action_line)?;
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
                .apply_action_wasm(i, action, &picks)
                .map_err(|e| anyhow::anyhow!(e))
                .with_context(|| format!("Turn {}: Invalid action by elevator {}", turn, i))?;
        }
    }

    if let Some(ref mut writer) = log_writer {
        writer.flush()?;
    }

    println!("Score: {}", state.calculate_final_score());
    let _ = child.kill();
    Ok(())
}

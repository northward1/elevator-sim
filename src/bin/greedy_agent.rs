use std::io::{self, BufRead};

#[allow(clippy::needless_range_loop)]
fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    // Read initial header: N M C T lambda
    let header_line = match lines.next() {
        Some(Ok(l)) => l,
        _ => return,
    };
    let header: Vec<&str> = header_line.split_whitespace().collect();
    if header.len() < 4 {
        return;
    }
    let n: usize = header[0].parse().unwrap();
    let m: usize = header[1].parse().unwrap();
    let c: usize = header[2].parse().unwrap();
    let t: usize = header[3].parse().unwrap();

    for _ in 0..t {
        // Read current floors of M elevators
        let h_line = lines.next().unwrap().unwrap();
        let h: Vec<usize> = h_line
            .split_whitespace()
            .map(|x| x.parse().unwrap())
            .collect();

        // Read elevator states
        let mut elevator_passengers = vec![];
        for _ in 0..m {
            let line = lines.next().unwrap().unwrap();
            let parts: Vec<usize> = line
                .split_whitespace()
                .map(|x| x.parse().unwrap())
                .collect();
            let count = parts[0];
            let mut ps = vec![];
            for j in 0..count {
                ps.push(parts[2 * j + 1]); // target_floor
            }
            elevator_passengers.push(ps);
        }

        // Read floor waiting states
        let mut floor_waiting = vec![];
        for _ in 0..n {
            let line = lines.next().unwrap().unwrap();
            let parts: Vec<usize> = line
                .split_whitespace()
                .map(|x| x.parse().unwrap())
                .collect();
            let count = parts[0];
            let mut ps = vec![];
            for j in 0..count {
                ps.push(parts[2 * j + 1]); // target_floor
            }
            floor_waiting.push(ps);
        }

        let mut picked_on_floor = vec![0; n];

        // Greedy Decision
        for i in 0..m {
            let current_floor = h[i];
            let my_passengers = &elevator_passengers[i];
            let waiting_here = &floor_waiting[current_floor];

            // 1. Should we OPEN to drop off?
            let has_delivery = my_passengers.contains(&current_floor);

            // 2. Should we OPEN to pick up?
            let available_to_pick = if waiting_here.len() > picked_on_floor[current_floor] {
                waiting_here.len() - picked_on_floor[current_floor]
            } else {
                0
            };
            let can_pickup = my_passengers.len() < c && available_to_pick > 0;

            if has_delivery || can_pickup {
                let mut output = String::from("OPEN");
                if can_pickup {
                    let space = c - my_passengers.len();
                    let num_to_pick = space.min(available_to_pick);
                    for j in 0..num_to_pick {
                        // The index must be relative to the CURRENT waiting list.
                        // However, local_judge applies actions sequentially.
                        // If elevator 0 picks passenger 0, elevator 1's passenger 0 is the OLD passenger 1.
                        // This is tricky. For this simple agent, let's just pick the first available.
                        output.push_str(&format!(" {}", j));
                    }
                    picked_on_floor[current_floor] += num_to_pick;
                }
                println!("{}", output);
                continue;
            }

            // 3. Move towards a destination
            if !my_passengers.is_empty() {
                let target = my_passengers[0];
                if target > current_floor {
                    println!("UP");
                } else {
                    println!("DOWN");
                }
            } else {
                // 4. Move towards nearest waiting passenger
                let mut best_floor = None;
                let mut min_dist = usize::MAX;
                for f in 0..n {
                    if !floor_waiting[f].is_empty() {
                        let dist = (f as isize - current_floor as isize).unsigned_abs();
                        if dist < min_dist {
                            min_dist = dist;
                            best_floor = Some(f);
                        }
                    }
                }

                if let Some(f) = best_floor {
                    if f > current_floor {
                        println!("UP");
                    } else {
                        println!("DOWN");
                    }
                } else {
                    println!("STAY");
                }
            }
        }
    }
}

use anyhow::{Result, bail};
use rand::SeedableRng;
use rand::distr::{Distribution, Uniform};
use rand_distr::Poisson;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passenger {
    pub id: usize,
    pub arrival_turn: usize,
    pub target_floor: usize,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Elevator {
    pub floor: usize,
    pub capacity: usize,
    #[serde(skip)]
    pub(crate) passengers: Vec<Passenger>,
}

#[wasm_bindgen]
impl Elevator {
    #[wasm_bindgen(getter)]
    pub fn passenger_count(&self) -> usize {
        self.passengers.len()
    }

    #[wasm_bindgen]
    pub fn get_passenger_target(&self, idx: usize) -> usize {
        self.passengers[idx].target_floor
    }

    #[wasm_bindgen]
    pub fn get_passenger_arrival_turn(&self, idx: usize) -> usize {
        self.passengers[idx].arrival_turn
    }
}

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub turn: usize,
    pub score: u64,
    pub elevators: Vec<ElevatorSnapshot>,
    pub floors: Vec<FloorSnapshot>,
}

#[derive(Serialize, Deserialize)]
pub struct ElevatorSnapshot {
    pub floor: usize,
    pub passenger_count: usize,
    pub passengers: Vec<Passenger>,
}

#[derive(Serialize, Deserialize)]
pub struct FloorSnapshot {
    pub waiting_count: usize,
    pub waiting: Vec<Passenger>,
}

#[wasm_bindgen]
pub struct SimulationState {
    pub n: usize,
    pub m: usize,
    pub c: usize,
    pub t: usize,
    pub turn: usize,
    pub score: u64,
    elevators: Vec<Elevator>,
    waiting_passengers: Vec<Vec<Passenger>>,
}

impl SimulationState {
    pub fn apply_action(
        &mut self,
        elevator_idx: usize,
        action: &str,
        picks: &[usize],
    ) -> Result<()> {
        if elevator_idx >= self.m {
            bail!("Invalid elevator index: {}", elevator_idx);
        }

        match action {
            "UP" => {
                self.elevators[elevator_idx].floor =
                    (self.elevators[elevator_idx].floor + 1).min(self.n - 1);
            }
            "DOWN" => {
                self.elevators[elevator_idx].floor =
                    self.elevators[elevator_idx].floor.saturating_sub(1);
            }
            "STAY" => {}
            "OPEN" => {
                let current_floor = self.elevators[elevator_idx].floor;

                // 1. Drop off
                let (delivered, remaining): (Vec<Passenger>, Vec<Passenger>) = self.elevators
                    [elevator_idx]
                    .passengers
                    .drain(..)
                    .partition(|p| p.target_floor == current_floor);

                for p in delivered {
                    let duration = self.turn - p.arrival_turn + 1;
                    self.score += (duration as u64).pow(2);
                }
                self.elevators[elevator_idx].passengers = remaining;

                // 2. Pick up
                let mut sorted_picks = picks.to_vec();
                sorted_picks.sort_unstable_by(|a, b| b.cmp(a)); // Descending to remove safely

                for &idx in &sorted_picks {
                    if idx >= self.waiting_passengers[current_floor].len() {
                        bail!("Invalid passenger index {} at floor {}", idx, current_floor);
                    }
                    if self.elevators[elevator_idx].passengers.len()
                        >= self.elevators[elevator_idx].capacity
                    {
                        continue;
                    }
                    let p = self.waiting_passengers[current_floor].remove(idx);
                    self.elevators[elevator_idx].passengers.push(p);
                }
            }
            _ => bail!("Unknown action: {}", action),
        }
        Ok(())
    }

    pub fn create_snapshot(&self) -> Snapshot {
        Snapshot {
            turn: self.turn,
            score: self.score,
            elevators: self
                .elevators
                .iter()
                .map(|e| ElevatorSnapshot {
                    floor: e.floor,
                    passenger_count: e.passengers.len(),
                    passengers: e.passengers.clone(),
                })
                .collect(),
            floors: self
                .waiting_passengers
                .iter()
                .map(|f| FloorSnapshot {
                    waiting_count: f.len(),
                    waiting: f.clone(),
                })
                .collect(),
        }
    }
}

#[wasm_bindgen]
impl SimulationState {
    #[wasm_bindgen(constructor)]
    pub fn new(n: usize, m: usize, c: usize, t: usize) -> Self {
        Self {
            n,
            m,
            c,
            t,
            elevators: (0..m)
                .map(|_| Elevator {
                    floor: n / 2,
                    passengers: vec![],
                    capacity: c,
                })
                .collect(),
            waiting_passengers: vec![vec![]; n],
            turn: 0,
            score: 0,
        }
    }

    #[wasm_bindgen]
    pub fn apply_action_wasm(
        &mut self,
        elevator_idx: usize,
        action: &str,
        picks: &[usize],
    ) -> Result<(), String> {
        self.apply_action(elevator_idx, action, picks)
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen]
    pub fn calculate_final_score(&self) -> u64 {
        let mut final_score = self.score;
        for floor_passengers in &self.waiting_passengers {
            for p in floor_passengers {
                let duration = self.t - p.arrival_turn;
                final_score += (duration as u64).pow(2);
            }
        }
        for e in &self.elevators {
            for p in &e.passengers {
                let duration = self.t - p.arrival_turn;
                final_score += (duration as u64).pow(2);
            }
        }
        final_score
    }

    #[wasm_bindgen]
    pub fn get_elevator_floor(&self, idx: usize) -> usize {
        self.elevators[idx].floor
    }

    #[wasm_bindgen]
    pub fn get_elevator_passenger_count(&self, idx: usize) -> usize {
        self.elevators[idx].passengers.len()
    }

    #[wasm_bindgen]
    pub fn get_elevator_passenger_target(&self, elevator_idx: usize, p_idx: usize) -> usize {
        self.elevators[elevator_idx].passengers[p_idx].target_floor
    }

    #[wasm_bindgen]
    pub fn get_waiting_passenger_count(&self, floor: usize) -> usize {
        self.waiting_passengers[floor].len()
    }

    #[wasm_bindgen]
    pub fn get_waiting_passenger_target(&self, floor: usize, p_idx: usize) -> usize {
        self.waiting_passengers[floor][p_idx].target_floor
    }

    #[wasm_bindgen]
    pub fn get_waiting_passenger_arrival_turn(&self, floor: usize, p_idx: usize) -> usize {
        self.waiting_passengers[floor][p_idx].arrival_turn
    }

    #[wasm_bindgen]
    pub fn add_passenger(&mut self, floor: usize, target: usize, arrival_turn: usize, id: usize) {
        self.waiting_passengers[floor].push(Passenger {
            id,
            arrival_turn,
            target_floor: target,
        });
    }
}

#[wasm_bindgen]
#[allow(clippy::needless_range_loop)]
pub fn run_simulation_wasm(seed: u64, output_text: &str) -> Result<JsValue, String> {
    let n = 10;
    let m = 3;
    let c = 10;
    let t = 100;
    let lambda = 0.1;

    let mut rng = Pcg64::seed_from_u64(seed);
    let poi = Poisson::new(lambda).map_err(|e| e.to_string())?;
    let target_dist = Uniform::new(0, n).map_err(|e| e.to_string())?;

    // Pre-generate all passengers for all floors and turns to match local_judge exactly
    let mut passenger_source: Vec<Vec<Vec<Passenger>>> = vec![vec![vec![]; t]; n];
    let mut next_id = 0;
    for i in 0..n {
        for turn in 0..t {
            let count: u32 = poi.sample(&mut rng) as u32;
            for _ in 0..count {
                let mut target = target_dist.sample(&mut rng);
                while target == i {
                    target = target_dist.sample(&mut rng);
                }
                passenger_source[i][turn].push(Passenger {
                    id: next_id,
                    arrival_turn: turn,
                    target_floor: target,
                });
                next_id += 1;
            }
        }
    }

    let mut sim = SimulationState::new(n, m, c, t);
    let mut history = Vec::with_capacity(t);

    let output_lines: Vec<&str> = output_text.trim().split('\n').collect();
    let mut current_line = 0;

    for turn in 0..t {
        sim.turn = turn;
        // Add pre-generated passengers for this turn
        for floor in 0..n {
            let passengers = std::mem::take(&mut passenger_source[floor][turn]);
            for p in passengers {
                sim.add_passenger(floor, p.target_floor, p.arrival_turn, p.id);
            }
        }

        // Apply actions
        for el_idx in 0..m {
            if current_line >= output_lines.len() {
                return Err(format!(
                    "Output too short. Expected {} lines ({} turns * {} elevators), found {}.",
                    t * m,
                    t,
                    m,
                    output_lines.len()
                ));
            }
            let line = output_lines[current_line].trim();
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let action = parts[0];
                let picks: Vec<usize> = parts[1..].iter().map(|x| x.parse().unwrap_or(0)).collect();
                sim.apply_action(el_idx, action, &picks)
                    .map_err(|e| format!("Turn {}: {}", turn, e))?;
            }
            current_line += 1;
        }

        history.push(sim.create_snapshot());
    }

    serde_wasm_bindgen::to_value(&history).map_err(|e| e.to_string())
}

#[wasm_bindgen]
#[allow(clippy::needless_range_loop)]
pub fn generate_passengers_wasm(seed: u64) -> Result<JsValue, String> {
    let mut rng = Pcg64::seed_from_u64(seed);
    let n = 10;
    let t = 100;
    let lambda = 0.1;
    let poi = Poisson::new(lambda).map_err(|e| e.to_string())?;
    let target_dist = Uniform::new(0, n).map_err(|e| e.to_string())?;

    let mut passenger_source: Vec<Vec<Vec<Passenger>>> = vec![vec![vec![]; t]; n];
    let mut next_passenger_id = 0;

    for i in 0..n {
        for turn in 0..t {
            let count: u32 = poi.sample(&mut rng) as u32;
            for _ in 0..count {
                let mut target = target_dist.sample(&mut rng);
                while target == i {
                    target = target_dist.sample(&mut rng);
                }
                passenger_source[i][turn].push(Passenger {
                    id: next_passenger_id,
                    arrival_turn: turn,
                    target_floor: target,
                });
                next_passenger_id += 1;
            }
        }
    }

    serde_wasm_bindgen::to_value(&passenger_source).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevator_movement() -> Result<()> {
        let mut sim = SimulationState::new(10, 3, 10, 100);
        sim.elevators[0].floor = 5;
        sim.apply_action(0, "UP", &[])?;
        assert_eq!(sim.elevators[0].floor, 6);
        sim.apply_action(0, "DOWN", &[])?;
        assert_eq!(sim.elevators[0].floor, 5);
        Ok(())
    }

    #[test]
    fn test_invalid_elevator_index() {
        let mut sim = SimulationState::new(10, 3, 10, 100);
        assert!(sim.apply_action(3, "UP", &[]).is_err());
    }

    #[test]
    fn test_invalid_passenger_pick() {
        let mut sim = SimulationState::new(10, 3, 10, 100);
        sim.elevators[0].floor = 0;
        assert!(sim.apply_action(0, "OPEN", &[0]).is_err());
    }

    #[test]
    fn test_delivery_score() -> Result<()> {
        let mut sim = SimulationState::new(10, 3, 10, 100);
        sim.turn = 10;
        sim.elevators[0].floor = 1;
        sim.elevators[0].passengers.push(Passenger {
            id: 1,
            arrival_turn: 5,
            target_floor: 1,
        });
        sim.apply_action(0, "OPEN", &[])?;
        // Duration = 10 - 5 + 1 = 6. Score = 6^2 = 36
        assert_eq!(sim.score, 36);
        assert!(sim.elevators[0].passengers.is_empty());
        Ok(())
    }
}

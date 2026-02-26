use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passenger {
    pub id: usize,
    pub arrival_turn: usize,
    pub target_floor: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Elevator {
    pub floor: usize,
    pub passengers: Vec<Passenger>,
    pub capacity: usize,
}

pub struct SimulationState {
    pub n: usize,
    pub m: usize,
    pub c: usize,
    pub t: usize,
    pub elevators: Vec<Elevator>,
    pub waiting_passengers: Vec<Vec<Passenger>>,
    pub turn: usize,
    pub score: u64,
}

impl SimulationState {
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
                        // Just ignore if full (or could bail depending on policy)
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

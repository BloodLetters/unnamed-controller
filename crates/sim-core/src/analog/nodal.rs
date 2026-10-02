use crate::netlist::NodeId;
use std::collections::{HashMap, HashSet};

/// Minimum resistance used to approximate an ideal short (wire) branch.
const MIN_BRANCH_RESISTANCE: f32 = 1e-3;

/// Represents a resistive connection between two circuit nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResistorBranch {
    pub node_a: NodeId,
    pub node_b: NodeId,
    pub resistance_ohms: f32,
}

/// Represents an independent or companion current source branch between two nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurrentSourceBranch {
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub current_amps: f32,
}

/// Linear nodal analysis solver for DC resistor networks and voltage sources.
#[derive(Default, Debug, Clone)]
pub struct NodalSolver {
    fixed_voltages: HashMap<NodeId, f32>,
    resistors: Vec<ResistorBranch>,
    current_sources: Vec<CurrentSourceBranch>,
}

impl NodalSolver {
    /// Creates a new, empty nodal analysis solver.
    pub fn new() -> Self {
        Self::default()
    }

    /// Constrains a node to a fixed voltage potential (e.g. Ground or VCC rail).
    pub fn set_fixed_voltage(&mut self, node: NodeId, voltage: f32) {
        self.fixed_voltages.insert(node, voltage);
    }

    /// Adds a resistor branch between two nodes.
    pub fn add_resistor(&mut self, node_a: NodeId, node_b: NodeId, resistance_ohms: f32) {
        if node_a != node_b {
            self.resistors.push(ResistorBranch {
                node_a,
                node_b,
                resistance_ohms: resistance_ohms.max(MIN_BRANCH_RESISTANCE),
            });
        }
    }

    /// Injects an independent or companion current flowing from from_node to to_node.
    pub fn add_current_source(&mut self, from_node: NodeId, to_node: NodeId, current_amps: f32) {
        if from_node != to_node && current_amps.abs() > 1e-12 {
            self.current_sources.push(CurrentSourceBranch {
                from_node,
                to_node,
                current_amps,
            });
        }
    }

    /// Clears all defined constraints and branches.
    pub fn clear(&mut self) {
        self.fixed_voltages.clear();
        self.resistors.clear();
        self.current_sources.clear();
    }

    /// Solves the resistor network and returns the calculated voltage for each node.
    pub fn solve(&self) -> HashMap<NodeId, f32> {
        let mut node_set = HashSet::new();
        for &node in self.fixed_voltages.keys() {
            node_set.insert(node);
        }
        for r in &self.resistors {
            node_set.insert(r.node_a);
            node_set.insert(r.node_b);
        }
        for cs in &self.current_sources {
            node_set.insert(cs.from_node);
            node_set.insert(cs.to_node);
        }

        let mut nodes: Vec<NodeId> = node_set.into_iter().collect();
        nodes.sort_by_key(|node| node.0);
        let n = nodes.len();
        if n == 0 {
            return HashMap::new();
        }

        let node_to_idx: HashMap<NodeId, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, &node)| (node, i))
            .collect();

        let mut a = vec![vec![0.0_f32; n]; n];
        let mut b = vec![0.0_f32; n];

        for (i, &node) in nodes.iter().enumerate() {
            if let Some(&v) = self.fixed_voltages.get(&node) {
                a[i][i] = 1.0;
                b[i] = v;
            } else {
                let mut net_current = 0.0_f32;
                for cs in &self.current_sources {
                    if cs.to_node == node {
                        net_current += cs.current_amps;
                    } else if cs.from_node == node {
                        net_current -= cs.current_amps;
                    }
                }
                b[i] = net_current;

                let mut total_g = 0.0_f32;
                for r in &self.resistors {
                    let g = 1.0_f32 / r.resistance_ohms;
                    if r.node_a == node {
                        let j = node_to_idx[&r.node_b];
                        a[i][j] -= g;
                        total_g += g;
                    } else if r.node_b == node {
                        let j = node_to_idx[&r.node_a];
                        a[i][j] -= g;
                        total_g += g;
                    }
                }
                if total_g > 0.0 {
                    a[i][i] = total_g;
                } else {
                    a[i][i] = 1.0;
                    b[i] = 0.0;
                }
            }
        }

        let solution = solve_linear_system(&mut a, &mut b);
        let mut results = HashMap::with_capacity(n);
        for (i, &node) in nodes.iter().enumerate() {
            results.insert(node, solution.get(i).copied().unwrap_or(0.0));
        }

        results
    }
}

/// Solves an N x N linear system A * x = B using Gaussian elimination with partial pivoting.
#[allow(clippy::needless_range_loop)]
fn solve_linear_system(a: &mut [Vec<f32>], b: &mut [f32]) -> Vec<f32> {
    let n = b.len();
    let scale = a
        .iter()
        .flat_map(|row| row.iter())
        .fold(0.0_f32, |acc, value| acc.max(value.abs()))
        .max(1.0);
    let epsilon = scale * 1e-12;

    for i in 0..n {
        let mut max_row = i;
        let mut max_val = a[i][i].abs();
        for k in (i + 1)..n {
            let val = a[k][i].abs();
            if val > max_val {
                max_val = val;
                max_row = k;
            }
        }

        if max_val < epsilon {
            continue;
        }

        if max_row != i {
            a.swap(i, max_row);
            b.swap(i, max_row);
        }

        let pivot = a[i][i];
        for k in (i + 1)..n {
            let factor = a[k][i] / pivot;
            a[k][i] = 0.0;
            for j in (i + 1)..n {
                let sub = factor * a[i][j];
                a[k][j] -= sub;
            }
            let b_sub = factor * b[i];
            b[k] -= b_sub;
        }
    }

    let mut x = vec![0.0_f32; n];
    for i in (0..n).rev() {
        if a[i][i].abs() < epsilon {
            x[i] = 0.0;
            continue;
        }
        let mut sum = 0.0_f32;
        for j in (i + 1)..n {
            sum += a[i][j] * x[j];
        }
        x[i] = (b[i] - sum) / a[i][i];
    }

    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voltage_divider_equal_resistors() {
        let mut solver = NodalSolver::new();
        let vcc = NodeId(0);
        let mid = NodeId(1);
        let gnd = NodeId(2);

        solver.set_fixed_voltage(vcc, 5.0);
        solver.set_fixed_voltage(gnd, 0.0);
        solver.add_resistor(vcc, mid, 1000.0);
        solver.add_resistor(mid, gnd, 1000.0);

        let voltages = solver.solve();
        assert_eq!(voltages.get(&vcc).copied(), Some(5.0));
        assert_eq!(voltages.get(&gnd).copied(), Some(0.0));
        let v_mid = voltages.get(&mid).copied().unwrap();
        assert!((v_mid - 2.5).abs() < 1e-4);
    }

    #[test]
    fn test_voltage_divider_unequal_resistors() {
        let mut solver = NodalSolver::new();
        let vcc = NodeId(10);
        let mid = NodeId(11);
        let gnd = NodeId(12);

        solver.set_fixed_voltage(vcc, 10.0);
        solver.set_fixed_voltage(gnd, 0.0);
        solver.add_resistor(vcc, mid, 3000.0);
        solver.add_resistor(mid, gnd, 1000.0);

        let voltages = solver.solve();
        let v_mid = voltages.get(&mid).copied().unwrap();
        assert!((v_mid - 2.5).abs() < 1e-4);
    }

    #[test]
    fn test_three_resistor_ladder() {
        let mut solver = NodalSolver::new();
        let n0 = NodeId(0);
        let n1 = NodeId(1);
        let n2 = NodeId(2);
        let n3 = NodeId(3);

        solver.set_fixed_voltage(n0, 12.0);
        solver.set_fixed_voltage(n3, 0.0);
        solver.add_resistor(n0, n1, 1000.0);
        solver.add_resistor(n1, n2, 1000.0);
        solver.add_resistor(n2, n3, 1000.0);

        let voltages = solver.solve();
        let v1 = voltages.get(&n1).copied().unwrap();
        let v2 = voltages.get(&n2).copied().unwrap();

        assert!((v1 - 8.0).abs() < 1e-4);
        assert!((v2 - 4.0).abs() < 1e-4);
    }

    #[test]
    fn test_current_source_injection() {
        let mut solver = NodalSolver::new();
        let node_a = NodeId(1);
        let gnd = NodeId(0);

        solver.set_fixed_voltage(gnd, 0.0);
        solver.add_resistor(node_a, gnd, 1000.0);
        solver.add_current_source(gnd, node_a, 0.002);

        let voltages = solver.solve();
        let va = voltages.get(&node_a).copied().unwrap();
        assert!((va - 2.0).abs() < 1e-4);
    }
}

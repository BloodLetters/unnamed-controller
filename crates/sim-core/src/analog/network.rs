//! Declarative resistive network assembly from component pins and supply rails.

use std::collections::{HashMap, HashSet};

use crate::analog::nodal::NodalSolver;
use crate::netlist::graph::NetlistGraph;
use crate::netlist::{NodeId, PinId};

/// Accumulates supply rail claims and resistive branches before they reach the nodal solver.
///
/// Multiple sources claiming one node at different potentials represent a drive conflict.
/// Conflicted nodes are released so the digital contention resolver stays authoritative.
#[derive(Default, Debug, Clone)]
pub struct NetworkBuilder {
    rail_claims: HashMap<NodeId, f32>,
    conflicting_nodes: HashSet<NodeId>,
    resistors: Vec<(NodeId, NodeId, f32)>,
}

impl NetworkBuilder {
    /// Creates an empty network builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Claims the node containing a pin as held at a fixed potential by a supply rail.
    pub fn claim_rail_pin(&mut self, netlist: &NetlistGraph, pin: PinId, voltage: f32) {
        if let Some(node) = netlist.get_node_for_pin(pin) {
            self.claim_rail(node, voltage);
        }
    }

    /// Claims a node as held at a fixed potential by a supply rail.
    pub fn claim_rail(&mut self, node: NodeId, voltage: f32) {
        match self.rail_claims.get(&node) {
            Some(&existing) if (existing - voltage).abs() > f32::EPSILON => {
                self.conflicting_nodes.insert(node);
            }
            Some(_) => {}
            None => {
                self.rail_claims.insert(node, voltage);
            }
        }
    }

    /// Registers a resistive branch between the nodes containing two pins.
    pub fn add_resistor_pin(
        &mut self,
        netlist: &NetlistGraph,
        pin_a: PinId,
        pin_b: PinId,
        resistance_ohms: f32,
    ) {
        if let (Some(node_a), Some(node_b)) = (
            netlist.get_node_for_pin(pin_a),
            netlist.get_node_for_pin(pin_b),
        ) {
            self.add_resistor(node_a, node_b, resistance_ohms);
        }
    }

    /// Registers a resistive branch between two resolved nodes.
    pub fn add_resistor(&mut self, node_a: NodeId, node_b: NodeId, resistance_ohms: f32) {
        if node_a != node_b {
            self.resistors.push((node_a, node_b, resistance_ohms));
        }
    }

    /// Returns the number of registered resistive branches.
    pub fn resistor_count(&self) -> usize {
        self.resistors.len()
    }

    /// Returns true when no node carries a drive conflict.
    pub fn is_conflict_free(&self) -> bool {
        self.conflicting_nodes.is_empty()
    }

    /// Clears every accumulated rail claim and resistive branch.
    pub fn clear(&mut self) {
        self.rail_claims.clear();
        self.conflicting_nodes.clear();
        self.resistors.clear();
    }

    /// Transfers the accumulated network into a nodal solver, skipping conflicted nodes.
    pub fn apply_to(&self, solver: &mut NodalSolver) {
        solver.clear();

        for (&node, &voltage) in &self.rail_claims {
            if !self.conflicting_nodes.contains(&node) {
                solver.set_fixed_voltage(node, voltage);
            }
        }

        for &(node_a, node_b, resistance_ohms) in &self.resistors {
            solver.add_resistor(node_a, node_b, resistance_ohms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a netlist with two pins sharing a single node.
    fn netlist_with_shared_node() -> NetlistGraph {
        let mut netlist = NetlistGraph::new();
        let node = netlist.add_node();
        netlist.connect_pin(PinId(0), node);
        netlist.connect_pin(PinId(1), node);
        netlist
    }

    #[test]
    fn test_matching_rail_claims_are_agreed() {
        let mut builder = NetworkBuilder::new();
        builder.claim_rail(NodeId(0), 3.3);
        builder.claim_rail(NodeId(0), 3.3);

        assert!(builder.is_conflict_free());

        let mut solver = NodalSolver::new();
        builder.apply_to(&mut solver);
        assert_eq!(solver.solve().get(&NodeId(0)).copied(), Some(3.3));
    }

    #[test]
    fn test_diverging_rail_claims_release_the_node() {
        let mut builder = NetworkBuilder::new();
        builder.claim_rail(NodeId(0), 3.3);
        builder.claim_rail(NodeId(0), 5.0);

        assert!(!builder.is_conflict_free());

        let mut solver = NodalSolver::new();
        builder.apply_to(&mut solver);
        assert!(solver.solve().is_empty());
    }

    #[test]
    fn test_resistor_branch_solves_a_divider() {
        let mut builder = NetworkBuilder::new();
        builder.claim_rail(NodeId(0), 5.0);
        builder.claim_rail(NodeId(2), 0.0);
        builder.add_resistor(NodeId(0), NodeId(1), 1000.0);
        builder.add_resistor(NodeId(1), NodeId(2), 1000.0);

        let mut solver = NodalSolver::new();
        builder.apply_to(&mut solver);

        let voltages = solver.solve();
        assert!((voltages.get(&NodeId(1)).copied().unwrap() - 2.5).abs() < 1e-4);
    }

    #[test]
    fn test_claim_rail_pin_resolves_through_netlist() {
        let netlist = netlist_with_shared_node();
        let mut builder = NetworkBuilder::new();
        builder.claim_rail_pin(&netlist, PinId(0), 3.3);
        builder.claim_rail_pin(&netlist, PinId(1), 3.3);

        assert!(builder.is_conflict_free());

        let mut solver = NodalSolver::new();
        builder.apply_to(&mut solver);
        assert_eq!(solver.solve().get(&NodeId(0)).copied(), Some(3.3));
    }

    #[test]
    fn test_add_resistor_pin_skips_unconnected_pins() {
        let netlist = netlist_with_shared_node();
        let mut builder = NetworkBuilder::new();
        builder.add_resistor_pin(&netlist, PinId(0), PinId(1), 220.0);
        builder.add_resistor_pin(&netlist, PinId(0), PinId(99), 220.0);

        let mut solver = NodalSolver::new();
        builder.apply_to(&mut solver);
        assert!(solver.solve().is_empty());
    }

    #[test]
    fn test_clear_resets_accumulated_network() {
        let mut builder = NetworkBuilder::new();
        builder.claim_rail(NodeId(0), 3.3);
        builder.add_resistor(NodeId(0), NodeId(1), 220.0);
        builder.clear();

        assert!(builder.is_conflict_free());

        let mut solver = NodalSolver::new();
        builder.apply_to(&mut solver);
        assert!(solver.solve().is_empty());
    }
}

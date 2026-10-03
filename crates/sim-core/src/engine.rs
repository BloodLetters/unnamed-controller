use crate::analog::network::NetworkBuilder;
use crate::analog::nodal::NodalSolver;
use crate::clock::scheduler::{EventId, EventScheduler, SimTime};
use crate::netlist::graph::NetlistGraph;
use crate::netlist::{NodeId, PinId};
use crate::pins::{AnalogVoltage, DigitalState, PinDrive, PinState, resolve_node_pin_state};
use std::collections::HashMap;

/// Nominal potential applied to a node held high by a strong digital driver.
const HIGH_LOGIC_VOLTAGE: f32 = 3.3;

/// Nominal potential applied to a node held low by a strong digital driver.
const LOW_LOGIC_VOLTAGE: f32 = 0.0;

/// Event payload for scheduled simulation engine actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineEvent {
    SetPinDrive { pin: PinId, drive: PinDrive },
    EvaluateNetlist,
    AdvanceClock,
}

/// The main simulation engine coordinating netlist state, events, and electrical solvers.
pub struct Engine {
    pub netlist: NetlistGraph,
    pub node_states: HashMap<NodeId, PinState>,
    pub node_voltages: HashMap<NodeId, f32>,
    pub nodal_solver: NodalSolver,
    pub network: NetworkBuilder,
    pub scheduler: EventScheduler<EngineEvent>,
    pub ticks: u64,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            netlist: NetlistGraph::new(),
            node_states: HashMap::new(),
            node_voltages: HashMap::new(),
            nodal_solver: NodalSolver::new(),
            network: NetworkBuilder::new(),
            scheduler: EventScheduler::new(),
            ticks: 0,
        }
    }
}

impl Engine {
    /// Creates a new simulation engine instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates the electrical state of all nodes using driver strength and pull resistor resolution.
    pub fn evaluate_netlist_drives(&mut self, driving_drives: &HashMap<PinId, PinDrive>) {
        for (&node_id, connected_pins) in self.netlist.get_all_nodes() {
            let drives = connected_pins
                .iter()
                .filter_map(|pin_id| driving_drives.get(pin_id).copied());

            let resolved = resolve_node_pin_state(drives);
            self.node_states.insert(node_id, resolved);
        }
    }

    /// Evaluates the electrical state of all nodes from simple digital pin states.
    pub fn evaluate_netlist(&mut self, driving_pins: &HashMap<PinId, PinState>) {
        let drives: HashMap<PinId, PinDrive> = driving_pins
            .iter()
            .map(|(&pin, &state)| {
                let drive = match state {
                    PinState::Digital(DigitalState::High) => PinDrive::Driven(DigitalState::High),
                    PinState::Digital(DigitalState::Low) => PinDrive::Driven(DigitalState::Low),
                    PinState::Digital(DigitalState::HighZ) => PinDrive::HighZ,
                    PinState::Analog(_) => PinDrive::HighZ,
                };
                (pin, drive)
            })
            .collect();

        self.evaluate_netlist_drives(&drives);
    }

    /// Solves the resistive network and refreshes node voltages and analog node states.
    ///
    /// A node held by a strong digital driver keeps its logic potential rather than the solved
    /// value, so a driven output is never dragged down by current drawn through a series resistor.
    pub fn solve_analog_network(&mut self) {
        self.network.apply_to(&mut self.nodal_solver);

        let mut solved = self.nodal_solver.solve();

        for (&node_id, &state) in &self.node_states {
            let driven_voltage = match state {
                PinState::Digital(DigitalState::High) => Some(HIGH_LOGIC_VOLTAGE),
                PinState::Digital(DigitalState::Low) => Some(LOW_LOGIC_VOLTAGE),
                _ => None,
            };
            if let Some(voltage) = driven_voltage {
                solved.insert(node_id, voltage);
            }
        }

        for (&node_id, &voltage) in &solved {
            let state = self
                .node_states
                .get(&node_id)
                .copied()
                .unwrap_or(PinState::Digital(DigitalState::HighZ));
            if !matches!(
                state,
                PinState::Digital(DigitalState::High | DigitalState::Low)
            ) {
                self.node_states
                    .insert(node_id, PinState::Analog(AnalogVoltage(voltage)));
            }
        }

        self.node_voltages = solved;
    }

    /// Discards every accumulated rail claim, resistive branch, and solved node voltage.
    pub fn clear_network(&mut self) {
        self.network.clear();
        self.nodal_solver.clear();
        self.node_voltages.clear();
    }

    /// Gets the resolved logical state of a netlist node.
    pub fn get_node_state(&self, node_id: NodeId) -> PinState {
        self.node_states
            .get(&node_id)
            .copied()
            .unwrap_or(PinState::Digital(DigitalState::HighZ))
    }

    /// Gets the solved analog voltage of a node in volts.
    pub fn get_node_voltage(&self, node_id: NodeId) -> f32 {
        self.node_voltages.get(&node_id).copied().unwrap_or(0.0)
    }

    /// Schedules a future simulation engine event.
    pub fn schedule_event(&mut self, delta_ticks: u64, event: EngineEvent) -> EventId {
        self.scheduler.schedule_after(delta_ticks, event)
    }

    /// Advances the simulation clock and processes all due scheduled events.
    pub fn tick(&mut self) {
        self.ticks += 1;
        self.scheduler.advance_time(1);

        let due_events = self.scheduler.pop_due(SimTime(self.ticks));
        for (_id, event) in due_events {
            match event {
                EngineEvent::SetPinDrive { pin, drive } => {
                    if let Some(node_id) = self.netlist.get_node_for_pin(pin) {
                        self.node_states
                            .insert(node_id, resolve_node_pin_state(std::iter::once(drive)));
                    }
                }
                EngineEvent::EvaluateNetlist | EngineEvent::AdvanceClock => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an engine where two pins share one node.
    fn engine_with_shared_node() -> (Engine, NodeId) {
        let mut engine = Engine::new();
        let shared = engine.netlist.add_node();
        engine.netlist.connect_pin(PinId(0), shared);
        engine.netlist.connect_pin(PinId(1), shared);
        (engine, shared)
    }

    #[test]
    fn test_rail_claim_establishes_node_voltage() {
        let (mut engine, shared) = engine_with_shared_node();
        engine
            .network
            .claim_rail_pin(&engine.netlist, PinId(0), 3.3);
        engine.solve_analog_network();

        assert_eq!(engine.get_node_voltage(shared), 3.3);
    }

    #[test]
    fn test_resistive_divider_between_two_rails() {
        let mut engine = Engine::new();
        let vcc = engine.netlist.add_node();
        let mid = engine.netlist.add_node();
        let gnd = engine.netlist.add_node();

        engine.network.claim_rail(vcc, 5.0);
        engine.network.claim_rail(gnd, 0.0);
        engine.network.add_resistor(vcc, mid, 1000.0);
        engine.network.add_resistor(mid, gnd, 1000.0);
        engine.solve_analog_network();

        assert!((engine.get_node_voltage(mid) - 2.5).abs() < 1e-4);
    }

    #[test]
    fn test_floating_node_becomes_analog_state() {
        let mut engine = Engine::new();
        let vcc = engine.netlist.add_node();
        let mid = engine.netlist.add_node();
        let gnd = engine.netlist.add_node();

        engine.netlist.connect_pin(PinId(0), vcc);
        engine.netlist.connect_pin(PinId(1), mid);
        engine.netlist.connect_pin(PinId(2), gnd);

        engine.network.claim_rail(vcc, 5.0);
        engine.network.claim_rail(gnd, 0.0);
        engine.network.add_resistor(vcc, mid, 1000.0);
        engine.network.add_resistor(mid, gnd, 10_000.0);
        engine.solve_analog_network();

        let mid_voltage = engine.get_node_voltage(mid);
        assert!((mid_voltage - 4.545).abs() < 1e-2);
        assert_eq!(
            engine.get_node_state(mid),
            PinState::Analog(AnalogVoltage(mid_voltage))
        );
    }

    #[test]
    fn test_strong_driver_holds_its_logic_potential() {
        let (mut engine, shared) = engine_with_shared_node();
        let sink = engine.netlist.add_node();

        engine.netlist.connect_pin(PinId(2), sink);
        engine.network.claim_rail(sink, 0.0);
        engine.network.add_resistor(shared, sink, 220.0);

        let mut drives = HashMap::new();
        drives.insert(PinId(0), PinDrive::Driven(DigitalState::High));
        engine.evaluate_netlist_drives(&drives);
        engine.solve_analog_network();

        assert_eq!(engine.get_node_voltage(shared), HIGH_LOGIC_VOLTAGE);
    }

    #[test]
    fn test_conflicting_rail_claims_are_not_pinned() {
        let (mut engine, shared) = engine_with_shared_node();
        engine
            .network
            .claim_rail_pin(&engine.netlist, PinId(0), 3.3);
        engine
            .network
            .claim_rail_pin(&engine.netlist, PinId(1), 5.0);
        engine.solve_analog_network();

        assert!(!engine.network.is_conflict_free());
        assert_eq!(engine.get_node_voltage(shared), 0.0);
    }

    #[test]
    fn test_clear_network_drops_solved_voltages() {
        let (mut engine, shared) = engine_with_shared_node();
        engine
            .network
            .claim_rail_pin(&engine.netlist, PinId(0), 3.3);
        engine.solve_analog_network();
        assert_eq!(engine.get_node_voltage(shared), 3.3);

        engine.clear_network();
        assert_eq!(engine.get_node_voltage(shared), 0.0);
    }
}

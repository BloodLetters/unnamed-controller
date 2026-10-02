use crate::analog::nodal::NodalSolver;
use crate::clock::scheduler::{EventId, EventScheduler, SimTime};
use crate::netlist::graph::NetlistGraph;
use crate::netlist::{NodeId, PinId};
use crate::pins::{AnalogVoltage, DigitalState, PinDrive, PinState, resolve_node_pin_state};
use std::collections::HashMap;

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

    /// Solves DC nodal voltages for any analog resistor networks in the netlist.
    pub fn solve_analog_network(&mut self) {
        self.node_voltages = self.nodal_solver.solve();
        for (&node_id, &voltage) in &self.node_voltages {
            if self.node_states.get(&node_id) == Some(&PinState::Digital(DigitalState::HighZ)) {
                self.node_states
                    .insert(node_id, PinState::Analog(AnalogVoltage(voltage)));
            }
        }
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

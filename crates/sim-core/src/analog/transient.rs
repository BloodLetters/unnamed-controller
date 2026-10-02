use crate::netlist::NodeId;

/// State tracker for a transient capacitor companion model using Backward Euler integration.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CapacitorCompanion {
    pub node_a: NodeId,
    pub node_b: NodeId,
    pub capacitance_farads: f32,
    pub v_prev: f32,
    pub current_amps: f32,
}

impl CapacitorCompanion {
    /// Constructs a capacitor companion model with initial zero charge.
    pub fn new(node_a: NodeId, node_b: NodeId, capacitance_farads: f32) -> Self {
        Self {
            node_a,
            node_b,
            capacitance_farads,
            v_prev: 0.0,
            current_amps: 0.0,
        }
    }

    /// Calculates equivalent companion parallel resistance for time step dt.
    pub fn equivalent_resistance(&self, dt_sec: f32) -> f32 {
        (dt_sec / self.capacitance_farads.max(1e-12)).max(1e-6)
    }

    /// Calculates companion history current source injected into node_a from node_b.
    pub fn companion_current(&self, dt_sec: f32) -> f32 {
        (self.capacitance_farads / dt_sec.max(1e-9)) * self.v_prev
    }

    /// Updates stored capacitor voltage and current from the solved nodal potentials.
    pub fn update(&mut self, dt_sec: f32, v_a: f32, v_b: f32) {
        let v_new = v_a - v_b;
        self.current_amps = (self.capacitance_farads / dt_sec.max(1e-9)) * (v_new - self.v_prev);
        self.v_prev = v_new;
    }

    /// Returns the stored electrostatic energy in Joules ($E = \frac{1}{2} C V^2$).
    pub fn stored_energy_joules(&self) -> f32 {
        0.5 * self.capacitance_farads * self.v_prev * self.v_prev
    }
}

/// State tracker for a transient inductor companion model using Backward Euler integration.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InductorCompanion {
    pub node_a: NodeId,
    pub node_b: NodeId,
    pub inductance_henrys: f32,
    pub i_prev: f32,
    pub voltage_volts: f32,
}

impl InductorCompanion {
    /// Constructs an inductor companion model with initial zero current.
    pub fn new(node_a: NodeId, node_b: NodeId, inductance_henrys: f32) -> Self {
        Self {
            node_a,
            node_b,
            inductance_henrys,
            i_prev: 0.0,
            voltage_volts: 0.0,
        }
    }

    /// Calculates equivalent companion parallel resistance for time step dt.
    pub fn equivalent_resistance(&self, dt_sec: f32) -> f32 {
        (self.inductance_henrys / dt_sec.max(1e-9)).max(1e-6)
    }

    /// Calculates companion history current source for time step dt.
    pub fn companion_current(&self) -> f32 {
        self.i_prev
    }

    /// Updates stored inductor current and back-EMF voltage from solved nodal potentials.
    pub fn update(&mut self, dt_sec: f32, v_a: f32, v_b: f32) {
        let v_new = v_a - v_b;
        self.voltage_volts = v_new;
        let di = (v_new * dt_sec) / self.inductance_henrys.max(1e-9);
        self.i_prev += di;
    }

    /// Returns the stored magnetic energy in Joules ($E = \frac{1}{2} L I^2$).
    pub fn stored_energy_joules(&self) -> f32 {
        0.5 * self.inductance_henrys * self.i_prev * self.i_prev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capacitor_charge_step() {
        let n_a = NodeId(1);
        let n_b = NodeId(0);
        let mut cap = CapacitorCompanion::new(n_a, n_b, 0.001);
        let dt = 0.001;

        assert!((cap.equivalent_resistance(dt) - 1.0).abs() < 1e-4);
        assert_eq!(cap.companion_current(dt), 0.0);

        cap.update(dt, 5.0, 0.0);
        assert_eq!(cap.v_prev, 5.0);
        assert_eq!(cap.companion_current(dt), 5.0);
        assert!((cap.stored_energy_joules() - 0.0125).abs() < 1e-4);
    }

    #[test]
    fn test_inductor_current_ramp() {
        let n_a = NodeId(1);
        let n_b = NodeId(0);
        let mut ind = InductorCompanion::new(n_a, n_b, 0.01);
        let dt = 0.001;

        assert!((ind.equivalent_resistance(dt) - 10.0).abs() < 1e-4);
        ind.update(dt, 10.0, 0.0);
        assert!((ind.i_prev - 1.0).abs() < 1e-4);
        assert!((ind.stored_energy_joules() - 0.005).abs() < 1e-4);
    }

    #[test]
    fn test_rc_circuit_transient_charging() {
        use crate::analog::nodal::NodalSolver;

        let n_vcc = NodeId(1);
        let n_cap = NodeId(2);
        let n_gnd = NodeId(0);

        let mut cap = CapacitorCompanion::new(n_cap, n_gnd, 100e-6);
        let r_series = 1000.0_f32;
        let dt = 0.001_f32;

        for _ in 0..100 {
            let mut solver = NodalSolver::new();
            solver.set_fixed_voltage(n_vcc, 5.0);
            solver.set_fixed_voltage(n_gnd, 0.0);
            solver.add_resistor(n_vcc, n_cap, r_series);
            solver.add_resistor(n_cap, n_gnd, cap.equivalent_resistance(dt));
            solver.add_current_source(n_gnd, n_cap, cap.companion_current(dt));

            let results = solver.solve();
            let v_cap = results[&n_cap];
            cap.update(dt, v_cap, 0.0);
        }

        assert!((cap.v_prev - 3.16).abs() < 0.05);
    }
}

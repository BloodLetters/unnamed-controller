use std::collections::HashMap;

/// Unique identifier for a circuit node (net/wire).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeId(pub usize);

/// Unique identifier for a component pin in the netlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PinId(pub usize);

/// Defines a connection between a pin and a node.
#[derive(Debug, Clone)]
pub struct Connection {
    pub pin: PinId,
    pub node: NodeId,
}

/// Manages the electrical connections between pins and nodes.
#[derive(Debug, Default)]
pub struct NetlistGraph {
    nodes: HashMap<NodeId, Vec<PinId>>,
    pins: HashMap<PinId, NodeId>,
    next_node_id: usize,
}

impl NetlistGraph {
    /// Creates a new, empty netlist graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new node to the graph and returns its ID.
    pub fn add_node(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        self.nodes.insert(id, Vec::new());
        id
    }

    /// Connects a pin to a specific node, detaching it from any previous node.
    pub fn connect_pin(&mut self, pin: PinId, node: NodeId) {
        if let Some(previous_node) = self.pins.insert(pin, node)
            && previous_node != node
            && let Some(previous_pins) = self.nodes.get_mut(&previous_node)
        {
            previous_pins.retain(|&connected| connected != pin);
        }
        if let Some(connected_pins) = self.nodes.get_mut(&node).filter(|cp| !cp.contains(&pin)) {
            connected_pins.push(pin);
        }
    }

    /// Retrieves all pins connected to a specific node.
    pub fn get_connected_pins(&self, node: NodeId) -> Option<&Vec<PinId>> {
        self.nodes.get(&node)
    }

    /// Retrieves the node a specific pin is connected to.
    pub fn get_node_for_pin(&self, pin: PinId) -> Option<NodeId> {
        self.pins.get(&pin).copied()
    }

    /// Retrieves all nodes in the graph.
    pub fn get_all_nodes(&self) -> &HashMap<NodeId, Vec<PinId>> {
        &self.nodes
    }
}

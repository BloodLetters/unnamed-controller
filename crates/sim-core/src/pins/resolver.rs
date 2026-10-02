use super::types::{DigitalState, DriveStrength, PinDrive, PinState};

/// Resolves the collective digital state of a net from multiple driving pin sources.
pub fn resolve_node_digital_state<I>(drives: I) -> DigitalState
where
    I: IntoIterator<Item = PinDrive>,
{
    let mut has_strong_high = false;
    let mut has_strong_low = false;
    let mut has_weak_high = false;
    let mut has_weak_low = false;

    for drive in drives {
        match drive.strength() {
            DriveStrength::Strong => match drive.to_digital_state() {
                DigitalState::High => has_strong_high = true,
                DigitalState::Low => has_strong_low = true,
                DigitalState::HighZ => {}
            },
            DriveStrength::Weak => match drive.to_digital_state() {
                DigitalState::High => has_weak_high = true,
                DigitalState::Low => has_weak_low = true,
                DigitalState::HighZ => {}
            },
            DriveStrength::HighZ => {}
        }
    }

    if has_strong_high && has_strong_low {
        return DigitalState::Low;
    }
    if has_strong_high {
        return DigitalState::High;
    }
    if has_strong_low {
        return DigitalState::Low;
    }

    if has_weak_high && has_weak_low {
        return DigitalState::Low;
    }
    if has_weak_high {
        return DigitalState::High;
    }
    if has_weak_low {
        return DigitalState::Low;
    }

    DigitalState::HighZ
}

/// Resolves multiple pin states and drives into an overall pin state.
pub fn resolve_node_pin_state<I>(drives: I) -> PinState
where
    I: IntoIterator<Item = PinDrive>,
{
    PinState::Digital(resolve_node_digital_state(drives))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_node_resolves_to_highz() {
        let drives = vec![PinDrive::HighZ, PinDrive::HighZ];
        assert_eq!(resolve_node_digital_state(drives), DigitalState::HighZ);
    }

    #[test]
    fn test_pull_up_pulls_floating_node_high() {
        let drives = vec![PinDrive::HighZ, PinDrive::PullUp];
        assert_eq!(resolve_node_digital_state(drives), DigitalState::High);
    }

    #[test]
    fn test_pull_down_pulls_floating_node_low() {
        let drives = vec![PinDrive::HighZ, PinDrive::PullDown];
        assert_eq!(resolve_node_digital_state(drives), DigitalState::Low);
    }

    #[test]
    fn test_strong_driver_overpowers_pull_resistor() {
        let drives1 = vec![PinDrive::Driven(DigitalState::Low), PinDrive::PullUp];
        assert_eq!(resolve_node_digital_state(drives1), DigitalState::Low);

        let drives2 = vec![PinDrive::Driven(DigitalState::High), PinDrive::PullDown];
        assert_eq!(resolve_node_digital_state(drives2), DigitalState::High);
    }

    #[test]
    fn test_open_drain_bus_behavior() {
        let bus_idle = vec![PinDrive::PullUp, PinDrive::HighZ, PinDrive::HighZ];
        assert_eq!(resolve_node_digital_state(bus_idle), DigitalState::High);

        let bus_pulled = vec![
            PinDrive::PullUp,
            PinDrive::Driven(DigitalState::Low),
            PinDrive::HighZ,
        ];
        assert_eq!(resolve_node_digital_state(bus_pulled), DigitalState::Low);
    }
}

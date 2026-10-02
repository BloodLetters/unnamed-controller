use sim_components::component::{Led, LedColor};
use sim_core::component::Component;
use sim_core::netlist::PinId;

#[test]
fn test_led_creation_and_pins() {
    let led = Led::new(PinId(101), PinId(102), LedColor::Blue);
    assert_eq!(led.anode(), PinId(101));
    assert_eq!(led.cathode(), PinId(102));
    assert_eq!(led.pins(), vec![PinId(101), PinId(102)]);
    assert_eq!(led.color, LedColor::Blue);
    assert!(!led.is_lit());
}

#[test]
fn test_led_all_color_presets_conduction() {
    for color in LedColor::ALL {
        let mut led = Led::new(PinId(1), PinId(2), color);
        let vf = color.forward_voltage();

        led.update_electrical(vf - 0.1, 0.0);
        assert!(!led.is_lit());

        led.update_electrical(vf + 0.2, 0.0);
        assert!(led.is_lit());
        assert!(led.brightness() > 0.0);
    }
}

#[test]
fn test_led_reverse_bias() {
    let mut led = Led::new(PinId(1), PinId(2), LedColor::Green);
    led.update_electrical(0.0, 5.0);
    assert!(!led.is_lit());
    assert_eq!(led.brightness(), 0.0);
}

#[test]
fn test_led_polarity_reversal() {
    let mut led = Led::new(PinId(5), PinId(6), LedColor::Red);
    assert_eq!(led.anode(), PinId(5));
    assert_eq!(led.cathode(), PinId(6));

    led.flip_polarity();
    assert_eq!(led.anode(), PinId(6));
    assert_eq!(led.cathode(), PinId(5));
    assert_eq!(led.pins(), vec![PinId(6), PinId(5)]);
}

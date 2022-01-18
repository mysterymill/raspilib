use std::collections::HashMap;

use super::gpiopins::GpioPins;

type PortState = HashMap<GpioPins, gpio::GpioValue>;
type ChangeCallback = fn(before: PortState, now: PortState);

pub struct PinManager<'p> {
    pin_outputs: Vec<&'p OutputPort>,
    pin_input: Vec<&'p InputPort>,
}

pub struct OutputPort {
    state: PortState,
}

pub struct InputPort {
    state: PortState,
    change_callback: ChangeCallback,
}

#[cfg(test)]
mod test {

}
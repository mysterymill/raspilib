use std::collections::HashMap;

use super::gpiopins::GpioPins;

type PortState = HashMap<GpioPins, gpio::GpioValue>;
type ChangeCallback = fn(before: PortState, now: PortState);

pub struct PinManager<'p> {
    // Not sure yet, but it might be correct to take ownership here
    output_ports: Vec<&'p OutputPort>,
    input_ports: Vec<&'p InputPort>,
}

pub struct OutputPort {
    state: PortState,
}

pub struct InputPort {
    state: PortState,
    change_callback: ChangeCallback,
}

lazy_static! {
    static ref PINMANAGER: PinManager<'static> = PinManager::new(); 
}

impl <'p> PinManager<'p> {
    fn new() -> PinManager<'p> {
        PinManager {
            input_ports: vec![],
            output_ports: vec![],
        }
    }
}

#[cfg(test)]
mod test {

}
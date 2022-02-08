use std::sync::Arc;

use super::pinmanager::{InputPort, OutputPort, PortFrame, PortDefinition, MismatchingPinsError, PinManager, self, PINMANAGER, Port};

#[derive(Debug)]
pub struct MatrixOutput<const I: usize, const O: usize> {
    input_port: Arc<InputPort<I>>,
    output_port: Arc<OutputPort<O>>,

    state: [PortFrame<O>; I],
}

impl <const I: usize, const O: usize> MatrixOutput<I, O> {
    pub fn new(input_pins: &PortDefinition<I>, output_pins: &PortDefinition<O>) -> Result<MatrixOutput<I, O>, MismatchingPinsError> {
        let mut pin_manager = PINMANAGER.lock().unwrap();

        let input_port = pin_manager.register_InputPort(input_pins)?;
        let output_port = pin_manager.register_OutputPort(output_pins)?;
        let state = [output_port.get_PortFrame().clone(); I];

        Ok(MatrixOutput{input_port, output_port, state})
    }
}

#[cfg(test)]
mod test {
    use crate::gpio::{pinmanager::PINMANAGER, gpiopins::GpioPins::*};

    use super::MatrixOutput;

    #[test]
    fn new_MatrixOutput_ok() {
        {
            let mut pinmanager = PINMANAGER.lock().unwrap();
            pinmanager.clear();
        }

        let matrix_output_result = MatrixOutput::new(&[GPIO_12, GPIO_06, GPIO_09], &[GPIO_04, GPIO_24, GPIO_19, GPIO_15]);
        assert!(matrix_output_result.is_ok());
    }

    #[test]
    fn new_MatrixOutput_conflict_in_port_fail() {
        {
            let mut pinmanager = PINMANAGER.lock().unwrap();
            pinmanager.clear();
        }

        let matrix_output_result = MatrixOutput::new(&[GPIO_12, GPIO_06, GPIO_12], &[GPIO_04, GPIO_15]);
        assert!(matrix_output_result.is_err());
        assert!(matrix_output_result.unwrap_err().1.contains(&GPIO_12))
    }

    #[test]
    fn new_MatrixOutput_conflict_between_ports_fail() {
        {
            let mut pinmanager = PINMANAGER.lock().unwrap();
            pinmanager.clear();
        }

        let matrix_output_result = 
            MatrixOutput::new(&[GPIO_12, GPIO_06, GPIO_11, GPIO_03], &[GPIO_12, GPIO_11, GPIO_03]);
        assert!(matrix_output_result.is_err());

        let conflict_pins = matrix_output_result.unwrap_err().1;
        assert!(conflict_pins.contains(&GPIO_03));
        assert!(conflict_pins.contains(&GPIO_11));
        assert!(conflict_pins.contains(&GPIO_12));
    }
}
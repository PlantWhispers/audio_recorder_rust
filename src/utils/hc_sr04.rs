/// HcSr04 Sound Emitter Module
///
/// Provides functionality to control the HC-SR04 sound emitter, particularly for triggering sound emissions.
use rppal::gpio::{Gpio, OutputPin};
use std::{thread, time::Duration};

pub struct HcSr04SoundEmitter {
    trigger: OutputPin,
}

impl HcSr04SoundEmitter {
    pub fn new(trigger_pin: u8) -> Self {
        let gpio = Gpio::new().unwrap();
        let mut trigger = gpio.get(trigger_pin).unwrap().into_output();
        trigger.set_low();
        Self { trigger }
    }

    pub fn emit_sound(&mut self) {
        self.trigger.set_high();
        thread::sleep(Duration::from_micros(10)); //TODO: Find a way to do this without sleeping
        self.trigger.set_low();
    }
}

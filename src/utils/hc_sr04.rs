/// HcSr04 Sound Emitter Module
///
/// Provides functionality to control the HC-SR04 sound emitter, particularly for triggering sound emissions.
use rppal::gpio::{Error, Gpio, OutputPin};
use std::{thread, time::Duration};

pub trait SoundEmitter {
    fn emit_sound(&mut self);
}

pub struct HcSr04SoundEmitter {
    trigger: OutputPin,
}

impl HcSr04SoundEmitter {
    pub fn new(trigger_pin: u8) -> Result<Self, Error> {
        let gpio = Gpio::new()?;
        let mut trigger = gpio.get(trigger_pin).unwrap().into_output();
        trigger.set_low();
        Ok(Self { trigger })
    }
}

impl SoundEmitter for HcSr04SoundEmitter {
    fn emit_sound(&mut self) {
        self.trigger.set_high();
        thread::sleep(Duration::from_micros(10));
        self.trigger.set_low();
    }
}

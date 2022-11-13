use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub struct AudioGraph {
    nodes: Vec<Box<dyn AudioNode + Send>>,
    pub output: Output,
    sample_rate: u32,
}

impl AudioGraph {
    pub fn new(sample_rate: u32) -> Self {
        AudioGraph {
            nodes: Vec::new(),
            output: Output::new(),
            sample_rate,
        }
    }
    pub fn process(&mut self, data: &mut [f32]) {
        self.output.process(data);
    }
}

pub trait AudioNode {
    // fn connect(&self, target: AudioNodeTarget);
    fn add_input(&mut self, input: AudioNodeTarget);
    fn process(&mut self, data: &mut [f32]) {
        // noop
    }
}

type AudioNodeTarget = Box<dyn AudioNode + Send>;
type AudioGraphRef = AudioGraph;

#[derive(Debug)]
pub enum OscWave {
    Triangle,
    Sine,
    // Square,
}

pub struct Osc {
    freq: f32,
    wave: OscWave,
    data: AudioNodeData,
    sample_clock: f32,
    sample_rate: u32,
}

impl Osc {
    pub fn new(sample_rate: u32, freq: f32, wave: OscWave) -> Osc {
        Osc {
            freq,
            wave,
            data: AudioNodeData::default(),
            sample_clock: 0.0,
            sample_rate,
        }
    }
}

impl AudioNode for Osc {
    fn process(&mut self, data: &mut [f32]) {
        for sample in data {
            let sample_rate = self.sample_rate as f32;

            self.sample_clock = (self.sample_clock + 1.0) % sample_rate;
            let time_secs = self.sample_clock / sample_rate;

            // let mut noise: i16 = rng.gen(); // generates a float between 0 and 1
            // noise /= 20;

            // let sin = (time_secs * freq * PI * 2.0).sin() / 20.0;

            let tri = ((((time_secs * self.freq) % 1.0) - 0.5).abs() - 0.25) * 4.0;

            *sample = tri / 20.0;
            // *sample = Sample::from(&(tri / 20.0));
        }
        // done
    }
    fn add_input(&mut self, input: AudioNodeTarget) {
        self.data.inputs.push(input);
    }
}

impl Debug for Osc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.freq)
            .field(&self.wave)
            .finish()
    }
}

pub struct Gain {
    amount: f32,
    data: AudioNodeData,
}

impl Gain {
    pub fn new(amount: f32) -> Gain {
        Gain {
            amount,
            data: AudioNodeData::default(),
        }
    }
}

impl AudioNode for Gain {
    fn process(&mut self, data: &mut [f32]) {
        if let Some(input) = self.data.inputs.first_mut() {
            input.process(data);

            for sample in data {
                *sample *= self.amount;
            }
            // done
        }
    }
    fn add_input(&mut self, input: AudioNodeTarget) {
        self.data.inputs.push(input);
    }
}

pub struct Output {
    data: AudioNodeData,
}

impl Output {
    fn new() -> Self {
        Self {
            data: AudioNodeData::default(),
        }
    }
}

impl AudioNode for Output {
    fn process(&mut self, data: &mut [f32]) {
        if let Some(input) = self.data.inputs.first_mut() {
            input.process(data);
            // done
        }
    }
    fn add_input(&mut self, input: AudioNodeTarget) {
        self.data.inputs.push(input);
    }
}

#[derive(Default)]
struct AudioNodeData {
    inputs: Vec<AudioNodeTarget>,
}

pub fn create_graph(sample_rate: u32) -> AudioGraph {
    let mut audio_graph = AudioGraph::new(sample_rate);
    let osc = Box::new(Osc::new(audio_graph.sample_rate, 440.0, OscWave::Triangle));
    let mut gain = Box::new(Gain::new(0.5));
    // osc.connect(gain);
    // gain.connect(&audio_graph.output);
    gain.add_input(osc);
    audio_graph.output.add_input(gain);
    let mut data = [0_f32; 480];
    audio_graph.process(&mut data);
    audio_graph
}

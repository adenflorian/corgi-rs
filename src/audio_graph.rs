use std::{
    cell::RefCell,
    f32::consts::PI,
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex},
};

use rand::{thread_rng, Rng};

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
    phasor: f32,
    sample_rate: u32,
    s: f32,
}

impl Osc {
    pub fn new(sample_rate: u32, freq: f32, wave: OscWave) -> Osc {
        Osc {
            freq,
            wave,
            data: AudioNodeData::default(),
            sample_clock: 0.0,
            sample_rate,
            s: 1.0 / sample_rate as f32,
            phasor: 0.0,
        }
    }
}

impl AudioNode for Osc {
    fn process(&mut self, data: &mut [f32]) {
        let mut rng = thread_rng();
        let sample_rate = self.sample_rate as f32;
        let p = 1. / self.freq;
        // dbg!(self.phasor);
        for sample in data {
            // self.sample_clock = (self.sample_clock + 1.0) % sample_rate;
            // let time_secs = self.sample_clock / sample_rate;
            // self.sample_clock = (self.sample_clock + 1.0) % self.freq;
            // let time_secs = self.sample_clock / sample_rate;
            self.phasor = (self.phasor + (self.s / p)) % 1.0;

            // let mut noise: f32 = rng.gen::<f32>() - 0.5; // generates a float between 0 and 1
            // noise /= 20;

            // let sin = (time_secs * self.freq * PI * 2.0).sin();

            // let tri = ((((phasor * self.freq) % 1.0) - 0.5).abs() - 0.25) * 4.0;

            // let ftime = self.sample_clock / self.freq;
            let tri2 = triangle(self.phasor, 0.0);

            // *sample = noise / 20.0;
            // *sample = sin / 20.0;
            *sample = tri2 / 10.0;
            // *sample = Sample::from(&(tri / 20.0));
        }
        // done
    }
    fn add_input(&mut self, input: AudioNodeTarget) {}
}

fn triangle(time: f32, phase: f32) -> f32 {
    4.0 * (((time - phase + 0.25) % 1.0) - 0.5).abs() - 1.0
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
        process_inputs_to_data(&mut self.data.inputs, data);

        for sample in data {
            *sample *= self.amount;
        }
    }
    fn add_input(&mut self, input: AudioNodeTarget) {
        self.data.inputs.push(Input::new(input));
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
        process_inputs_to_data(&mut self.data.inputs, data);
    }
    fn add_input(&mut self, input: AudioNodeTarget) {
        self.data.inputs.push(Input::new(input));
    }
}

#[derive(Default)]
struct AudioNodeData {
    inputs: Vec<Input>,
}

struct Input {
    node: AudioNodeTarget,
    buffer: [f32; 4096],
}

impl Input {
    fn new(node: AudioNodeTarget) -> Self {
        Self {
            node,
            buffer: [0.0; 4096],
        }
    }
}

fn process_inputs_to_data(inputs: &mut [Input], data: &mut [f32]) {
    for (i, input) in inputs.iter_mut().enumerate() {
        // data.iter_mut().enumerate().take(input.buffer.len())
        // for i in 0..input.buffer.len() {
        //     data[i] += input.buffer[i];
        // }

        if i == 0 {
            input.node.process(data);
        } else {
            input.node.process(&mut input.buffer[..data.len()]);
            // for (i, sample) in input.buffer.iter().enumerate() {
            //     data[i] += sample;
            // }
            for (i, sample) in data.iter_mut().enumerate() {
                *sample += input.buffer[i];
            }
        }
    }
}

pub fn create_graph(sample_rate: u32) -> AudioGraph {
    let mut audio_graph = AudioGraph::new(sample_rate);
    let osc1 = Box::new(Osc::new(audio_graph.sample_rate, 220.0, OscWave::Triangle));
    let osc2 = Box::new(Osc::new(audio_graph.sample_rate, 293.66, OscWave::Triangle));
    let mut gain = Box::new(Gain::new(0.5));
    // osc.connect(gain);
    // gain.connect(&audio_graph.output);
    gain.add_input(osc1);
    // gain.add_input(osc2);
    audio_graph.output.add_input(gain);
    // let mut data = [0_f32; 480];
    // audio_graph.process(&mut data);

    audio_graph
}

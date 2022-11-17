#![allow(unused)]

use std::{
    cell::RefCell,
    collections::VecDeque,
    f32::consts::PI,
    sync::{mpsc::*, *},
};

use audio_graph::*;
use cpal::{traits::*, *};

use eframe::egui;

pub mod audio_graph;

pub fn run() {
    println!("Hello, world!");

    let host = cpal::default_host();

    let output_devices: Vec<_> = host.output_devices().unwrap().collect();

    for device in output_devices {
        let mut supported_configs_range: Vec<_> = device
            .supported_output_configs()
            .expect("error while querying configs")
            .collect();
        dbg!(device.name().unwrap());
        dbg!(supported_configs_range);
    }

    let device = host
        .default_output_device()
        .expect("no output device available");

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");

    let mut supported_configs: VecDeque<_> = supported_configs_range.collect();

    dbg!(&supported_configs);

    let first_config = supported_configs.pop_front();

    let supported_config = first_config
        .expect("no supported config?!")
        .with_max_sample_rate();

    let sample_format = supported_config.sample_format();

    let config: StreamConfig = supported_config.into();

    let (sen, rcv) = mpsc::channel::<f32>();
    let freq_mutex = Arc::new(Mutex::new(Some(440.0_f32)));

    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(&device, &config, &freq_mutex, rcv),
        SampleFormat::I16 => build_stream::<i16>(&device, &config, &freq_mutex, rcv),
        SampleFormat::U16 => build_stream::<u16>(&device, &config, &freq_mutex, rcv),
    }
    .unwrap();

    // stream.play().unwrap();

    start_eframe(sen, freq_mutex, stream);

    // stream.pause().unwrap();
}

pub fn run2() {
    println!("Hello, world 2!");

    let host = cpal::default_host();

    let output_devices: Vec<_> = host.output_devices().unwrap().collect();

    for device in output_devices {
        let mut supported_configs_range: Vec<_> = device
            .supported_output_configs()
            .expect("error while querying configs")
            .collect();
        dbg!(device.name().unwrap());
        dbg!(supported_configs_range);
    }

    let device = host
        .default_output_device()
        .expect("no output device available");

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");

    let mut supported_configs: VecDeque<_> = supported_configs_range.collect();

    dbg!(&supported_configs);

    let first_config = supported_configs.pop_front();

    let supported_config = first_config
        .expect("no supported config?!")
        .with_max_sample_rate();

    let sample_format = supported_config.sample_format();

    let config: StreamConfig = supported_config.into();

    let (sen, rcv) = mpsc::channel::<f32>();
    let freq_mutex = Arc::new(Mutex::new(Some(440.0_f32)));

    let stream = build_stream2(&device, &config).unwrap();

    // let stream = match sample_format {
    //     SampleFormat::F32 => build_stream::<f32>(&device, &config, &freq_mutex, rcv),
    //     SampleFormat::I16 => build_stream::<i16>(&device, &config, &freq_mutex, rcv),
    //     SampleFormat::U16 => build_stream::<u16>(&device, &config, &freq_mutex, rcv),
    // }
    // .unwrap();

    // stream.play().unwrap();

    start_eframe(sen, freq_mutex, stream);

    // stream.pause().unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
fn start_eframe(sender: Sender<f32>, freq_mutex: Arc<Mutex<Option<f32>>>, stream: Stream) {
    stream.play().unwrap();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "sine wave freq slider",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc, sender, freq_mutex, stream))),
    );
}

#[cfg(target_arch = "wasm32")]
fn start_eframe(sender: Sender<f32>, freq_mutex: Arc<Mutex<Option<f32>>>, stream: Stream) {
    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "the_canvas_id", // hardcode it
        web_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc, sender, freq_mutex, stream))),
    )
    .expect("failed to start eframe");
}

pub fn build_stream<T: Sample>(
    device: &Device,
    config: &StreamConfig,
    freq_mutex: &Arc<Mutex<Option<f32>>>,
    rcv: Receiver<f32>,
) -> Result<Stream, BuildStreamError> {
    let channel_count = config.channels as usize;
    let sample_rate = config.sample_rate.0 as f32;
    let mut sample_clock = 0f32;
    let freq_mutex_clone = freq_mutex.clone();
    let mut freq = 440.0;

    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            // Get freq using channel, grab the last message sent
            // Like a manual spinlock?
            // while let Ok(freq2) = rcv.try_recv() {
            //     freq = freq2;
            // }

            // let mut rng = rand::thread_rng();

            // Get freq using mutex
            if let Ok(x) = freq_mutex_clone.try_lock() {
                if let Some(f) = *x {
                    freq = f;
                }
            }

            // react to stream events and read or write stream data here.
            for frame in data.chunks_mut(channel_count) {
                sample_clock = (sample_clock + 1.0) % sample_rate;
                let time_secs = sample_clock / sample_rate;

                // let mut noise: i16 = rng.gen(); // generates a float between 0 and 1
                // noise /= 20;

                // let sin = (time_secs * freq * PI * 2.0).sin() / 20.0;

                let tri = ((((time_secs * freq) % 1.0) - 0.5).abs() - 0.25) * 4.0;

                for sample in frame {
                    *sample = Sample::from(&(tri / 20.0));
                }
            }
        },
        move |err| {
            // react to errors here.
            println!("{:?}", err);
        },
    )
}
pub fn build_stream2(device: &Device, config: &StreamConfig) -> Result<Stream, BuildStreamError> {
    let mut audio_graph = create_graph(config.sample_rate.0, config.channels as usize);

    // let (sender, receiver) = mpsc::channel::<AudioGraph>();
    // let mut ag2: Option<AudioGraph> = None;

    device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // while let Ok(ag) = receiver.try_recv() {
            //     ag2 = Some(ag);
            // }
            audio_graph.process(data);
        },
        move |err| {
            // react to errors here.
            println!("{:?}", err);
        },
    )
}

// trying to use inner cpal functions so thread i can initialize the thread, but having trouble getting access to the wasapi functions from here
// pub fn build_stream3(device: &Device, config: &StreamConfig) -> Result<Stream, BuildStreamError> {
//     let mut audio_graph = create_graph(config.sample_rate.0, config.channels as usize);

//     let (sender, receiver) = mpsc::channel::<AudioGraph>();
//     let mut ag2: Option<AudioGraph> = None;

//     let mut data_callback = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
//         // while let Ok(ag) = receiver.try_recv() {
//         //     ag2 = Some(ag);
//         // }
//         audio_graph.process(data);
//     };

//     let error_callback = move |err| {
//         // react to errors here.
//         println!("{:?}", err);
//     };

//     // device.build_output_stream_raw(
//     //     config,
//     //     f32::FORMAT,
//     //     move |data, info| {
//     //         data_callback(
//     //             data.as_slice_mut()
//     //                 .expect("host supplied incorrect sample type"),
//     //             info,
//     //         )
//     //     },
//     //     error_callback,
//     // )

//     let stream_inner = device.build_output_stream_raw_inner(config, sample_format)?;
//     Ok(cpal::platform::Stream::new_output(
//         stream_inner,
//         data_callback,
//         error_callback,
//     ))
// }

pub struct MyEguiApp {
    freq: f32,
    sender: Sender<f32>,
    freq_mutex: Arc<Mutex<Option<f32>>>,
    stream: Stream,
}

impl MyEguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        sender: Sender<f32>,
        freq_mutex: Arc<Mutex<Option<f32>>>,
        stream: Stream,
    ) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            freq: 440.0,
            sender,
            freq_mutex,
            stream,
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label("frequency:");
            ui.add(
                egui::Slider::new(&mut self.freq, 10.0..=4000.0)
                    .orientation(egui::SliderOrientation::Horizontal)
                    .suffix(" Hz"),
            );
            if (ui.button("Play").clicked()) {
                self.stream.play().unwrap();
            }
            if (ui.button("Stop").clicked()) {
                self.stream.pause().unwrap();
            }
        });
        // self.sender.send(self.freq);
        self.freq_mutex.lock().unwrap().replace(self.freq);
    }
}


use std::sync::mpsc::{channel, Receiver, Sender};
use egui_macroquad::egui::ahash::{HashMap, HashMapExt};

use crate::app_state;
use crate::state::UiState;

//Liste des instruments
pub const INSTRUMENTS: [&str; 5] = ["Kick", "Snare", "Open Hat", "Close Hat", "Clap"];

pub struct AppState {
    pub ui: UiState,
    //Les 16 pas
    pub patterns: HashMap<String, [bool; 16]>,
    //Master Volume
    pub master_volume: f32,
    //State
    pub is_playing: bool,
    pub current_step: usize,
    //BPM
    pub bpm: f32,
    pub last_step_time: f64,

    //SAMPLES
    pub selected_instrument: Option<String>,
    pub instrument_samples: HashMap<String,String>,
    //Thread
    pub rx_sample: Option<Receiver<(String, String)>>,
    pub tx_sample: Option<Sender<(String, String)>>,
    //Volume de chaque instrument
    pub instrument_volumes: HashMap<String, f32>,
    //Mute et Solo
    pub instrument_mute: HashMap<String, bool>,
    pub instrument_solo: HashMap<String, bool>,

}

impl AppState {
    pub async fn new() -> Self {
        let (tx,rx) = channel();
        //Init des patterns
        let mut patterns = HashMap::new();
        //Init du volume de chaque instrument
        let mut instrument_volumes = HashMap::new();
        //Mute et Solo
        let mut instrument_mute = HashMap::new();
        let mut instrument_solo = HashMap::new();

        for &instrument in INSTRUMENTS.iter() {
            let inst_name = instrument.to_string();
            patterns.insert(inst_name.clone(), [false; 16]);
            //Le volume de chaque piste => 0.8
            instrument_volumes.insert(inst_name.clone(), 0.8);
            instrument_mute.insert(inst_name.clone(), false);
            instrument_solo.insert(inst_name.clone(), false);
        }
        Self {
            ui: UiState::default(),
            patterns: HashMap::default(),
            master_volume: 0.8,
            is_playing: false,
            current_step: 0,
            bpm: 120.0,
            last_step_time: 0.0,
            selected_instrument: None,
            instrument_samples: HashMap::default(),
            rx_sample: Some(rx),
            tx_sample: Some(tx),
            instrument_volumes: HashMap::default(),
            instrument_mute: HashMap::default(),
            instrument_solo: HashMap::default(),
        }
    }

    //Retourne une reference mutable vers le tableau de pas d'un instrument
    pub fn get_or_init_pattern(&mut self, instrument: &str) -> &mut [bool;16] {
        self.patterns.entry(instrument.to_string()).or_insert([false; 16])
    }
    pub fn check_sample_loading(&mut self) {
        if let Some(rx) = &self.rx_sample {
            // try_recv() récupère le message s'il y en a un, sans bloquer
            while let Ok((inst, path)) = rx.try_recv() {
                println!("✅ Sample reçu pour {}: {}", inst, path);
                self.instrument_samples.insert(inst.clone(), path);
                // Important : Initialiser le pattern si c'est un nouvel instrument
                self.patterns.entry(inst.clone()).or_insert([false; 16]);
                self.instrument_volumes.entry(inst).or_insert(0.8);
            }
        }
    }
}

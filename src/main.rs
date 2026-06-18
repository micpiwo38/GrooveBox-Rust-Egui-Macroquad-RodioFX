use egui_macroquad::{egui, macroquad};
use egui_macroquad::macroquad::color::{BLACK, GREEN};
use egui_macroquad::macroquad::prelude::{clear_background, next_frame};
use egui_macroquad::macroquad::text::draw_text;
use egui_macroquad::ui as egui_ui;
use kira::effect;
// Moteur de son Kira
use kira::manager::{AudioManager, AudioManagerSettings, DefaultBackend};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::effect::reverb::ReverbBuilder;
use kira::track::{TrackBuilder};
use macroquad::time::get_time;

mod state;
mod app_state;
mod gui;

use app_state::AppState;

#[macroquad::main(window_configuration)]
async fn main() {
    // 1. Initialisation du moteur audio Kira
    let mut manager: AudioManager<DefaultBackend> =
        AudioManager::new(AudioManagerSettings::default())
            .expect("Failed to create audio manager");

    // 1 BIS Effet reverb Kira
    let reverb_builder = ReverbBuilder::new()
        .feedback(0.0) // Taille de la pièce = room_size
        .mix(0.0);

    // On crée un builder de piste, on lui ajoute la réverb, et on l'ajoute au manager
    let mut track_builder = TrackBuilder::new();
    let mut reverb_handle = track_builder.add_effect(reverb_builder);

    // On enregistre cette piste dans le manager (elle sera connectée au Master automatiquement)
    let master_fx_track = manager.add_sub_track(track_builder)
        .expect("Failed to create Master FX track");
    // 2. Initialisation de l'état de l'application
    let mut state = AppState::new().await;

    // Le timer BPM
    let mut last_step_time: f64 = 0.0;

    // Boucle principale
    loop {
        // A. Vérifier le chargement des fichiers audio (UI -> Main Thread)
        state.check_sample_loading();

        // B. Afficher l'interface graphique
        egui_ui(|ctx| {
            gui::layout::draw(ctx, &mut state);
        });

        // C. Rendu Macroquad (fond + texte de debug)
        clear_background(BLACK);

        let statut_text = if state.is_playing {
            format!("Playing : Step {:02} - BPM {}", state.current_step + 1, state.bpm as i32)
        } else {
            "Stopped".to_string()
        };
        draw_text(&statut_text, 20.0, 30.0, 30.0, GREEN);

        egui_macroquad::draw();

        // D. Logique du Séquenceur Audio
        if state.is_playing {
            let current_time = get_time();
            let step_duration = (60.0 / state.bpm) / 4.0; // Double croche

            if current_time - last_step_time >= step_duration as f64 {
                last_step_time = current_time;
                state.current_step = (state.current_step + 1) % 16;

                println!("Step {} / 16", state.current_step + 1);

                // Vérifier si un bouton SOLO est activé quelque part
                let any_solo_active = state.instrument_solo.values().any(|&is_solo| is_solo);

                // Copier la liste des instruments pour éviter les conflits d'emprunt
                let instrument_to_check: Vec<(String, String)> = state.instrument_samples
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                for (inst_name, file_path) in instrument_to_check {
                    // 1. Récupérer le pattern de l'instrument
                    if let Some(pattern) = state.patterns.get(&inst_name) {

                        // 2. Vérifier si le step actuel est activé dans le pattern
                        if pattern[state.current_step] {

                            // --- LOGIQUE MUTE / SOLO ---
                            let is_muted = state.instrument_mute.get(&inst_name).copied().unwrap_or(false);
                            let is_solo = state.instrument_solo.get(&inst_name).copied().unwrap_or(false);

                            // Règle : On joue si ce n'est pas Mute ET (c'est en Solo OU aucun Solo n'est actif globalement)
                            let should_play = !is_muted && (is_solo || !any_solo_active);

                            if should_play {
                                // Récupérer le volume individuel
                                let inst_vol = state.instrument_volumes.get(&inst_name).copied().unwrap_or(0.8);
                                // Calcul du volume final : Master * Individuel
                                let final_volume = (state.master_volume * inst_vol) as f64;

                                // 3. Charger et jouer le son
                                match StaticSoundData::from_file(&file_path) {
                                    Ok(sound_data) => {

                                        let settings = sound_data.with_settings(
                                            StaticSoundSettings::new().volume(final_volume)
                                                .output_destination(&master_fx_track)
                                        );
                                        let current_mix = if state.reverb_enabled { state.reverb_mix as f64 } else { 0.0 };
                                        let current_room_size = state.reverb_room_size as f64;

                                        reverb_handle.set_mix(current_mix, kira::tween::Tween::default());
                                        reverb_handle.set_feedback(current_room_size, kira::tween::Tween::default());

                                        if let Err(e) = manager.play(settings) {
                                            eprintln!("Error playing {}: {}", inst_name, e);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error loading sound {}: {}", file_path, e);
                                    }
                                }
                            } else {
                                // Debug optionnel pour voir pourquoi ça ne joue pas
                                // println!("Muted/Solo logic skipped: {} (Mute: {}, Solo: {}, AnySolo: {})", inst_name, is_muted, is_solo, any_solo_active);
                            }
                        }
                    }
                }
            }
        } else {
            // Reset du timer quand stoppé pour éviter un saut au prochain Play
            last_step_time = get_time();
            state.current_step = 0;
        }

        next_frame().await;
    }
}

fn window_configuration() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Mic GrooveBox".to_string(),
        window_width: 1600,
        window_height: 900,
        window_resizable: false,
        ..Default::default()
    }
}
use std::thread;
use egui_macroquad::egui::{Align, Button, Color32, Frame, Layout, Response, RichText, Slider, Stroke, Ui, Vec2};
use crate::app_state::{AppState, INSTRUMENTS};


pub fn draw(ui: &mut Ui, state: &mut AppState){
    ui.heading("MAIN");
    ui.separator();
    let total_space_available = ui.available_width();
    let left_panel_width = total_space_available * 0.2;
    let middle_panel_width = total_space_available * 0.6;
    let right_panel_width = total_space_available * 0.2;

    //Panel de gauche
    ui.horizontal(|ui| {
        Frame::new()
            .fill(Color32::from_rgba_unmultiplied(217, 112, 74, 60))
            .inner_margin(8.0)
            .show(ui, |ui|{
               ui.set_min_width(left_panel_width);
               ui.set_max_width(left_panel_width);
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui|{
                    ui.label("CONTROLS");
                    ui.add_space(10.0);
                    //Le volume
                    ui.label(format!("MASTER VOLUME: {:.0}%", (state.master_volume * 100.0).clamp(0.0, 100.0)));
                    ui.add(Slider::new(&mut state.master_volume, 0.0..=1.0));
                    //Bouton PLAY-PAUSE-STOP
                    ui.horizontal(|ui|{
                        if ui.button("▶ PLAY").clicked(){
                            state.is_playing = true;
                        }
                        if ui.button("⏸").clicked() { state.is_playing = !state.is_playing; }
                        if ui.button("⏹ STOP").clicked() {
                            state.is_playing = false;
                        }
                    });
                    ui.add_space(25.0);
                    ui.label("TEMPO BPM");
                    ui.add_space(5.0);
                    ui.horizontal(|ui|{
                        let btn_minus = Button::new("➖").min_size(Vec2::new(40.0, 30.0));
                        let response_minus = ui.add(btn_minus);
                        adjust_bpm(&response_minus, state, -1.0, -1.0);
                        ui.label(RichText::new(format!("{:.0}", state.bpm)).size(20.0).strong());
                        let btn_plus = Button::new("➕").min_size(Vec2::new(40.0, 30.0));
                        let response_plus = ui.add(btn_plus);
                        adjust_bpm(&response_plus, state, 1.0, 1.0);
                    });
                });
            });
        //Pannel de milieu
        // --- COLONNE CENTRE (INSTRUMENTS) ---
        Frame::new()
            .fill(Color32::from_rgba_unmultiplied(32, 30, 38, 60))
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_min_width(middle_panel_width);
                ui.set_max_width(middle_panel_width);

                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.set_width(middle_panel_width - 20.0);
                    ui.label("INSTRUMENTS");
                    ui.separator();
                    ui.add_space(10.0);

                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(15.0, 15.0);
                        let instruments = ["Kick", "Snare", "Open Hat", "Close Hat", "Clap"];

                        for inst in instruments {
                            ui.vertical(|ui| {
                                ui.set_min_width(100.0);
                                ui.set_max_width(100.0);
                                let sample_path = state.instrument_samples.get(inst).map(|s| s.as_str());
                                let is_selected = state.selected_instrument.as_deref() == Some(inst);

                                let btn_label = if sample_path.is_some() {
                                    format!("{} 🎵", inst)
                                } else {
                                    inst.to_string()
                                };

                                let mut rich_button_text = RichText::new(btn_label)
                                    .size(14.0)
                                    .color(Color32::WHITE);
                                if is_selected {
                                    rich_button_text = rich_button_text.color(Color32::YELLOW).strong();
                                }
                                let mut button = Button::new(rich_button_text).min_size(Vec2::new(100.0, 100.0));
                                //Du padding
                                if is_selected {
                                    button = button.fill(Color32::DARK_BLUE).stroke(Stroke::new(2.0, Color32::YELLOW));
                                }else{
                                    button = button.fill(Color32::DARK_GRAY).stroke(Stroke::new(2.0, Color32::WHITE));
                                }
                                let response = ui.add(button);
                                // Bouton pour charger un sample
                                if response.clicked() {
                                    let instrument_name = inst.to_string();
                                    if let Some(tx) = state.tx_sample.clone() {
                                        thread::spawn(move || {
                                            let file_path = rfd::FileDialog::new()
                                                .set_title(format!("Select sample for {}", instrument_name))
                                                .set_directory("samples")
                                                .add_filter("Audio Files", &["wav", "ogg"])
                                                .pick_file();
                                            if let Some(path) = file_path {
                                                let _ = tx.send((instrument_name, path.to_string_lossy().to_string()));
                                            }
                                        });
                                    }
                                }

                                // Bouton de sélection de la piste a editer
                                ui.add_space(2.0);
                                let select_btn_label = if is_selected { "✏️ Editing" } else { "✏️ Select" };
                                let select_btn = Button::new(select_btn_label)
                                    .min_size(Vec2::new(100.0, 40.0))
                                    .fill(if is_selected { Color32::BLUE } else { Color32::DARK_GRAY });
                                //Clic -> selectionner la piste a editer
                                if ui.add(select_btn).clicked() {
                                    state.selected_instrument = Some(inst.to_string());
                                }

                                // Nom du fichier
                                ui.add_space(2.0);
                                if let Some(path) = sample_path {
                                    let file_name = path.split('\\').last().unwrap_or(path.split('/').last().unwrap_or(path));
                                    let display_name = if file_name.len() > 10 { format!("...{}", &file_name[file_name.len()-8..]) } else { file_name.to_string() };
                                    ui.label(RichText::new(display_name).size(9.0).color(Color32::LIGHT_GREEN));
                                } else {
                                    ui.label(RichText::new("No sample").size(9.0).color(Color32::GRAY));
                                }
                                //Slider volume vertical
                                ui.add_space(5.0);
                                //Recuperer => initialisé le volume de chaque piste
                                let vol = state.instrument_volumes.entry(inst.to_string()).or_insert(0.8);
                                //Le slider est vertical
                                ui.add(Slider::new(vol, 0.0..=1.0)
                                    .vertical()
                                    .text("")
                                    .show_value(false)
                                );
                                //Indicateur du niveau du volume
                                ui.label(RichText::new("VOLUME").size(8.0).color(Color32::WHITE));
                                //Bouton Mute et Solo
                                ui.add_space(5.0);
                                ui.horizontal(|ui|{ ui.set_width(100.0); // Aligner avec le conteneur parent
                                    // 1. Récupérer les états Mute et Solo
                                    let is_muted = state.instrument_mute.get(inst).copied().unwrap_or(false);
                                    let is_solo = state.instrument_solo.get(inst).copied().unwrap_or(false);

                                    // 2. Définir les couleurs selon l'état
                                    let mute_color = if is_muted { Color32::RED } else { Color32::DARK_GRAY };
                                    let solo_color = if is_solo { Color32::GREEN } else { Color32::DARK_GRAY };

                                    // 3. Créer le bouton MUTE avec sa couleur
                                    let btn_mute = Button::new("M")
                                        .min_size(Vec2::new(20.0, 20.0))
                                        .fill(mute_color) // <--- ICI : On applique la couleur !
                                        .stroke(Stroke::new(1.0, Color32::WHITE));

                                    if ui.add(btn_mute).clicked() {
                                        // Inverser l'état Mute
                                        let new_state = !is_muted;
                                        state.instrument_mute.insert(inst.to_string(), new_state);
                                        // Optionnel : Si on mute, on peut désactiver le solo pour éviter les conflits
                                        if new_state {
                                            state.instrument_solo.insert(inst.to_string(), false);
                                        }
                                    }

                                    // 4. Créer le bouton SOLO avec sa couleur
                                    let btn_solo = Button::new("S")
                                        .min_size(Vec2::new(20.0, 20.0))
                                        .fill(solo_color) // <--- ICI : On applique la couleur !
                                        .stroke(Stroke::new(1.0, Color32::WHITE));

                                    if ui.add(btn_solo).clicked() {
                                        // Inverser l'état Solo
                                        let new_state = !is_solo;
                                        state.instrument_solo.insert(inst.to_string(), new_state);

                                        // Logique "Solo Exclusif" (optionnelle mais recommandée) :
                                        // Si on active un solo, on désactive tous les autres solos
                                        if new_state {
                                            for (key, val) in state.instrument_solo.iter_mut() {
                                                if key != inst {
                                                    *val = false;
                                                }
                                            }
                                        }
                                    }
                                });
                            });
                        }
                    });
                });
            });

        ui.separator();


        Frame::new()
            .fill(Color32::from_rgba_unmultiplied(240, 195, 110, 60))
            .inner_margin(8.0)
            .show(ui, |ui|{
               ui.set_min_width(right_panel_width);
               ui.set_max_width(right_panel_width);
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui|{
                    ui.label("EFFECTS");
                    ui.separator();
                    ui.add_space(10.0);
                });
            });
    });
}

//BPM
//Fonction utilitaire
pub fn adjust_bpm(response: &Response, state: &mut AppState, step_fast: f32, step_sign: f32) {
    //1. clic simple puis relaché
    if response.clicked(){
        state.bpm = (state.bpm + (step_sign * step_fast * 0.2)).clamp(20.0, 240.0);
    }
    //2. Maintien du clic
    if response.is_pointer_button_down_on(){
        let acceleration = 0.2;
        state.bpm = (state.bpm + (step_sign * step_fast * acceleration)).clamp(20.0, 240.0);
    }
}
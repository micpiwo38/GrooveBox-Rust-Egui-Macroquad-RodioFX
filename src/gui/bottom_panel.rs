use egui_macroquad::egui::{Color32, Stroke, Ui, Vec2};
use macroquad::color::DARKGRAY;
use crate::app_state::AppState;

pub fn draw(ui: &mut Ui, state: &mut AppState){
    ui.heading("PATTERNS");
    ui.separator();
    //1. Stocker les etats de lecture AVANT tout les emprunt mutable
    let is_playing = state.is_playing;
    let current_step = state.current_step;
    let current_instrument = state.selected_instrument.clone();

    //2. Conteneur verticale
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            //Si aucun instrument n'est selectionné, on affiche un message
            if current_instrument.is_none() {
                ui.label("Select an instrument to start playing");
                ui.add_space(20.0);
                //Affichage fantome => afficher mais inactif
                ui.horizontal_centered(|ui| {
                    let total_space_available = ui.available_width();
                    let safe_width = total_space_available - 120.0;
                    //Diviser l'espace disponible en 16 cases
                    let step_width = safe_width / 16.0;
                    for i in 0..16 {
                        ui.add(
                            egui_macroquad::egui::Button::new(format!("{:02}", i + 1))
                                .min_size(Vec2::new(step_width, 50.0))
                                .sense(egui_macroquad::egui::Sense::hover())
                                .stroke(Stroke::new(1.0, Color32::DARK_GRAY))
                                .fill(Color32::BLACK)
                        );
                    }
                });
                return;
            }
            let inst_name = current_instrument.unwrap();
            ui.label(format!("Edit Pattern: {}", inst_name));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let total_space_available = ui.available_width();
                let safe_width = total_space_available - 120.0;
                //Diviser l'espace disponible en 16 cases
                let step_width = safe_width / 16.0;
                //Emprunt mutable pour recuperer le tableau de pas
                let steps = state.get_or_init_pattern(&inst_name);

                for (index, step_enabled) in steps.iter_mut().enumerate() {
                    let label = format!("{:02}", index + 1);
                    //Couleur de fond : Rouge si activé sinon Gris
                    let color = if *step_enabled{
                        Color32::RED
                    }else{
                        Color32::DARK_GRAY
                    };
                    //Couleur de la bordure : pour le pas en cours
                    let border_color = if is_playing && current_step == index {
                        Color32::BLUE
                    }else{
                        Color32::WHITE
                    };
                    let response = ui.add(
                        egui_macroquad::egui::Button::new(label)
                            .min_size(Vec2::new(step_width, 50.0))
                            .stroke(Stroke::new(2.0, border_color))
                            .fill(color)
                    );
                    //Si clic = actif <=> inactif
                    if response.clicked() {
                        *step_enabled = !*step_enabled;
                    }
                }
            });
        });
    });
}
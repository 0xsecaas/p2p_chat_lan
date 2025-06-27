use eframe::egui;

pub struct WalkieTalkieApp {
    input: String,
    messages: Vec<String>,
}

impl Default for WalkieTalkieApp {
    fn default() -> Self {
        Self {
            input: String::new(),
            messages: Vec::new(),
        }
    }
}

impl eframe::App for WalkieTalkieApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Walkie Talkie Chat");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for msg in &self.messages {
                    ui.label(msg);
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                let input = ui.text_edit_singleline(&mut self.input);
                if input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !self.input.trim().is_empty() {
                        self.messages.push(self.input.clone());
                        self.input.clear();
                    }
                }
                if ui.button("Send").clicked() {
                    if !self.input.trim().is_empty() {
                        self.messages.push(self.input.clone());
                        self.input.clear();
                    }
                }
            });
        });
    }
}

pub fn run_gui() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "P2P Walkie Talkie GUI",
        options,
        Box::new(|_cc| Box::new(WalkieTalkieApp::default())),
    );
}

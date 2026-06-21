use eframe::egui; 
use std::collections::HashMap;
// fn structures 
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default(); //sets window popup to default options/settings
    eframe::run_native(
        "Chat Box App", 
        options, //Uses settings from 'options' variable 
        Box::new(|_cc| Ok(Box::new(ChatBoxApp::default()))), //when window starts, it starts ChatBoxApp using the variables from ChatBoxApp
    )
}

impl ChatBoxApp {
    fn save_data(&self) {
        let json = serde_json::to_string_pretty(&self.users).unwrap();
        std::fs::write("chat_history.json", json).unwrap();
    }

    fn load_data() -> Vec<User> {
        match std::fs::read_to_string("chat_history.json") {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }// These functions allow the app to save the chat history to a json file and load it when the app starts, so that the chat history is preserved between sessions. (Can be changed later to save/load from a database or other storage method if desired)
    fn copy_image_to_app_folder(original_path: &std::path::Path) -> Option<String> {
        std::fs::create_dir_all("chat_data/images").ok()?;

        let file_name = original_path.file_name()?.to_string_lossy();

        let new_path = format!("chat_data/images/{}", file_name);

        std::fs::copy(original_path, &new_path).ok()?;

        Some(new_path)
    } // coppies the image to a folder within the app's directory and returns the new path, so security permissions are not an issue when trying to load the image later. (Can be changed later to upload the image to a server or cloud storage if desired)
}

impl ChatBoxApp {
    fn load_image_from_path(&mut self, ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
        if let Some(tex) = self.image_cache.get(path) {
            return Some(tex.clone());
        }

        let image = image::open(path).ok()?;
        let size = [image.width() as usize, image.height() as usize];
        let rgba = image.to_rgba8();

        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            rgba.as_flat_samples().as_slice(),
        );

        let tex = ctx.load_texture(
            path.to_string(),
            color_image,
            egui::TextureOptions::LINEAR,
        );

        self.image_cache.insert(path.to_string(), tex.clone());
        Some(tex)
    }
}
// --------------------------------------------------------------------------

// Structs for the chat box app
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
struct ChatBoxMessage {
    sender: String,
    text: String,
    image_path: Option<String>,
}
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct User {
    name: String,
    messages: Vec<ChatBoxMessage>,
}
struct ChatBoxApp {
    input_text: String,
    users: Vec<User>,
    selected_user: usize,
    image_cache: HashMap<String, egui::TextureHandle>,
    selected_image: Option<String>,
}

impl Default for ChatBoxApp {
    fn default() -> Self {
        let loaded = ChatBoxApp::load_data();

        let users = if loaded.is_empty() {
            vec![
                User {
                    name: "GummyGoo9000".to_string(),
                    messages: Vec::new(),
                },
                User {
                    name: "BunnyClockLady".to_string(),
                    messages: Vec::new(),
                },
                User {
                    name: "TheOneAndOnlyGOAT6767".to_string(),
                    messages: Vec::new(),
                },
            ]
        } else {
            loaded
        };

        Self {
            input_text: String::new(),
            users,
            selected_user: 0,

            image_cache: HashMap::new(), 
            selected_image: None,
        }
    }
}

// --------------------------------------------------------------------------
impl eframe::App for ChatBoxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("users_panel").min_width(220.0).show(ctx, |ui| {
            ui.heading("💬 Chats");
            ui.separator();
            for (index, user) in self.users.iter().enumerate() {
                let selected = self.selected_user == index;
                let button = egui::Button::new(
                    egui::RichText::new(&user.name)
                        .size(16.0)
                )
                .fill(
                    if selected {
                        egui::Color32::from_rgb(80, 120, 255)
                    } else {
                        egui::Color32::from_rgb(60, 60, 60)
                    }
                )
                .corner_radius(12.0);

                if ui.add_sized([180.0, 40.0], button).clicked() {
                    self.selected_user = index;
                }
                ui.add_space(4.0);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(5.0);
            ui.label(
                egui::RichText::new(&self.users[self.selected_user].name)
                    .size(24.0)
                    .strong()
                    .color(egui::Color32::from_rgb(240, 248, 255)),
            );
            ui.add_space(15.0);

            egui::ScrollArea::vertical()
                .max_height(500.0)
                .max_width(500.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let messages = self.users[self.selected_user].messages.clone();
                    for message in &messages {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", message.sender));
                            if !message.text.is_empty() {
                                egui::Frame::new()
                                    .fill(egui::Color32::from_rgb(70, 70, 70))
                                    .corner_radius(10.0)
                                    .inner_margin(8.0)
                                    .show(ui, |ui| {
                                    egui::Frame::new()
                                        .corner_radius(12.0)
                                        .inner_margin(10.0)
                                        .fill(egui::Color32::from_rgb(65, 65, 65))
                                        .show(ui, |ui| {

                                            ui.set_max_width(400.0);

                                            ui.add(
                                                egui::Label::new(&message.text)
                                                    .wrap()
                                            );
                                        });
                                        });
                                    }
                        });

                        if let Some(image_path) = &message.image_path {
                            if let Some(texture) = self.load_image_from_path(ctx, image_path) {
                                let original_size = texture.size_vec2();
                                let max_width = 350.0;
                                let scale = (max_width / original_size.x).min(1.0);
                                let display_size = original_size * scale;
                                let image_button = egui::ImageButton::new(
                                    egui::Image::new((texture.id(), display_size))
                                );

                                if ui.add(image_button).clicked() {
                                    self.selected_image = Some(image_path.clone());
                                }
                            } else {
                                ui.label("Could not load image.");
                            }
                        }
                    }
                });

            ui.separator();

            ui.horizontal(|ui| {
                let text_box = ui.add_sized(
                    [500.0, 80.0],
                    egui::TextEdit::multiline(&mut self.input_text)
                );

                if ui.button("Send").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !self.input_text.trim().is_empty() {
                        self.users[self.selected_user].messages.push(ChatBoxMessage {
                            sender: "User".to_string(),
                            text: self.input_text.clone(),
                            image_path: None,
                        });
                        self.save_data();
                        self.input_text.clear();
                        text_box.request_focus();
                    }
                }

                if ui.button("Upload Photo").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg"])
                        .pick_file()
                    {
                        if let Some(saved_path) = ChatBoxApp::copy_image_to_app_folder(&path) {
                            self.users[self.selected_user].messages.push(ChatBoxMessage {
                                sender: "User".to_string(),
                                text: String::new(),
                                image_path: Some(saved_path),
                            });
                            self.save_data();
                        }
                    }
                }
            });
        });
        if let Some(image_path) = self.selected_image.clone() {
            let mut open = true;

            egui::Window::new("Photo Preview")
                .open(&mut open)
                .resizable(true)
                .show(ctx, |ui| {
                    if let Some(texture) = self.load_image_from_path(ctx, &image_path) {
                        let original_size = texture.size_vec2();

                        let max_width = 800.0;
                        let scale = (max_width / original_size.x).min(1.0);
                        let display_size = original_size * scale;

                        ui.image((texture.id(), display_size));
                    }
                    if ui.button("Download").clicked() {
                        if let Some(save_path) = rfd::FileDialog::new()
                            .set_file_name("image.png")
                            .save_file()
                        {
                            let _ = std::fs::copy(&image_path, save_path);
                        }
                    }
                });

            if !open {
                self.selected_image = None;
            }
        }
    }
}
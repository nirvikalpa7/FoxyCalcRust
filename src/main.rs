#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;  
  
fn main() -> eframe::Result<()>  {

    // Загружаем байты иконки во время компиляции
    let icon_bytes = include_bytes!("../assets/favicon_64x64.png");
    
    // Декодируем изображение с помощью библиотеки image
    let image = image::load_from_memory(icon_bytes)
        .expect("Не удалось загрузить иконку")
        .to_rgba8();
    
    let (width, height) = image.dimensions();
    
    // Создаем структуру IconData
    let icon_data = egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    // Передаем иконку в настройки окна
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_icon(icon_data),  // Добавляем иконку сюда
        ..Default::default()
    };

    eframe::run_native(  
        "Rust FoxyCalc v0.1",
        native_options,  
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::Theme::Dark); // Theme::Dark / Theme::Light
            Ok(Box::new(MyApp::new(cc)))
        }),  
    )
}  

struct MyApp {}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {}
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Привет! Посмотри на иконку окна.");
        });
    }
}

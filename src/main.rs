#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() -> eframe::Result<()>  {

    // Загружаем байты иконки во время компиляции
    let icon_bytes = include_bytes!("../assets/favicon_128x128.png");
    
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
            cc.egui_ctx.set_theme(egui::Theme::Light); // Theme::Dark / Theme::Light
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(cc)))
        }),  
    )
}  

struct MyApp {
    // Буферы для хранения текста из полей ввода
    invest_duration_text: String,   // Длительность инвестиций, годы
    interest_text: String,          // Процентная ставка инвестиций, проценты
    start_deposit_text: String,     // Начальный капитал
    deposit_text: String,           // Размер пополнения
    deposit_years_text: String,     // Сколько лет делаем пополнения
    withdraw_text: String,          // Размер ежемесячного изъятия
    withdraw_start_text: String,    // Год, начала изъятий
    inflation_text: String,         // Годовой размер инфляции, проценты
    salary_grow_text: String,       // Годовой рост пополнения, проценты
    reinvest_duration_text: String, // Период реинвестирования процентов
    age_of_investor_text: String,   // Возраст инвестора на первый год инвестиций
    calendar_year_text: String,     // Календарный год на первый год инвестиций
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {

        Self {
            invest_duration_text: String::new(),
            interest_text: String::new(),
            start_deposit_text: String::new(),
            deposit_text: String::new(),
            deposit_years_text: String::new(),
            withdraw_text: String::new(),
            withdraw_start_text: String::new(),
            inflation_text: String::new(),
            salary_grow_text: String::new(),
            reinvest_duration_text: String::new(),
            age_of_investor_text: String::new(),
            calendar_year_text: String::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Расширенный инвест калькулятор с пополнением, изъятием, инфляцией и ростом зарплаты");
            ui.add_space(10.0);

            // 1. Получаем текущие визуальные настройки
            let mut visuals = ctx.style().visuals.clone();

            // 2. Настраиваем Постоянную Серую Рамку для неактивных полей
            visuals.widgets.inactive.bg_stroke = egui::Stroke {
                width: 1.0,
                color: egui::Color32::from_gray(180), 
            };
            
            // 3. Применяем измененные стили к контексту
            ctx.set_visuals(visuals);

            // Задаем точные размеры для полей ввода [Ширина, Высота]
            let edit_size = egui::vec2(290.0, 24.0);
            let edit_space: f32 = 8.0;
            let button_icon_size: f32 = 35.0;

            // Вшиваем иконку (например, лежащую в папке assets)
            let icon_source_calc = egui::include_image!("../assets/button_icon_calc.png");
            let icon_source_demo = egui::include_image!("../assets/button_icon_demo.png");
            let icon_source_save = egui::include_image!("../assets/button_icon_save.png");
            let icon_source_lang = egui::include_image!("../assets/button_icon_lang.png");

            let icon_sized_calc = egui::Image::new(icon_source_calc)
                .fit_to_exact_size(egui::vec2(button_icon_size, button_icon_size)) // РАЗМЕР ИКОНКИ ТУТ
                .corner_radius(3.0);
            let icon_sized_demo = egui::Image::new(icon_source_demo)
                .fit_to_exact_size(egui::vec2(button_icon_size, button_icon_size)) 
                .corner_radius(3.0);
            let icon_sized_save = egui::Image::new(icon_source_save)
                .fit_to_exact_size(egui::vec2(button_icon_size, button_icon_size))
                .corner_radius(3.0);
            let icon_sized_lang = egui::Image::new(icon_source_lang)
                .fit_to_exact_size(egui::vec2(button_icon_size, button_icon_size))
                .corner_radius(3.0);

            let image_btn_calc = egui::Button::image_and_text(icon_sized_calc, "Рассчитать");
            let image_btn_demo = egui::Button::image_and_text(icon_sized_demo, "Демо данные");
            let image_btn_save = egui::Button::image_and_text(icon_sized_save, "Сохранить");
            let image_btn_lang = egui::Button::image_and_text(icon_sized_lang, "English");
            ui.horizontal(|ui| {
                if ui.add_sized(egui::vec2(130.0, 50.0), image_btn_calc).clicked() {
                    // Клик по иконке
                }
                if ui.add_sized(egui::vec2(150.0, 50.0), image_btn_demo).clicked() {
                    // Клик по иконке
                }
                if ui.add_sized(egui::vec2(130.0, 50.0), image_btn_save).clicked() {
                    // Клик по иконке
                }
                if ui.add_sized(egui::vec2(110.0, 50.0), image_btn_lang).clicked() {
                    // Клик по иконке
                }
            });

            ui.add_space(10.0);

            // ======= СТРОКА 1 =======
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Срок инвестиций (годы):"));
                    ui.add_space(4.0);
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.invest_duration_text).hint_text("35"));
                });

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Годовая процентная ставка:"));
                    ui.add_space(4.0);
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.interest_text).hint_text("11"));
                });
            });

            ui.add_space(edit_space); // Вертикальный отступ между строками формы

            // ======= СТРОКА 2 =======
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Начальный капитал:"));
                    ui.add_space(4.0); // Отступ между текстом и полем
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.start_deposit_text).hint_text("1000000"));
                });

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Ежемесячное пополнение:"));
                    ui.add_space(4.0);
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.deposit_text).hint_text("60000"));
                });
            });

            ui.add_space(edit_space); // Вертикальный отступ между строками формы

            // ======= СТРОКА 3 =======
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Сколько лет делаем пополнения:"));
                    ui.add_space(4.0); // Отступ между текстом и полем
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.deposit_years_text).hint_text("25"));
                });

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Размер ежемесячного изъятия (расходы):"));
                    ui.add_space(4.0);
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.withdraw_text).hint_text("100000"));
                });
            });

            ui.add_space(edit_space); // Вертикальный отступ между строками формы

            // ======= СТРОКА 4 =======
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Инвест год, начала изъятий:"));
                    ui.add_space(4.0); // Отступ между текстом и полем
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.withdraw_start_text).hint_text("26"));
                });

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Годовой размер инфляции, проценты:"));
                    ui.add_space(4.0);
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.inflation_text).hint_text("5"));
                });
            });

            ui.add_space(edit_space); // Вертикальный отступ между строками формы

            // ======= СТРОКА 5 =======
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Годовой рост пополнения, проценты:"));
                    ui.add_space(4.0); // Отступ между текстом и полем
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.salary_grow_text).hint_text("3"));
                });

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Реинвестирования процентов, месяцы (3,6,12):"));
                    ui.add_space(4.0);
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.reinvest_duration_text).hint_text("3"));
                });
            });

            ui.add_space(edit_space); // Вертикальный отступ между строками формы

            // ======= СТРОКА 6 =======
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Возраст инвестора на первый инвест год:"));
                    ui.add_space(4.0); // Отступ между текстом и полем
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.age_of_investor_text).hint_text("37"));
                });

                ui.vertical(|ui| {
                    ui.add(egui::Label::new("Календарный год на первый инвест год:"));
                    ui.add_space(4.0);
                    ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.calendar_year_text).hint_text("2026"));
                });
            });

            ui.add_space(15.0); // Вертикальный отступ между строками формы

            // Просто для демонстрации, что текст успешно сохраняется:
            //ui.label(format!("Вы ввели: {} и {} и {} и {}", self.first_name, self.last_name, self.phone, self.email));
        });
        
    }
}

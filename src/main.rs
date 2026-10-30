#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn configure_app_styles(cc: &eframe::CreationContext<'_>) {
    // 1. Настройка темы Theme::Dark / Theme::Light
    cc.egui_ctx.set_theme(egui::Theme::Light);
    
    // 2. Поддержка кнопок с иконками
    egui_extras::install_image_loaders(&cc.egui_ctx);
    
    // 3. Настройка рамок
    let mut visuals = cc.egui_ctx.style().visuals.clone();
    
    // Задаем постоянную серую рамку для полей ввода
    visuals.widgets.inactive.bg_stroke = egui::Stroke {
        width: 1.0,
        color: egui::Color32::from_gray(180), 
    };
                
    // Применяем стили один раз на всё время работы приложения
    cc.egui_ctx.set_visuals(visuals);

    // 4. Подключаем свой шрифт для корректного рисования символа рубля ₽
    let mut fonts = egui::FontDefinitions::default();

    // Вшиваем файл шрифта в бинарник
    fonts.font_data.insert(
        "my_font_roboto".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/Roboto.ttf"
        ))),
    );

    // Устанавливаем этот шрифт приоритетным для обычного текста (Proportional)
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font_roboto".to_owned());

    // Устанавливаем его же для моноширинного текста (например, в полях ввода, если нужно)
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "my_font_roboto".to_owned());

    // Передаем обновленную коллекцию шрифтов в контекст
    cc.egui_ctx.set_fonts(fonts);
}

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

            configure_app_styles(cc);

            Ok(Box::new(MyApp::new(cc)))
        }),  
    )
}  

// Структура для одной строки нашей таблицы расчетов
struct HistoryRow {
    invest_year: usize,
    age: usize,
    calendar_year: usize,
    deposits: f64,
    withdraw: f64,
    interest: f64,
    end_balance: f64,
}

// Для 3х комбо боксов, в fn draw_text_edits

// 1. Период реинвестирования
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReinvestPeriod {
    Months3 = 3,
    Months6 = 6,
    Months12 = 12,
}

impl ReinvestPeriod {
    fn to_string(&self) -> &'static str {
        match self {
            Self::Months3 => "3 месяца",
            Self::Months6 => "6 месяцев",
            Self::Months12 => "12 месяцев (раз в год)",
        }
    }
}

// 2. Валюта инвестиций
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Rub,
    Usd,
    Eur,
    Gbp,
    Cny,
}

impl Currency {
    fn to_string(&self) -> &'static str {
        match self {
            Self::Rub => "₽ (Рубль)",
            Self::Usd => "$ (Доллар США)",
            Self::Eur => "€ (Евро)",
            Self::Gbp => "£ (Фунт стерлингов)",
            Self::Cny => "¥ (Юань)",
        }
    }
}

// 3. Частота пополнения
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepositFrequency {
    Monthly,
    EveryTwoMonths,
    Quarterly,
    SemiAnnually,
}

impl DepositFrequency {
    fn to_string(&self) -> &'static str {
        match self {
            Self::Monthly => "Ежемесячно",
            Self::EveryTwoMonths => "Через месяц",
            Self::Quarterly => "Ежеквартально",
            Self::SemiAnnually => "Раз в полгода",
        }
    }
}

//===============================================================================

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
    age_of_investor_text: String,   // Возраст инвестора на первый год инвестиций
    calendar_year_text: String,     // Календарный год на первый год инвестиций

    calc_result_rows: Vec<HistoryRow>,

    // Переменные для выпадающих списков
    reinvest_period: ReinvestPeriod,     // Период реинвестирования процентов
    currency: Currency,                  // Выбор валюты
    deposit_frequency: DepositFrequency, // Частота пополнения инвестиций
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
            age_of_investor_text: String::new(),
            calendar_year_text: String::new(),

            calc_result_rows: Vec::new(),

            // Комбо боксы - задаем дефолтные значения при старте
            reinvest_period: ReinvestPeriod::Months3,
            currency: Currency::Rub,
            deposit_frequency: DepositFrequency::Monthly,
        }
    }

    // Инвестиционный рассчет по годам
    fn invest_calc(&mut self) {
        // Симулируем данные для демонстрации (например, расчет на 13 лет)
        // В вашем коде это будет реальный вектор, рассчитанный по формуле
        self.calc_result_rows = vec![
            HistoryRow { invest_year: 1,  age: 30, calendar_year: 2026, deposits: 260000.0, withdraw: 0.0, interest: 1100.0, end_balance: 17100.0 },
            HistoryRow { invest_year: 2,  age: 31, calendar_year: 2027, deposits: 260000.0, withdraw: 0.0, interest: 1700.0, end_balance: 24800.0 },
            HistoryRow { invest_year: 3,  age: 32, calendar_year: 2028, deposits: 260000.0, withdraw: 0.0, interest: 2300.0, end_balance: 33100.0 },
            HistoryRow { invest_year: 4,  age: 33, calendar_year: 2029, deposits: 260000.0, withdraw: 0.0, interest: 3000.0, end_balance: 42100.0 },
            HistoryRow { invest_year: 5,  age: 34, calendar_year: 2030, deposits: 260000.0, withdraw: 0.0, interest: 3700.0, end_balance: 51800.0 },
            HistoryRow { invest_year: 6,  age: 35, calendar_year: 2031, deposits: 260000.0, withdraw: 0.0, interest: 4300.0, end_balance: 63100.0 },
            HistoryRow { invest_year: 7,  age: 36, calendar_year: 2032, deposits: 260000.0, withdraw: 0.0, interest: 5000.0, end_balance: 72100.0 },
            HistoryRow { invest_year: 8,  age: 37, calendar_year: 2033, deposits: 260000.0, withdraw: 0.0, interest: 6700.0, end_balance: 81800.0 },
            HistoryRow { invest_year: 9,  age: 38, calendar_year: 2034, deposits: 0.0, withdraw: 93100.0, interest: 5000.0, end_balance: 92100.0 },
            HistoryRow { invest_year: 10, age: 39, calendar_year: 2035, deposits: 0.0, withdraw: 102100.0, interest: 6700.0, end_balance: 101800.0 },
            HistoryRow { invest_year: 11, age: 40, calendar_year: 2036, deposits: 0.0, withdraw: 112100.0, interest: 7700.0, end_balance: 111800.0 },
            HistoryRow { invest_year: 12, age: 41, calendar_year: 2037, deposits: 0.0, withdraw: 122100.0, interest: 8700.0, end_balance: 121800.0 },
        ];
        let new_interest = 9700.0;
        let new_invest_year: usize = 13;
        self.calc_result_rows.push(HistoryRow {
            invest_year: new_invest_year, 
            age: 42, 
            calendar_year: 2038, 
            deposits: 0.0, 
            withdraw: 132100.0, 
            interest: new_interest, 
            end_balance: 131800.0,
        });
    }

    fn draw_icon_buttons(&mut self, ui: &mut egui::Ui) {
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
                self.invest_duration_text = String::from("35");    // Длительность инвестиций, годы
                self.interest_text = String::from("11");           // Процентная ставка инвестиций, проценты
                self.start_deposit_text = String::from("1000000"); // Начальный капитал
                
                self.deposit_text = String::from("75000");       // Размер пополнения
                self.deposit_years_text = String::from("25");    // Сколько лет делаем пополнения
                self.withdraw_text = String::from("100000");     // Размер ежемесячного изъятия
                
                self.withdraw_start_text = String::from("26");   // Год, начала изъятий
                self.inflation_text = String::from("5");         // Годовой размер инфляции, проценты
                self.salary_grow_text = String::from("3");       // Годовой рост пополнения, проценты
                
                self.age_of_investor_text = String::from("44");  // Возраст инвестора на первый год инвестиций
                self.calendar_year_text = String::from("2026");  // Календарный год на первый год инвестиций

                // 3 комбо бокса
                self.reinvest_period = ReinvestPeriod::Months3;
                self.currency = Currency::Rub;
                self.deposit_frequency = DepositFrequency::Monthly;
            }
            if ui.add_sized(egui::vec2(130.0, 50.0), image_btn_save).clicked() {
                // Клик по иконке
            }
            if ui.add_sized(egui::vec2(110.0, 50.0), image_btn_lang).clicked() {
                // Клик по иконке
            }
        });
    }

    fn draw_text_edits(&mut self, ui: &mut egui::Ui) {
        
        // Задаем точные размеры для полей ввода [Ширина, Высота]
        let edit_size = egui::vec2(300.0, 24.0);
        let combo_box_width: f32 = 300.0;
        let edit_space: f32 = 7.0;

        // ======= СТРОКА 1 =======
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

            ui.vertical(|ui| {
                ui.add(egui::Label::new("Срок инвестиций (годы): *"));
                ui.add_space(4.0);
                ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.invest_duration_text).hint_text("35"));
            });

            ui.vertical(|ui| {
                ui.add(egui::Label::new("Годовая процентная ставка: *"));
                ui.add_space(4.0);
                ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.interest_text).hint_text("11"));
            });
        });

        ui.add_space(edit_space); // Вертикальный отступ между строками формы

        // ======= СТРОКА 2 =======
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

            ui.vertical(|ui| {
                ui.add(egui::Label::new("Начальный капитал: *"));
                ui.add_space(4.0); // Отступ между текстом и полем
                ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.start_deposit_text).hint_text("1000000"));
            });

            ui.vertical(|ui| {
                // --- КОМБОБОКС: Валюта ---
                ui.add(egui::Label::new("Валюта инвестиций: *"));
                ui.add_space(4.0);

                ui.spacing_mut().button_padding.y = 5.0;
                egui::ComboBox::from_id_salt("currency_combo")
                    .selected_text(self.currency.to_string())
                    .width(combo_box_width)
                    .show_ui(ui, |ui| {
                        ui.spacing_mut().button_padding.y = 3.0;
                        ui.selectable_value(&mut self.currency, Currency::Rub, Currency::Rub.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Usd, Currency::Usd.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Eur, Currency::Eur.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Gbp, Currency::Gbp.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Cny, Currency::Cny.to_string());
                    });
            });
        });

        ui.add_space(edit_space); // Вертикальный отступ между строками формы

        // ======= СТРОКА 3 =======
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

            ui.vertical(|ui| {
                ui.add(egui::Label::new("Ежемесячное пополнение: *"));
                ui.add_space(4.0);
                ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.deposit_text).hint_text("60000"));
            });

            ui.vertical(|ui| {
                // --- КОМБОБОКС: Частота пополнений ---
                ui.add(egui::Label::new("Частота пополнений: *"));
                ui.add_space(4.0);

                ui.spacing_mut().button_padding.y = 5.0;
                egui::ComboBox::from_id_salt("frequency_combo")
                    .selected_text(self.deposit_frequency.to_string())
                    .width(combo_box_width)
                    .show_ui(ui, |ui| {
                        ui.spacing_mut().button_padding.y = 3.0;
                        ui.selectable_value(&mut self.deposit_frequency, DepositFrequency::Monthly, DepositFrequency::Monthly.to_string());
                        ui.selectable_value(&mut self.deposit_frequency, DepositFrequency::EveryTwoMonths, DepositFrequency::EveryTwoMonths.to_string());
                        ui.selectable_value(&mut self.deposit_frequency, DepositFrequency::Quarterly, DepositFrequency::Quarterly.to_string());
                        ui.selectable_value(&mut self.deposit_frequency, DepositFrequency::SemiAnnually, DepositFrequency::SemiAnnually.to_string());
                    });
            });
        });

        ui.add_space(edit_space); // Вертикальный отступ между строками формы

        // ======= СТРОКА 4 =======
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

            ui.vertical(|ui| {
                ui.add(egui::Label::new("Сколько лет делаем пополнения: *"));
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

        // ======= СТРОКА 5 =======
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

        // ======= СТРОКА 6 =======
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

            ui.vertical(|ui| {
                ui.add(egui::Label::new("Годовой рост пополнения, проценты:"));
                ui.add_space(4.0); // Отступ между текстом и полем
                ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.salary_grow_text).hint_text("3"));
            });

            ui.vertical(|ui| {
                // --- КОМБОБОКС: Период реинвестирования ---
                ui.add(egui::Label::new("Реинвестирования процентов, месяцы: *"));
                ui.add_space(4.0);

                ui.spacing_mut().button_padding.y = 5.0;
                egui::ComboBox::from_id_salt("reinvest_combo")
                    .selected_text(self.reinvest_period.to_string())
                    .width(combo_box_width)
                    .show_ui(ui, |ui| {
                        ui.spacing_mut().button_padding.y = 3.0;
                        ui.selectable_value(&mut self.reinvest_period, ReinvestPeriod::Months3, ReinvestPeriod::Months3.to_string());
                        ui.selectable_value(&mut self.reinvest_period, ReinvestPeriod::Months6, ReinvestPeriod::Months6.to_string());
                        ui.selectable_value(&mut self.reinvest_period, ReinvestPeriod::Months12, ReinvestPeriod::Months12.to_string());
                    });
            });
        });

        ui.add_space(edit_space); // Вертикальный отступ между строками формы

        // ======= СТРОКА 7 =======
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 25.0; // Горизонтальный отступ между блоками ввода

            ui.vertical(|ui| {
                ui.add(egui::Label::new("Возраст инвестора на первый инвест год: *"));
                ui.add_space(4.0); // Отступ между текстом и полем
                ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.age_of_investor_text).hint_text("37"));
            });

            ui.vertical(|ui| {
                ui.add(egui::Label::new("Календарный год на первый инвест год: *"));
                ui.add_space(4.0);
                ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.calendar_year_text).hint_text("2026"));
            });
        });
    }

    fn draw_results_table(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Результаты моделирования роста вашего капитала по годам");
        ui.add_space(10.0);

        // Инициализируем построитель таблиц
        let header_height = 25.0;        // Высота заголовка        
        let row_height = 20.0;           // Высота одной строки данных
        let visible_rows_height = 220.0; // Высота таблицы
        
        egui_extras::TableBuilder::new(ui)
            .striped(true)    // Включаем чередование цветов строк (зебра) для читаемости
            .resizable(false) // Запрещаем пользователю менять ширину колонок вручную
            .vscroll(true)    // Включаем вертикальную полосу прокрутки
            .max_scroll_height(visible_rows_height) // Ограничиваем максимальную высоту тела таблицы
            .min_scrolled_height(0.0)
            .auto_shrink([false, false]) // Запрещаем таблице уменьшаться по вертикали и горизонтали
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center)) // Выравнивание внутри ячеек
            
            // Настраиваем 5 колонок и задаем им фиксированную или адаптивную ширину
            .column(egui_extras::Column::initial(30.0))   // Год инвестиций
            .column(egui_extras::Column::initial(50.0))   // Возраст инвестора
            .column(egui_extras::Column::initial(70.0))   // Календарный год
            .column(egui_extras::Column::initial(100.0))  // Пополнения за год
            .column(egui_extras::Column::initial(100.0))  // Изъятия за год
            .column(egui_extras::Column::initial(120.0))  // Проценты за год
            .column(egui_extras::Column::initial(130.0))  // Итоговый баланс
            
            // Задаем шапку таблицы
            .header(header_height, |mut header| {
                header.col(|ui| { ui.strong("Год"); });
                header.col(|ui| { ui.strong("Возраст"); });
                header.col(|ui| { ui.strong("Календарь"); });
                header.col(|ui| { ui.strong("Пополнения"); });
                header.col(|ui| { ui.strong("Изъятия"); });
                header.col(|ui| { ui.strong("Проценты"); });
                header.col(|ui| { ui.strong("Итог года"); });
            })
            
            // Заполняем таблицу данными
            .body(|body| {
                // В метод rows передаем общее количество строк и высоту одной строки
                body.rows(row_height, self.calc_result_rows.len(), |mut row| {
                    let row_index = row.index();
                    let data = &self.calc_result_rows[row_index];

                    // Заполняем ячейки текущей строки строго по порядку колонок
                    row.col(|ui| { ui.label(data.invest_year.to_string()); });
                    
                    row.col(|ui| { 
                        ui.label(egui::RichText::new(data.age.to_string()).color(egui::Color32::from_rgb(160, 70, 160))); 
                    });
                    row.col(|ui| { 
                        ui.label(egui::RichText::new(data.calendar_year.to_string()).color(egui::Color32::from_rgb(160, 70, 160))); 
                    });

                    row.col(|ui| { ui.label(format!("{:.2} ₽", data.deposits)); });
                    row.col(|ui| { ui.label(format!("{:.2} $", data.withdraw)); });
                    
                    row.col(|ui| { 
                        ui.label(egui::RichText::new(format!("+{:.2} ₽", data.interest)).color(egui::Color32::from_rgb(46, 204, 113))); 
                    });
                    
                    row.col(|ui| { 
                        ui.label(egui::RichText::new(format!("+{:.2} €", data.end_balance)).color(egui::Color32::from_rgb(0, 150, 200))); 
                    });
                });
            });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🖩 Расширенный инвест калькулятор с пополнением, изъятием, инфляцией и ростом зарплаты");
            ui.add_space(10.0);

            //================================================================
            // Отрисовка 4х кнопок с иконками
            //================================================================
            self.draw_icon_buttons(ui);
            ui.add_space(15.0);

            //================================================================
            // Отрисовка 12 labels и text edits
            //================================================================
            self.draw_text_edits(ui);
            ui.add_space(15.0);

            //================================================================
            // Рассчет и трисовка таблицы с результатами
            //================================================================
            self.invest_calc();
            self.draw_results_table(ui);
            ui.add_space(10.0);
        });
        
    }
}

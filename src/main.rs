#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod utility;

// Подлкючаем enums и structs
use utility::{ CalcHistoryRow, ReinvestPeriod, Currency, DepositFrequency, AssetClass, PortfolioAsset };
// Подлкючаем функции
use utility::{ format_thousands, get_russian_years_suffix};

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

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

    // 4. Подключаем свой шрифт для корректного рисования всех символов валют
    let mut fonts = egui::FontDefinitions::default();

    // Вшиваем файл шрифта в бинарник
    fonts.font_data.insert(
        "my_font_segoeui".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/segoeui.ttf"
        ))),
    );

    // Устанавливаем этот шрифт приоритетным для обычного текста (Proportional)
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font_segoeui".to_owned());

    // Устанавливаем его же для моноширинного текста (например, в полях ввода, если нужно)
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "my_font_segoeui".to_owned());

    // Передаем обновленную коллекцию шрифтов в контекст
    cc.egui_ctx.set_fonts(fonts);
}

//=======================================================================================
// ===== MAIN =====
//=======================================================================================
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

    // Настраиваем окно
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1340.0, 900.0]) // Размер окна приложения
            .with_icon(icon_data)             // Добавляем иконку сюда
           	.with_resizable(false)            // Запрещаем менять размер окна
	        .with_maximize_button(false),     // Отключаем кнопку развертывания на весь экран
        ..Default::default()
    };

    eframe::run_native(  
        "FoxyCalcRust v0.1",
        native_options,  
        Box::new(|cc| {

            configure_app_styles(cc);

            Ok(Box::new(MyApp::new(cc)))
        }),  
    )
}

//=======================================================================================
// MyApp
//=======================================================================================
struct MyApp {
    // Буферы для хранения текста из полей ввода
    invest_duration_text: String,   // Длительность инвестиций, годы
    interest_text: String,          // Процентная ставка инвестиций, проценты
    start_deposit_text: String,     // Начальный капитал
    deposit_text: String,           // Размер пополнения
    deposit_years_text: String,     // Сколько лет делаем пополнения
    withdraw_text: String,          // Размер ежемесячного изъятия (поле не обязательное)
    withdraw_start_text: String,    // Год, начала изъятий (поле не обязательное)
    inflation_text: String,         // Годовой размер инфляции, проценты (поле не обязательное)
    salary_grow_text: String,       // Годовой рост пополнения, проценты (поле не обязательное)
    investor_age_text: String,      // Возраст инвестора на первый год инвестиций
    calendar_year_text: String,     // Календарный год на первый год инвестиций

    // Вектор для сохранения результатов инвест рассчетов
    calc_result_rows: Vec<CalcHistoryRow>,

    // Переменные для выпадающих списков
    reinvest_period: ReinvestPeriod,     // Период реинвестирования процентов
    currency: Currency,                  // Выбор валюты
    deposit_frequency: DepositFrequency, // Частота пополнения инвестиций

    currency_sign: char, // Символ валюты для таблицы

    // Сохраняем результаты парсинга сюда, далее пойдет на вход для рассчета инвестиций по годам
    invest_duration_num: usize, // Длительность инвестиций, годы (целое число)
    interest_num: f64,          // Процентная ставка инвестиций, проценты (дробное число)
    start_deposit_num: f64,     // Начальный капитал (дробное число)
    deposit_num: f64,           // Размер пополнения (дробное число)
    deposit_years_num: usize,   // Сколько лет делаем пополнения (целое число)
    withdraw_num: f64,          // Размер ежемесячного изъятия (дробное число)
    withdraw_start_num: usize,  // Инвест год, начала изъятий (целое число)
    inflation_num: f64,         // Годовой размер инфляции, проценты (дробное число)
    salary_grow_num: f64,       // Годовой рост пополнения, проценты (дробное число)
    investor_age_num: usize,    // Возраст инвестора на первый год инвестиций (целое число)
    calendar_year_num: usize,   // Календарный год на первый год инвестиций (целое число)

    error_message: Option<String>, // Поле для предупреждений об ошибках парсинга

    calc_time_duration: f64, // Время расчета таблицы

    // Поля для моделирования инвест портфеля
    portfolio_assets: Vec<PortfolioAsset>, // Массив добавленных активов
    selected_asset_class: AssetClass,      // Выбранный элемент в комбобоксе классических активов
    asset_amount_text: String,             // Текст стоимости из эдита
    custom_asset_name: String,             // Текст имени кастомного актива из эдита
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
            investor_age_text: String::new(),
            calendar_year_text: String::new(),

            calc_result_rows: vec![
                CalcHistoryRow { invest_year: 0,  investor_age: 0, calendar_year: 0, deposits: 0.0, withdraw: 0.0, interest: 0.0, end_balance: 0.0 },
                CalcHistoryRow { invest_year: 0,  investor_age: 0, calendar_year: 0, deposits: 0.0, withdraw: 0.0, interest: 0.0, end_balance: 0.0 },
                CalcHistoryRow { invest_year: 0,  investor_age: 0, calendar_year: 0, deposits: 0.0, withdraw: 0.0, interest: 0.0, end_balance: 0.0 },
                CalcHistoryRow { invest_year: 0,  investor_age: 0, calendar_year: 0, deposits: 0.0, withdraw: 0.0, interest: 0.0, end_balance: 0.0 },
                CalcHistoryRow { invest_year: 0,  investor_age: 0, calendar_year: 0, deposits: 0.0, withdraw: 0.0, interest: 0.0, end_balance: 0.0 },
                CalcHistoryRow { invest_year: 0,  investor_age: 0, calendar_year: 0, deposits: 0.0, withdraw: 0.0, interest: 0.0, end_balance: 0.0 },
                CalcHistoryRow { invest_year: 0,  investor_age: 0, calendar_year: 0, deposits: 0.0, withdraw: 0.0, interest: 0.0, end_balance: 0.0 },
                CalcHistoryRow { invest_year: 0,  investor_age: 0, calendar_year: 0, deposits: 0.0, withdraw: 0.0, interest: 0.0, end_balance: 0.0 },
                CalcHistoryRow { invest_year: 0,  investor_age: 0, calendar_year: 0, deposits: 0.0, withdraw: 0.0, interest: 0.0, end_balance: 0.0 },
                CalcHistoryRow { invest_year: 0,  investor_age: 0, calendar_year: 0, deposits: 0.0, withdraw: 0.0, interest: 0.0, end_balance: 0.0 },
            ],

            // Комбо боксы - задаем дефолтные значения при старте
            reinvest_period: ReinvestPeriod::Months3,
            currency: Currency::Rub,
            deposit_frequency: DepositFrequency::Monthly,

            currency_sign: '₽', // Символ валюты для таблицы

            invest_duration_num: 0,
            interest_num: 0.0,
            start_deposit_num: 0.0,
            deposit_num: 0.0,
            deposit_years_num: 0,
            withdraw_num: 0.0,
            withdraw_start_num: 0,
            inflation_num: 0.0,
            salary_grow_num: 0.0,
            investor_age_num: 0,
            calendar_year_num: 0,

            error_message: None, // Сообщение об ошибке парсинга

            calc_time_duration: 0.0, // Время расчета таблицы
        
            // Поля для моделирования инвест портфеля
            portfolio_assets: Vec::new(),
            selected_asset_class: AssetClass::Cash,
            asset_amount_text: String::new(),
            custom_asset_name: String::new(),
        }
    }

    //=======================================================================================
    // Инвестиционный рассчет по годам
    //=======================================================================================
    fn invest_calc(&mut self) {
        
        // Очищаем результаты прошлых расчетов
        self.calc_result_rows.clear();
        
        // Символ валюты
        self.currency_sign = self.currency.to_string().chars().next().unwrap_or('?');

        // Извлекаем шаг реинвестирования (в месяцах) из энума
        let reinvest_step = match self.reinvest_period {
            ReinvestPeriod::Months3 => 3,
            ReinvestPeriod::Months6 => 6,
            ReinvestPeriod::Months12 => 12,
        };

        // Переводим годовую ставку в месячную (сложный процент)
        // interest_num задан в процентах (например, 12.0 -> 0.12)
        let annual_rate = self.interest_num / 100.0;
        let monthly_rate = (1.0 + annual_rate).powf(1.0 / 12.0) - 1.0;

        // Инициализируем переменные состояния для расчета
        let mut current_balance = self.start_deposit_num;
        let mut current_monthly_deposit = self.deposit_num;
        
        // Переменная для накопления процентов между периодами реинвестирования
        let mut accumulated_interest = 0.0;

        // Внешний цикл: Идем по инвестиционным годам (от 1 до invest_duration_num)
        for year_idx in 1..=self.invest_duration_num {
            // Текущие параметры инвестора и календаря для этой строки (этого года)
            let current_investor_age = self.investor_age_num + (year_idx - 1);
            let current_calendar_year = self.calendar_year_num + (year_idx - 1);

            // Счетчики для накопления годовой статистики по строке
            let mut yearly_deposits = 0.0;
            let mut yearly_withdraws = 0.0;
            let mut yearly_interest_earned = 0.0;

            // Внутренний цикл: Идем по месяцам внутри текущего года (1..=12)
            for month in 1..=12 {
                
                // 1. ПОПОЛНЕНИЕ КАПИТАЛА (Проверяем, укладываемся ли в доступные для пополнения годы)
                if year_idx <= self.deposit_years_num {
                    let is_deposit_month = match self.deposit_frequency {
                        DepositFrequency::Monthly => true,
                        DepositFrequency::EveryTwoMonths => month % 2 == 1, // Каждый нечетный месяц (1, 3, 5...)
                        DepositFrequency::Quarterly => month % 3 == 1,      // Раз в квартал (1, 4, 7, 10)
                        DepositFrequency::SemiAnnually => month == 1 || month == 7, // Раз в полгода
                    };

                    if is_deposit_month {
                        current_balance += current_monthly_deposit;
                        yearly_deposits += current_monthly_deposit;
                    }
                }

                // 2. ИЗЪЯТИЕ СРЕДСТВ (Ежемесячно, если наступил или прошел год начала изъятий) и если изъятие было заданно
                if self.withdraw_start_num > 0 && self.withdraw_num > 0.0 && year_idx >= self.withdraw_start_num {
                    // Изъятие происходит только если на балансе есть деньги
                    let actual_withdraw = if current_balance >= self.withdraw_num {
                        self.withdraw_num
                    } else {
                        current_balance // Забираем остаток, если на балансе меньше нужной суммы
                    };

                    current_balance -= actual_withdraw;
                    yearly_withdraws += actual_withdraw;
                }

                // 3. РАСЧЕТ И НАКОПЛЕНИЕ НАЧИСЛЕННЫХ ПРОЦЕНТОВ ЗА МЕСЯЦ
                let monthly_interest = current_balance * monthly_rate;
                accumulated_interest += monthly_interest;
                yearly_interest_earned += monthly_interest;

                // 4. КАПИТАЛИЗАЦИЯ (РЕИНВЕСТИРОВАНИЕ) ПРОЦЕНТОВ
                // Проценты падают на баланс только в определенные месяцы (например, каждые 3, 6 или 12 месяцев)
                if month % reinvest_step == 0 {
                    current_balance += accumulated_interest;
                    accumulated_interest = 0.0; // Сбрасываем накопитель для следующего периода
                }
            }

            // Если в конце года остались нереинвестированные проценты (для безопасности)
            if accumulated_interest > 0.0 {
                current_balance += accumulated_interest;
                accumulated_interest = 0.0;
            }

            // Сохраняем результаты за весь прошедший год в структуру строки таблицы
            self.calc_result_rows.push(CalcHistoryRow {
                invest_year: year_idx,
                investor_age: current_investor_age,
                calendar_year: current_calendar_year,
                deposits: yearly_deposits,
                withdraw: yearly_withdraws,
                interest: yearly_interest_earned,
                end_balance: current_balance,
            });

            // 5. ИНДЕКСАЦИЯ НА ИНФЛЯЦИЮ И РОСТ ПОПОЛНЕНИЙ (Применяется в конце каждого года на будущий год)
            // Рост размера пополнений (например, рост зарплаты), если он был задан
            if self.salary_grow_num > 0.0 {
                let salary_grow_factor = 1.0 + (self.salary_grow_num / 100.0);
                current_monthly_deposit *= salary_grow_factor;
            }

            // 6. Корректировка изъятий на размер инфляции (чтобы сохранять покупательскую способность)
            // Инфляция начинает работу только с первого года изъятия, если эти 3 необходимх параметра были заданны
            if self.withdraw_start_num > 0 && self.withdraw_num > 0.0 && self.inflation_num > 0.0 && year_idx >= self.withdraw_start_num {
                let inflation_factor = 1.0 + (self.inflation_num / 100.0);
                self.withdraw_num *= inflation_factor;
            }
        }
    }

    // Парсинг текстовых полей в числовые входные данные для рассчета
    fn text_edits_parsing(&mut self) -> bool {

        // 1. Автоподстановка "0" для пустых или состоящих из пробелов необязательных полей
        if self.withdraw_text.trim().is_empty() { self.withdraw_text = "0".to_string(); }
        if self.withdraw_start_text.trim().is_empty() { self.withdraw_start_text = "0".to_string(); }
        if self.inflation_text.trim().is_empty() { self.inflation_text = "0".to_string(); }
        if self.salary_grow_text.trim().is_empty() { self.salary_grow_text = "0".to_string(); }

        // Хелпер для парсинга и валидации дробных чисел с проверкой диапазона
        let parse_f64 = |text: &str, name: &str, min: f64, max: f64| -> Result<f64, String> {
            let val: f64 = text.trim().replace(',', ".").parse()
                .map_err(|_| format!("Ошибка. Поле '{}' должно быть числом (например: 12.5)", name))?;
            if val < min || val > max {
                return Err(format!("Ошибка. Значение в поле '{}' должно быть в диапазоне от {} до {}", name, min, max));
            }
            Ok(val)
        };

        // Хелпер для парсинга и валидации целых чисел с проверкой диапазона
        let parse_usize = |text: &str, name: &str, min: usize, max: usize| -> Result<usize, String> {
            let val: usize = text.trim().parse()
                .map_err(|_| format!("Ошибка. Поле '{}' должно быть целым положительным числом", name))?;
            if val < min || val > max {
                return Err(format!("Ошибка. Значение в поле '{}' должно быть в диапазоне от {} до {}", name, min, max));
            }
            Ok(val)
        };

        // Запуск последовательной валидации всех полей
        let result = (|| -> Result<(), String> {
            self.invest_duration_num = parse_usize(&self.invest_duration_text, "Длительность инвестиций", 1, 100)?;
            self.interest_num = parse_f64(&self.interest_text, "Процентная ставка", 0.0, 100.0)?;
            self.start_deposit_num = parse_f64(&self.start_deposit_text, "Начальный капитал", 0.0, f64::MAX)?;
            self.deposit_num = parse_f64(&self.deposit_text, "Размер пополнения", 0.0, f64::MAX)?;
            self.deposit_years_num = parse_usize(&self.deposit_years_text, "Сколько лет делаем пополнения", 0, self.invest_duration_num)?;
            self.withdraw_num = parse_f64(&self.withdraw_text, "Размер ежемесячного изъятия", 0.0, f64::MAX)?;
            self.withdraw_start_num = parse_usize(&self.withdraw_start_text, "Инвест год начала изъятий", 0, self.invest_duration_num + 1)?;
            self.inflation_num = parse_f64(&self.inflation_text, "Размер инфляции", 0.0, 100.0)?;
            self.salary_grow_num = parse_f64(&self.salary_grow_text, "Рост пополнения", 0.0, 100.0)?;
            self.investor_age_num = parse_usize(&self.investor_age_text, "Возраст инвестора", 0, 120)?;
            self.calendar_year_num = parse_usize(&self.calendar_year_text, "Календарный год", 1800, 2200)?;
            
            // Дополнительные кросс-валидации
            if self.deposit_years_num > self.invest_duration_num {
                return Err("Ошибка. Срок пополнений не может быть больше всей длительности инвестиций".to_string());
            }
            if self.withdraw_num > 0.0 && self.withdraw_start_num == 0 {
                return Err("Ошибка. Вы задали значение для изъятий, но не задали корректный год начала для них".to_string());
            }
            if self.inflation_num > 0.0 && self.withdraw_num == 0.0 && self.withdraw_start_num == 0 {
                return Err("Ошибка. Вы задали инфляцию для изъятий (рассходов), но не задали размер изъятия и корректный год начала списаний".to_string());
            }

            Ok(())
        })();

        // Обработка результата
        match result {
            Ok(_) => {
                self.error_message = None;
                true // Парсинг успешен, можно считать
            }
            Err(err_msg) => {
                self.error_message = Some(err_msg);
                false // Ошибка, расчет делать нельзя
            }
        }
    }

    // Функция отображения ошибки парсинга (MessageBox)
    fn error_messagebox_show(&mut self, ctx: &egui::Context) {
        if let Some(error_text) = &self.error_message {
            // Создаем окно
            let response = egui::Window::new("Ошибка валидации")
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)) // Всегда строго по центру
                .show(ctx, |ui| {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        // Красивая красная иконка предупреждения
                        ui.label(egui::RichText::new("⚠").color(egui::Color32::RED).size(32.0));
                        ui.label(error_text);
                    });
                    
                    ui.add_space(10.0);
                    
                    // Если пользователь нажал кнопку ОК, возвращаем true
                    // Выравниваем кнопку по центру окна
                    ui.vertical_centered(|ui| {
                        let ok_btn = egui::Button::new("Ok");
                        ui.add_sized(egui::vec2(100.0, 30.0), ok_btn).clicked()
                    }).inner // .inner вытаскивает bool (clicked) из замыкания vertical_centered
                });

            // Проверяем: закрыл ли пользователь окно (нажал ли на кнопку ОК)
            // Проверяем: если кнопка ОК была нажата, то внутри будет Some(true)
            // .flatten() превратит Option<Option<bool>> в простой Option<bool>
            if response.map(|res| res.inner).flatten().unwrap_or(false) {
                self.error_message = None;
            }
        }
    }

    // Запуск инвест рассчета - выносим всю цепочку действий в отдельную функцию для вызова из 2х разных мест
    fn run_invest_calculation(&mut self) {
        if self.text_edits_parsing() {
            // НАЧАЛО ЗАМЕРА
            let start_time = std::time::Instant::now();
            
            self.invest_calc();

            // КОНЕЦ ЗАМЕРА
            let duration = start_time.elapsed();
            self.calc_time_duration = duration.as_secs_f64() * 1000.0;
        }
    }

    // Добавляем в UI главные кнопки с иконками (toolbar)
    fn draw_icon_buttons(&mut self, ui: &mut egui::Ui) {
        let button_icon_size: f32 = 35.0;

        // Вшиваем иконку (например, лежащую в папке assets)
        let icon_source_calc = egui::include_image!("../assets/button_icon_calc.png");
        let icon_source_demo = egui::include_image!("../assets/button_icon_demo.png");
        let icon_source_export = egui::include_image!("../assets/button_icon_export.png");
        let icon_source_lang = egui::include_image!("../assets/button_icon_lang.png");
        let icon_source_about = egui::include_image!("../assets/button_icon_about.png");

        let icon_sized_calc = egui::Image::new(icon_source_calc)
            .fit_to_exact_size(egui::vec2(button_icon_size, button_icon_size)) // РАЗМЕР ИКОНКИ ТУТ
            .corner_radius(3.0);
        let icon_sized_demo = egui::Image::new(icon_source_demo)
            .fit_to_exact_size(egui::vec2(button_icon_size, button_icon_size)) 
            .corner_radius(3.0);
        let icon_sized_export = egui::Image::new(icon_source_export)
            .fit_to_exact_size(egui::vec2(button_icon_size, button_icon_size))
            .corner_radius(3.0);
        let icon_sized_lang = egui::Image::new(icon_source_lang)
            .fit_to_exact_size(egui::vec2(button_icon_size, button_icon_size))
            .corner_radius(3.0);
        let icon_sized_about = egui::Image::new(icon_source_about)
            .fit_to_exact_size(egui::vec2(button_icon_size, button_icon_size))
            .corner_radius(3.0);

        let image_btn_calc = egui::Button::image_and_text(icon_sized_calc, "Рассчитать");
        let image_btn_demo = egui::Button::image_and_text(icon_sized_demo, "Демо данные");
        let image_btn_export = egui::Button::image_and_text(icon_sized_export, "Экспорт");
        let image_btn_lang = egui::Button::image_and_text(icon_sized_lang, "Language");
        let image_btn_about = egui::Button::image_and_text(icon_sized_about, "О программе");
        ui.horizontal(|ui| {
            // Инвест рассчет
            if ui.add_sized(egui::vec2(130.0, 50.0), image_btn_calc).clicked() {
                self.run_invest_calculation();
            }
            // Загрузить демо данные
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
                
                self.investor_age_text = String::from("44");  // Возраст инвестора на первый год инвестиций
                self.calendar_year_text = String::from("2026");  // Календарный год на первый год инвестиций

                // 3 комбо бокса
                self.reinvest_period = ReinvestPeriod::Months3;
                self.currency = Currency::Rub;
                self.deposit_frequency = DepositFrequency::Monthly;

                // Заполняем текущий инвест портфель - активы
                self.portfolio_assets = vec![
                    PortfolioAsset { name: "Наличные".to_string(), value: 3000000.0 },
                    PortfolioAsset { name: "Банковский вклад".to_string(), value: 560000.0 },
                    PortfolioAsset { name: "Золото".to_string(), value: 5000000.0 },
                    PortfolioAsset { name: "Дивидиндные акции".to_string(), value: 7500000.0 },
                    PortfolioAsset { name: "Облигации".to_string(), value: 4400000.0 },
                    PortfolioAsset { name: "Криптовалюта".to_string(), value: 3000000.0 },
                    PortfolioAsset { name: "Недвижимость".to_string(), value: 25000000.0 },
                    PortfolioAsset { name: "Движимое имущество".to_string(), value: 5700000.0 },
                ];

                // Делаем инвест рассчет, заполнится таблица и нарисуется график
                self.run_invest_calculation();
            }
            if ui.add_sized(egui::vec2(120.0, 50.0), image_btn_export).clicked() {
                // Клик по иконке
            }
            if ui.add_sized(egui::vec2(120.0, 50.0), image_btn_lang).clicked() {
                // Клик по иконке
            }
            // О программе
            if ui.add_sized(egui::vec2(150.0, 50.0), image_btn_about).clicked() {
                // Клик по иконке            
            }
        });
    }

    // Добавляем в UI Edits и ComboBoxes
    fn draw_text_edits(&mut self, ui: &mut egui::Ui) {
        
        // Задаем точные размеры для полей ввода [Ширина, Высота]
        let edit_size = egui::vec2(310.0, 24.0);
        let combo_box_width: f32 = 310.0;
        let edit_space: f32 = 7.0;

        ui.heading("📈 Введите исходные данные для инвестиционный расчета");
        ui.add_space(10.0);

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
                        ui.selectable_value(&mut self.currency, Currency::Usd, Currency::Usd.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Eur, Currency::Eur.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Gbp, Currency::Gbp.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Chf, Currency::Chf.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Cny, Currency::Cny.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Rub, Currency::Rub.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Try, Currency::Try.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Inr, Currency::Inr.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Krw, Currency::Krw.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Thb, Currency::Thb.to_string());
                        ui.selectable_value(&mut self.currency, Currency::Vnd, Currency::Vnd.to_string());
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
                ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.investor_age_text).hint_text("37"));
            });

            ui.vertical(|ui| {
                ui.add(egui::Label::new("Календарный год на первый инвест год: *"));
                ui.add_space(4.0);
                ui.add_sized(edit_size, egui::TextEdit::singleline(&mut self.calendar_year_text).hint_text("2026"));
            });
        });
    }

    // Рисуем в UI таблицу с результатами инвест рассчета
    fn draw_results_table(&mut self, ui: &mut egui::Ui) {
        // Инициализируем построитель таблиц
        let header_height = 25.0;        // Высота заголовка        
        let row_height = 20.0;           // Высота одной строки данных
        let visible_rows_height = 220.0; // Высота таблицы (10 строк)
        
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
            // Дробная часть в таблице не отображается, маленькие пополнения, изъятия и пр. менее 1.0 будут нулями
            .body(|body| {
                // В метод rows передаем общее количество строк и высоту одной строки
                body.rows(row_height, self.calc_result_rows.len(), |mut row| {
                    let row_index = row.index();
                    let data = &self.calc_result_rows[row_index];

                    // Заполняем ячейки текущей строки строго по порядку колонок
                    row.col(|ui| { ui.label(data.invest_year.to_string()); });
                    
                    row.col(|ui| { 
                        ui.label(egui::RichText::new(data.investor_age.to_string())
                            .color(egui::Color32::from_rgb(160, 70, 160))); 
                    });
                    row.col(|ui| { 
                        ui.label(egui::RichText::new(data.calendar_year.to_string())
                            .color(egui::Color32::from_rgb(160, 70, 160))); 
                    });

                    // Столбец - пополнения
                    let deposits_amount = data.deposits as i64;
                    let deposits_str = if deposits_amount > 0 {
                        format!("+{} {}", format_thousands(deposits_amount), self.currency_sign)
                    } else {
                        format!("{} {}", format_thousands(deposits_amount), self.currency_sign)
                    };
                    row.col(|ui| { 
                        ui.label(egui::RichText::new(deposits_str)
                            .color(egui::Color32::from_rgb(100, 130, 100)));
                    });

                    // Столбец - изъятия
                    let withdraw_amount = data.withdraw as i64;
                    let withdraw_str = if withdraw_amount > 0 {
                        format!("-{} {}", format_thousands(withdraw_amount), self.currency_sign)
                    } else {
                        format!("{} {}", format_thousands(withdraw_amount), self.currency_sign)
                    };
                    row.col(|ui| {
                        ui.label(egui::RichText::new(withdraw_str)
                            .color(egui::Color32::from_rgb(200, 70, 70))); 
                    });                    

                    // Столбец - проценты
                    row.col(|ui| {
                        ui.label(egui::RichText::new(format!("+{} {}", format_thousands(data.interest as i64), self.currency_sign))
                            .color(egui::Color32::from_rgb(46, 204, 113))); 
                    });
                    
                    // Столбец - итоговый баланс года
                    row.col(|ui| {
                        ui.label(egui::RichText::new(format!("{} {}", format_thousands(data.end_balance as i64), self.currency_sign))
                            .color(egui::Color32::from_rgb(0, 140, 225))); 
                    });
                });
            });
    }


    // Рисуем график с ростом наших инвестиций (вторая колонка в UI)
    fn plot_graph(&mut self, ui: &mut egui::Ui) {
        if !self.calc_result_rows.is_empty() {
            ui.vertical(|ui| {
                let years_count = self.calc_result_rows.len();
                ui.heading(format!("⚡ Прогноз роста ваших инвестиций, {} {}", 
                    years_count, get_russian_years_suffix(years_count)));
                ui.add_space(10.0);

                // 1. Точки для ПЕРВОЙ линии (Итоговый капитал)
                let points_capital: PlotPoints = self.calc_result_rows
                    .iter()
                    .map(|row| [row.invest_year as f64, row.end_balance])
                    .collect();

                let line_capital = Line::new("Инвестиции", points_capital)
                    .color(egui::Color32::from_rgb(0, 140, 225)) // Синяя
                    .width(3.0_f32);

                // 2. Точки для ВТОРОЙ линии (Накопление пополнений с учетом стартового капитала)
                let mut cumulative_deposits = self.start_deposit_num; // Начинаем со стартового капитала
                let points_deposits: PlotPoints = self.calc_result_rows
                    .iter()
                    .map(|row| {
                        cumulative_deposits += row.deposits; // Плюсуем пополнения за текущий год
                        [row.invest_year as f64, cumulative_deposits]
                    })
                    .collect();

                let line_deposits = Line::new("Пополнения", points_deposits)
                    .color(egui::Color32::from_rgb(100, 130, 100)) // Темно-серый
                    .width(3.0_f32);

                // Формируем текст для оси Y, например: "Капитал, $"
                let y_label_text = format!("Капитал, {}", self.currency_sign);

                // Настройка самого холста графика
                Plot::new("investment_growth_plot")
                    .width(650.0)
                    .height(410.0)
                    // ДОБАВЛЯЕМ ТЕКСТОВЫЕ ПОДПИСИ ОСЕЙ ДЛЯ ВЕРСИИ 0.33
                    .x_axis_label("Срок инвестирования (годы)")
                    .y_axis_label(y_label_text)
                    .allow_zoom(true)
                    .allow_drag(true)
                    // ВКЛЮЧАЕМ ОТОБРАЖЕНИЕ ЛЕГЕНДЫ
                    .legend(egui_plot::Legend::default().position(egui_plot::Corner::LeftTop))
                    .show(ui, |plot_ui| {
                        // 1. Отрисовываем обе линии на одном холсте
                        plot_ui.line(line_capital);
                        plot_ui.line(line_deposits);

                        if self.withdraw_start_num > 0 {
                            // 2. Добавляем кастомную вертикальную линию (например, на 10-й инвест-год)
                            // Задаем координату X. Например, берем год начала изъятий
                            let target_x = self.withdraw_start_num as f64; 
                            
                            let vertical_line = egui_plot::VLine::new("Начало изъятий", target_x)
                                .color(egui::Color32::from_rgb(240, 70, 70)) // Мягкий красный цвет
                                .width(1.5_f32)
                                .style(egui_plot::LineStyle::Dashed { length: 6.0 }); // Делаем линию пунктирной
                                
                            plot_ui.vline(vertical_line);

                            // 3. Рисуем цветную точку-маркер на пересечении
                            // Находим строку данных для нужного года, чтобы узнать точный баланс (координату Y)
                            if let Some(row) = self.calc_result_rows.iter().find(|r| r.invest_year == self.withdraw_start_num) {
                                let target_y = row.end_balance;

                                // Создаем точку
                                let marker_point = egui_plot::Points::new("", vec![[target_x, target_y]])
                                    .color(egui::Color32::from_rgb(240, 70, 70)) // Ярко-красный цвет точки
                                    .radius(6.0_f32) // Размер (радиус) точки в пикселях
                                    .shape(egui_plot::MarkerShape::Circle); // Форма (Circle, Square, Diamond, Cross и др.)

                                plot_ui.points(marker_point);
                            }
                        }

                        // 4. Фича: оранжевая линия текущего портфеля
                        let total_portfolio_value: f64 = self.portfolio_assets.iter().map(|a| a.value).sum();

                        // Проверяем условия отображения: портфель не пуст, и его стоимость лежит между 
                        // первым балансом (стартовым капиталом) и финальным результатом
                        if total_portfolio_value > 0.0 && !self.calc_result_rows.is_empty() {
                            let first_year_balance = self.calc_result_rows.first().unwrap().end_balance;
                            let last_year_balance = self.calc_result_rows.last().unwrap().end_balance;

                            // Нам нужно определить, попадает ли портфель в диапазон прогноза
                            // Берем минимальное и максимальное значение из двух крайних точек (на случай падения графика)
                            let min_bound = first_year_balance.min(last_year_balance);
                            let max_bound = first_year_balance.max(last_year_balance);

                            if total_portfolio_value >= min_bound && total_portfolio_value <= max_bound {
                                // Горизонтальная линия через весь график на уровне Y = общая стоимость
                                let orange_line = egui_plot::HLine::new("Текущий портфель", total_portfolio_value)
                                    .color(egui::Color32::from_rgb(180, 180, 0)) // Темно желтый
                                    .width(1.5_f32)
                                    .style(egui_plot::LineStyle::Dashed { length: 5.0 }); // Пунктир

                                plot_ui.hline(orange_line);
                            }
                        }
                    });
            });
        } else {
            ui.label("Нажмите кнопку 'Рассчитать', чтобы увидеть график прогноза.");
        }
    }

    // Моделирование текущего инвестиционного портфеля
    /*fn draw_portfolio_manager(&mut self, ui: &mut egui::Ui) {
        ui.heading("💼 Ваш текущий инвестиционный портфель");
        ui.add_space(10.0);

        // Считаем общую стоимость портфеля
        let total_portfolio_value: f64 = self.portfolio_assets.iter().map(|a| a.value).sum();

        // Делим пространство под графиком (650px) на две половины
        ui.columns(2, |columns| {
            let [left_col, right_col] = columns else { panic!("In the draw_portfolio_manager function, 2 columns are expected in the UI") };

            // ================= ЛЕВАЯ ПОЛОВИНА: Форма добавления =================
            left_col.vertical(|ui| {
                let edit_width = egui::vec2(310.0, 24.0);
                let combo_box_width: f32 = 310.0;

                // 1. Комбобокс классов активов
                ui.label(egui::RichText::new("Выберите категорию актива: *"));
                ui.spacing_mut().button_padding.y = 4.0;
                egui::ComboBox::from_id_salt("portfolio_asset_class_combo")
                    .selected_text(self.selected_asset_class.to_string())
                    .width(combo_box_width)
                    .show_ui(ui, |ui| {
                        for class in AssetClass::all() {
                            ui.selectable_value(&mut self.selected_asset_class, *class, class.to_string());
                        }
                    });
                ui.add_space(8.0);

                // 2. Эдит для стоимости
                ui.label(egui::RichText::new("Стоимость актива: *"));
                //ui.add(egui::TextEdit::singleline(&mut self.asset_amount_text).desired_width(edit_width));
                ui.add_sized(edit_width, egui::TextEdit::singleline(&mut self.asset_amount_text).hint_text("25000"));
                ui.add_space(8.0);

                // 3. Эдит для специфического имени
                ui.label(egui::RichText::new("Свой актив (необязательно):"));
                //ui.add(egui::TextEdit::singleline(&mut self.custom_asset_name).desired_width(edit_width));
                ui.add_sized(edit_width, egui::TextEdit::singleline(&mut self.custom_asset_name).hint_text("Серебро"));
                ui.add_space(15.0);

                // Кнопки управления и лейбл суммы
                ui.horizontal(|ui| {
                    // Кнопка Добавить актив
                    if ui.add_sized(egui::vec2(140.0, 24.0), egui::Button::new("➕ Добавить актив")).clicked() {
                        // Валидация стоимости
                        let parsed_amount = self.asset_amount_text.trim().replace(',', ".");
                        match parsed_amount.parse::<f64>() {
                            Ok(amount) if amount >= 0.0 => {
                                let max_portfolio_assets: usize = 10;
                                if self.portfolio_assets.len() >= max_portfolio_assets {
                                    self.error_message = Some(format!("Максимальное количество активов ограничено {} строками!", max_portfolio_assets));
                                } else {
                                    // Определяем имя: если специфическое пустое, берем из комбобокса
                                    let asset_name = if self.custom_asset_name.trim().is_empty() {
                                        self.selected_asset_class.to_string().to_string()
                                    } else {
                                        self.custom_asset_name.trim().to_string()
                                    };

                                    self.portfolio_assets.push(PortfolioAsset {
                                        name: asset_name,
                                        value: amount,
                                    });

                                    // Очищаем поля ввода
                                    self.asset_amount_text.clear();
                                    self.custom_asset_name.clear();
                                }
                            }
                            _ => {
                                self.error_message = Some("Укажите корректную стоимость актива (положительное число)!".to_string());
                            }
                        }
                    }

                    // Кнопка Очистить
                    if ui.add_sized(egui::vec2(160.0, 24.0), egui::Button::new("🗑 Очистить портфель")).clicked() {
                        self.portfolio_assets.clear();
                    }
                });

                ui.add_space(12.0);
                // Оранжевый лейбл с общей суммой портфеля
                let yellow_color = egui::Color32::from_rgb(180, 180, 0);
                ui.label(
                    egui::RichText::new(format!(
                        "Размер вашего портфеля: {} {}", 
                        format_thousands(total_portfolio_value as i64), 
                        self.currency_sign
                    ))
                    .strong()
                    .size(14.0)
                    .color(yellow_color)
                );
            });

            // ================= ПРАВАЯ ПОЛОВИНА: Список добавленных активов =================
            right_col.vertical(|ui| {
                if self.portfolio_assets.is_empty() {
                    ui.weak("Портфель пуст. Добавьте активы слева.");
                } else {
                    let mut asset_to_remove = None;

                    // Циклом выводим строки элементов (максимум 7)
                    for (idx, asset) in self.portfolio_assets.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // 1. Имя актива (Readonly)
                            let mut name_tmp = asset.name.clone();
                            ui.add(egui::TextEdit::singleline(&mut name_tmp).interactive(false).desired_width(150.0));

                            // 2. Стоимость актива (Readonly с округлением)
                            let mut val_str = format!("{} {}", format_thousands(asset.value as i64), self.currency_sign);
                            ui.add(egui::TextEdit::singleline(&mut val_str).interactive(false).desired_width(110.0));

                            // 3. Кнопка-крестик для удаления
                            if ui.button(egui::RichText::new("❌").color(egui::Color32::RED)).clicked() {
                                asset_to_remove = Some(idx);
                            }
                        });
                    }

                    // Если была нажата кнопка удаления — вырезаем из вектора
                    if let Some(idx) = asset_to_remove {
                        self.portfolio_assets.remove(idx);
                    }
                }
            });
        });
    }*/

    // Моделирование текущего инвестиционного портфеля
    fn draw_portfolio_manager(&mut self, ui: &mut egui::Ui) {
        ui.heading("💼 Ваш текущий инвестиционный портфель");
        ui.add_space(10.0);

        // Считаем общую стоимость портфеля
        let total_portfolio_value: f64 = self.portfolio_assets.iter().map(|a| a.value).sum();

        // Делим пространство под графиком (650px) на две половины
        ui.columns(2, |columns| {
            let [left_col, right_col] = columns else { panic!("In the draw_portfolio_manager function, 2 columns are expected in the UI") };

            // ================= ЛЕВАЯ ПОЛОВИНА: Форма добавления =================
            left_col.vertical(|ui| {
                let edit_width = egui::vec2(310.0, 24.0);
                let combo_box_width: f32 = 310.0;

                // 1. Комбобокс классов активов
                ui.label(egui::RichText::new("Выберите категорию актива: *"));
                ui.spacing_mut().button_padding.y = 4.0;
                egui::ComboBox::from_id_salt("portfolio_asset_class_combo")
                    .selected_text(self.selected_asset_class.to_string())
                    .width(combo_box_width)
                    .show_ui(ui, |ui| {
                        for class in AssetClass::all() {
                            ui.selectable_value(&mut self.selected_asset_class, *class, class.to_string());
                        }
                    });
                ui.add_space(8.0);

                // 2. Эдит для стоимости
                ui.label(egui::RichText::new("Стоимость актива: *"));
                ui.add_sized(edit_width, egui::TextEdit::singleline(&mut self.asset_amount_text).hint_text("25000"));
                ui.add_space(8.0);

                // 3. Эдит для специфического имени
                ui.label(egui::RichText::new("Свой актив (необязательно):"));
                ui.add_sized(edit_width, egui::TextEdit::singleline(&mut self.custom_asset_name).hint_text("Серебро"));
                ui.add_space(15.0);

                // Кнопки управления и лейбл суммы
                ui.horizontal(|ui| {
                    let can_add = !self.asset_amount_text.trim().is_empty();
                    
                    // ИСПРАВЛЕНИЕ: Используем add_enabled + min_size для фиксации размера
                    let add_btn = egui::Button::new("➕ Добавить актив").min_size(egui::vec2(140.0, 24.0));
                    if ui.add_enabled(can_add, add_btn).clicked() {
                        // Валидация стоимости
                        let parsed_amount = self.asset_amount_text.trim().replace(',', ".");
                        match parsed_amount.parse::<f64>() {
                            Ok(amount) if amount > 0.0 => {
                                let max_portfolio_assets: usize = 10;
                                if self.portfolio_assets.len() >= max_portfolio_assets {
                                    self.error_message = Some(format!("Максимальное количество активов ограничено {} строками!", max_portfolio_assets));
                                } else {
                                    let asset_name = if self.custom_asset_name.trim().is_empty() {
                                        self.selected_asset_class.to_string().to_string()
                                    } else {
                                        self.custom_asset_name.trim().to_string()
                                    };

                                    self.portfolio_assets.push(PortfolioAsset {
                                        name: asset_name,
                                        value: amount,
                                    });

                                    self.asset_amount_text.clear();
                                    self.custom_asset_name.clear();
                                }
                            }
                            _ => {
                                self.error_message = Some("Укажите корректную стоимость актива (положительное число)!".to_string());
                            }
                        }
                    }

                    let has_assets = !self.portfolio_assets.is_empty();
                    
                    // ИСПРАВЛЕНИЕ: Используем add_enabled + min_size для фиксации размера
                    let clear_btn = egui::Button::new("🗑 Очистить портфель").min_size(egui::vec2(160.0, 24.0));
                    if ui.add_enabled(has_assets, clear_btn).clicked() {
                        self.portfolio_assets.clear();
                    }
                });

                ui.add_space(12.0);
                let yellow_color = egui::Color32::from_rgb(180, 180, 0);
                ui.label(
                    egui::RichText::new(format!(
                        "Размер вашего портфеля: {} {}", 
                        format_thousands(total_portfolio_value as i64), 
                        self.currency_sign
                    ))
                    .strong()
                    .size(14.0)
                    .color(yellow_color)
                );
            });

            // ================= ПРАВАЯ ПОЛОВИНА: Список добавленных активов =================
            right_col.vertical(|ui| {
                if self.portfolio_assets.is_empty() {
                    ui.weak("Портфель пуст. Добавьте активы слева.");
                } else {
                    let mut asset_to_remove = None;
                    
                    // Мягкий серый цвет для заднего фона текстовых полей
				    let readonly_bg = ui.visuals().code_bg_color; 
				
				    // Переопределяем цвет, который egui использует специально для подложки TextEdit
				    ui.visuals_mut().extreme_bg_color = readonly_bg;

                    // Циклом выводим строки элементов (максимум 10)
                    for (idx, asset) in self.portfolio_assets.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // 1. Имя актива (Readonly с серым фоном)
                            let mut name_tmp = asset.name.clone();
                            ui.add(egui::TextEdit::singleline(&mut name_tmp).interactive(false).desired_width(150.0));

                            // 2. Стоимость актива (Readonly с серым фоном)
                            let mut val_str = format!("{} {}", format_thousands(asset.value as i64), self.currency_sign);
                            ui.add(egui::TextEdit::singleline(&mut val_str).interactive(false).desired_width(110.0));

                            // 3. Кнопка-крестик для удаления
                            if ui.button(egui::RichText::new("❌").color(egui::Color32::RED)).clicked() {
                                asset_to_remove = Some(idx);
                            }
                        });
                    }

                    // Возвращаем глобальные стили виджетов в исходное состояние
                    ui.reset_style();

                    // Если была нажата кнопка удаления — вырезаем из вектора
                    if let Some(idx) = asset_to_remove {
                        self.portfolio_assets.remove(idx);
                    }
                }
            });
        });
    }


} // impl MyApp

//=======================================================================================
// Отрисовка интерфейса приложения
//=======================================================================================
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        egui::TopBottomPanel::bottom("status_bar")
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Выводим текст с форматированием до 4 знаков после запятой
                ui.label(format!("⏱ Расчет занял время: {:.4} мс", self.calc_time_duration));
                ui.separator();
                ui.label("ℹ Пополнения делаются в первый месяц интервала пополнения");
                ui.separator();
                ui.label("💾 При закрытии приложения исходные данные и инвест портфель сохраняются");

                // Добавим разделитель и выведем дополнительную информацию
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak("(c) Sidelnikov Dmitriy, 2026");
                });
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🖩 Расширенный инвестиционный калькулятор с пополнением, изъятием, инфляцией и ростом зарплаты");
            ui.add_space(10.0);

            //================================================================
            // Отрисовка 4х кнопок с иконками
            //================================================================
            self.draw_icon_buttons(ui);
            ui.add_space(15.0);

            //================================================================
            // Делим интерфейс строго на 2 столбца фиксированной пропорции
            //================================================================
            ui.columns(2, |columns| {
                // Увеличиваем отступ между колонками для читаемости
                // Разделяем массив на две независимые изменяемые ссылки
                let [left_ui, right_ui] = columns else { panic!("In the update function, 2 columns are expected in the UI") };

                // --- ЛЕВЫЙ СТОЛБЕЦ ---
                left_ui.vertical(|ui| {
                    // Отрисовка 12 labels и text edits
                    self.draw_text_edits(ui);
                    ui.add_space(20.0);

                    ui.heading("📊 Результаты моделирования роста вашего капитала по годам");
                    ui.add_space(10.0);
                    
                    // Рендерим таблицу. Теперь она ограничена шириной левой колонки
                    self.draw_results_table(ui);
                });

                // --- ПРАВЫЙ СТОЛБЕЦ ---
                right_ui.vertical(|ui| {
                    // Отображение графика роста инвестиций (теперь он точно влезет)
                    self.plot_graph(ui);
                    
                    // Сюда можно добавлять элементы, которые будут "ниже" графика
                    ui.add_space(15.0);
                    
                    // Рисуем UI для вашего текущего инвест портфеля
                    self.draw_portfolio_manager(ui);
                });
            });

            //================================================================
            // Отображение MessageBox об ошибке парсинга
            //================================================================
            self.error_messagebox_show(ctx);
        });        
    } // fn update
} // impl eframe::App for MyApp

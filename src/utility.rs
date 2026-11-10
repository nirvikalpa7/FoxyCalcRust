//=======================================================================================
// Модлуль для вспомогательнных функций и структур
//=======================================================================================

// Структура для одной строки нашей таблицы расчетов
pub struct CalcHistoryRow {
    pub invest_year: usize,   // Год инвестирования
    pub investor_age: usize,  // Возраст инвестора
    pub calendar_year: usize, // Календарный год
    pub deposits: f64,        // Внесенные за год деньги
    pub withdraw: f64,        // Изъятые за год деньги
    pub interest: f64,        // Полученный за год процент
    pub end_balance: f64,     // Итоговый баланс капитала
}

//=======================================================================================
// Для 3х комбо боксов, в fn draw_text_edits
//=======================================================================================

// 1. Период реинвестирования
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReinvestPeriod {
    Months3 = 3,
    Months6 = 6,
    Months12 = 12,
}

impl ReinvestPeriod {
    pub fn to_string(&self) -> &'static str {
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
    Usd,
    Eur,
    Gbp,
    Chf,
    Cny,
    Rub,
    Try,
    Inr,
    Krw,
    Thb,
    Vnd,
}

impl Currency {
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Usd => "$ (Доллар США)",
            Self::Eur => "€ (Евро)",
            Self::Gbp => "£ (Фунт стерлингов)",
            Self::Chf => "₣ (Швейцарский франк)",
            Self::Cny => "¥ (Юань/Йена)",
            Self::Rub => "₽ (Рубль)",
            Self::Try => "₺ (Турецкая лира)",
            Self::Inr => "₹ (Индийская рупия)",
            Self::Krw => "₩ (Корейская вона)",
            Self::Thb => "฿ (Тайский бат)",
            Self::Vnd => "₫ (Вьетнамский донг)",
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
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Monthly => "Ежемесячно",
            Self::EveryTwoMonths => "Через месяц",
            Self::Quarterly => "Ежеквартально",
            Self::SemiAnnually => "Раз в полгода",
        }
    }
}

// Вспомогательная функция для разделения тысяч пробелами (например: 1520300 -> "1 520 300")
pub fn format_thousands(mut n: i64) -> String {
    if n == 0 { return "0".to_string(); }
    let mut result = String::new();
    let is_negative = n < 0;
    if is_negative { n = n.abs(); }
    
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    
    for (i, &byte) in bytes.iter().enumerate() {
        result.push(byte as char);
        if (len - i - 1) % 3 == 0 && i != len - 1 {
            result.push(' ');
        }
    }
    if is_negative { result.insert(0, '-'); }
    result
}

// Возвращает правильную форму слова "год/года/лет" в зависимости от числа
pub fn get_russian_years_suffix(years: usize) -> &'static str {
    let remainder_10 = years % 10;
    let remainder_100 = years % 100;

    if (11..=19).contains(&remainder_100) {
        "лет" // Исключения: 11-19, 111-119 и т.д.
    } else {
        match remainder_10 {
            1 => "год",
            2 | 3 | 4 => "года",
            _ => "лет", // 0, 5, 6, 7, 8, 9
        }
    }
}

//=======================================================================================
// Моделирование текущего инвестиционного портфеля
//=======================================================================================

// Классические типы активов для комбобокса
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetClass {
    Cash,
    BankDeposit,
    Gold,
    PreciousMetals,
    Stocks,
    DividendStocks,
    Bonds,
    Funds,
    Crypto,
    RealEstate,
    CommercialRealEstate,
    MovableProperty,
}

impl AssetClass {
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Cash => "Наличные",
            Self::BankDeposit => "Банковский вклад",
            Self::Gold => "Золото",
            Self::PreciousMetals => "Драгоценные металлы",
            Self::Stocks => "Акции",
            Self::DividendStocks => "Дивидендные акции",
            Self::Bonds => "Облигации",
            Self::Funds => "Фонды",
            Self::Crypto => "Криптовалюта",
            Self::RealEstate => "Недвижимость",
            Self::CommercialRealEstate => "Коммерческая недвижимость",
            Self::MovableProperty => "Движимое имущество",
        }
    }
    
    pub fn all() -> &'static [Self] {
        &[
            Self::Cash, Self::BankDeposit, Self::Gold, Self::PreciousMetals,
            Self::Stocks, Self::DividendStocks, Self::Bonds, Self::Funds,
            Self::Crypto, Self::RealEstate, Self::CommercialRealEstate, Self::MovableProperty
        ]
    }
}

// Структура одного добавленного актива
#[derive(Clone)]
pub struct PortfolioAsset {
    pub name: String,
    pub value: f64,
}

//=======================================================================================

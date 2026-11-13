//=======================================================================================
// Модлуль для функции инвестиционного расчета по годам
//=======================================================================================

use crate::utility::{CalcHistoryRow, ReinvestPeriod, DepositFrequency};

// Структура, объединяющая все спарсенные числовые параметры
#[derive(Debug, Clone, Copy)]
pub struct CalcInputParams {
    pub invest_duration_num: usize, // Длительность инвестиций, годы
    pub interest_num: f64,          // Процентная ставка инвестиций, проценты
    pub start_deposit_num: f64,     // Начальный капитал
    pub deposit_num: f64,           // Размер пополнения
    pub deposit_years_num: usize,   // Сколько лет делаем пополнения
    pub withdraw_num: f64,          // Размер ежемесячного изъятия
    pub withdraw_start_num: usize,  // Инвест год, начала изъятий
    pub inflation_num: f64,         // Годовой размер инфляции, проценты
    pub salary_grow_num: f64,       // Годовой рост пополнения, проценты
    pub investor_age_num: usize,    // Возраст инвестора на первый год инвестиций
    pub calendar_year_num: usize,   // Календарный год на первый год инвестиций
}

// Реализуем конструктор new()
impl CalcInputParams {
    /// Создает дефолтные параметры для инвестиционного расчета
    pub fn new() -> Self {
        Self {
            invest_duration_num: 0,  // Значения по умолчанию для калькулятора
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
        }
    }
}

// Дополнительно реализуем трейт Default
impl Default for CalcInputParams {
    fn default() -> Self {
        Self::new()
    }
}

// Вспомогательная функция
fn revert_step_to_years_fraction(step: usize) -> f64 {
    step as f64 / 12.0
}
//=======================================================================================
// Выполняет инвестиционный расчет по годам на основе входных параметров
//=======================================================================================
pub fn invest_calc(
    params: &CalcInputParams,
    calc_result_rows: &mut Vec<CalcHistoryRow>,
    reinvest_period: &ReinvestPeriod,
    deposit_frequency: &DepositFrequency) 
{
    // Очищаем результаты прошлых расчетов
    calc_result_rows.clear();

    // Извлекаем шаг реинвестирования (в месяцах) из энума
    let reinvest_step = match reinvest_period {
        ReinvestPeriod::Months3 => 3,
        ReinvestPeriod::Months6 => 6,
        ReinvestPeriod::Months12 => 12,
    };
   
    // Находим эффективную ставку на один шаг реинвестирования
    let annual_rate = params.interest_num / 100.0;
    
    // Вместо фиксированной месячной ставки считаем ставку на период (шаг) реинвестирования
    // Например, если шаг 3 месяца, то ставка за 3 месяца = (1 + годовая)^f64(3/12) - 1
    let step_rate = (1.0 + annual_rate).powf(revert_step_to_years_fraction(reinvest_step)) - 1.0;
    
    // А месячная ставка внутри этого шага рассчитывается как обычная линейная доля, 
    // так как внутри шага баланс не капитализируется
    let monthly_rate = step_rate / reinvest_step as f64;

    // Инициализируем переменные состояния для расчета
    let mut current_balance = params.start_deposit_num;
    let mut current_monthly_deposit = params.deposit_num;
    let mut mutable_withdraw_num = params.withdraw_num;
    
    // Переменная для накопления процентов между периодами реинвестирования
    let mut accumulated_interest = 0.0;

    // Внешний цикл: Идем по инвестиционным годам (от 1 до invest_duration_num)
    for year_idx in 1..=params.invest_duration_num {
        let current_investor_age = params.investor_age_num + (year_idx - 1);
        let current_calendar_year = params.calendar_year_num + (year_idx - 1);

        // Счетчики для накопления годовой статистики по строке
        let mut yearly_deposits = 0.0;
        let mut yearly_withdraws = 0.0;
        let mut yearly_interest_earned = 0.0;

        // Внутренний цикл: Идем по месяцам внутри текущего года (1..=12)
        for month in 1..=12 {
            
            // 1. ПОПОЛНЕНИЕ КАПИТАЛА
            if year_idx <= params.deposit_years_num {
                let is_deposit_month = match deposit_frequency {
                    DepositFrequency::Monthly => true,
                    DepositFrequency::EveryTwoMonths => month % 2 == 1,
                    DepositFrequency::Quarterly => month % 3 == 1,
                    DepositFrequency::SemiAnnually => month == 1 || month == 7,
                };

                if is_deposit_month {
                    current_balance += current_monthly_deposit;
                    yearly_deposits += current_monthly_deposit;
                }
            }

            // 2. ИЗЪЯТИЕ СРЕДСТВ
            if params.withdraw_start_num > 0 && mutable_withdraw_num > 0.0 && year_idx >= params.withdraw_start_num {
                let actual_withdraw = if current_balance >= mutable_withdraw_num {
                    mutable_withdraw_num
                } else {
                    current_balance
                };

                current_balance -= actual_withdraw;
                yearly_withdraws += actual_withdraw;
            }

            // 3. РАСЧЕТ И НАКОПЛЕНИЕ НАЧИСЛЕННЫХ ПРОЦЕНТОВ ЗА МЕСЯЦ
            let monthly_interest = current_balance * monthly_rate;
            accumulated_interest += monthly_interest;
            yearly_interest_earned += monthly_interest;

            // 4. КАПИТАЛИЗАЦИЯ (РЕИНВЕСТИРОВАНИЕ) ПРОЦЕНТОВ
            if month % reinvest_step == 0 {
                current_balance += accumulated_interest;
                accumulated_interest = 0.0;
            }
        }

        // Если в конце года остались нереинвестированные проценты
        if accumulated_interest > 0.0 {
            current_balance += accumulated_interest;
            accumulated_interest = 0.0;
        }

        // Сохраняем результаты за весь прошедший год в структуру строки таблицы
        calc_result_rows.push(CalcHistoryRow {
            invest_year: year_idx,
            investor_age: current_investor_age,
            calendar_year: current_calendar_year,
            deposits: yearly_deposits,
            withdraw: yearly_withdraws,
            interest: yearly_interest_earned,
            end_balance: current_balance,
        });

        // 5. ИНДЕКСАЦИЯ НА РОСТ ПОПОЛНЕНИЙ
        if params.salary_grow_num > 0.0 {
            let salary_grow_factor = 1.0 + (params.salary_grow_num / 100.0);
            current_monthly_deposit *= salary_grow_factor;
        }

        // 6. ИНДЕКСАЦИЯ НА ИНФЛЯЦИЮ
        if params.withdraw_start_num > 0 && mutable_withdraw_num > 0.0 && params.inflation_num > 0.0 && year_idx >= params.withdraw_start_num {
            let inflation_factor = 1.0 + (params.inflation_num / 100.0);
            mutable_withdraw_num *= inflation_factor;
        }
    }
}

//=======================================================================================
// Юнит-тесты
//=======================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    // ИСПРАВЛЕНО: Базовые параметры теперь имеют доходность 12% годовых
    fn create_default_params() -> CalcInputParams {
        CalcInputParams {
            invest_duration_num: 10,
            interest_num: 12.0, // Реальная процентная ставка вместо 0%
            start_deposit_num: 100_000.0,
            deposit_num: 0.0,
            deposit_years_num: 0,
            withdraw_num: 0.0,
            withdraw_start_num: 0,
            inflation_num: 0.0,
            salary_grow_num: 0.0,
            investor_age_num: 30,
            calendar_year_num: 2026,
        }
    }

    // НОВАЯ функция для тестов со стратегией накопления и пополнений
    fn create_accumulating_params() -> CalcInputParams {
        CalcInputParams {
            invest_duration_num: 5,
            interest_num: 10.0,          // 10% годовых
            start_deposit_num: 50_000.0,  // Стартуем с 50 000
            deposit_num: 5_000.0,         // Ежемесячно докидываем по 5 000
            deposit_years_num: 5,         // Пополняем на протяжении всех 5 лет
            withdraw_num: 0.0,
            withdraw_start_num: 0,
            inflation_num: 0.0,
            salary_grow_num: 0.0,
            investor_age_num: 25,
            calendar_year_num: 2026,
        }
    }

    #[test]
    fn test_duration_and_calendar_mapping() {
        let params = create_default_params();
        let mut results = Vec::new();

        invest_calc(&params, &mut results, &ReinvestPeriod::Months12, &DepositFrequency::Monthly);

        assert_eq!(results.len(), 10);
        assert_eq!(results[0].invest_year, 1);
        assert_eq!(results[0].investor_age, 30);
        assert_eq!(results[0].calendar_year, 2026);

        assert_eq!(results[9].invest_year, 10);
        assert_eq!(results[9].investor_age, 39);
        assert_eq!(results[9].calendar_year, 2035);
    }

    #[test]
    fn test_perfect_annual_compound_interest() {
        // Проверяем классический сложный процент без пополнений
        // 100 000 под 12% годовых при реинвестировании раз в 12 месяцев:
        // Через 1 год на счете должно быть ровно 112 000
        let mut params = create_default_params();
        params.invest_duration_num = 1;
        params.interest_num = 12.0;
        let mut results = Vec::new();

        invest_calc(&params, &mut results, &ReinvestPeriod::Months12, &DepositFrequency::Monthly);

        assert!((results[0].end_balance - 112_000.0).abs() < 1e-2);
        assert!((results[0].interest - 12_000.0).abs() < 1e-2);
    }

    #[test]
    fn test_compound_interest_multiple_years() {
        // Сложный процент за 3 года без пополнений под 10% годовых:
        // 100 000 * 1.1 * 1.1 * 1.1 = 133 100
        let mut params = create_default_params();
        params.invest_duration_num = 3;
        params.interest_num = 10.0;
        let mut results = Vec::new();

        invest_calc(&params, &mut results, &ReinvestPeriod::Months12, &DepositFrequency::Monthly);

        assert!((results[2].end_balance - 133_100.0).abs() < 1e-2);
    }

    #[test]
    fn test_accumulation_monthly_deposits_growth() {
        // ТЕСТ С ПОПОЛНЕНИЯМИ: Берем параметры накопления (10% годовых, 50к старт, 5к/мес пополнение)
        // За первый год чистая сумма пополнений должна составить 5000 * 12 = 60 000
        // Итоговый баланс будет значительно выше 110 000 (50к + 60к) за счет начисления % на пополнения
        let params = create_accumulating_params();
        let mut results = Vec::new();

        invest_calc(&params, &mut results, &ReinvestPeriod::Months12, &DepositFrequency::Monthly);

        assert_eq!(results[0].deposits, 60_000.0);
        // За счет процентов баланс в конце 1-го года должен быть больше 115 000
        assert!(results[0].end_balance > 115_000.0); 
    }

    #[test]
    fn test_accumulation_stopped_deposits() {
        // ТЕСТ С ПОПОЛНЕНИЯМИ: Пополняем только первый год, а на второй год пополнения прекращаются
        // Но проценты на накопленную сумму во второй год продолжают капать!
        let mut params = create_accumulating_params();
        params.invest_duration_num = 2;
        params.deposit_years_num = 1; // Ограничиваем пополнения только одним годом
        let mut results = Vec::new();

        invest_calc(&params, &mut results, &ReinvestPeriod::Months12, &DepositFrequency::Monthly);

        assert_eq!(results[0].deposits, 60_000.0);  // 1-й год: внесли 60к
        assert_eq!(results[1].deposits, 0.0);       // 2-й год: пополнения прекратились
        
        // Баланс второго года вырос исключительно за счет капитализации процентов
        assert!(results[1].end_balance > results[0].end_balance);
        assert!(results[1].interest > 0.0);
    }

    #[test]
    fn test_withdrawals_activation() {
        let mut params = create_default_params();
        params.invest_duration_num = 2;
        params.withdraw_num = 2000.0;
        params.withdraw_start_num = 2;
        let mut results = Vec::new();

        invest_calc(&params, &mut results, &ReinvestPeriod::Months12, &DepositFrequency::Monthly);

        assert_eq!(results[0].withdraw, 0.0); 
        assert_eq!(results[1].withdraw, 24_000.0); 
    }

    #[test]
    fn test_salary_growth_indexing() {
        let mut params = create_default_params();
        params.invest_duration_num = 2;
        params.deposit_num = 1000.0;
        params.deposit_years_num = 2;
        params.salary_grow_num = 10.0;
        params.interest_num = 0.0; // Для простоты проверки математики индексации обнулим ставку
        let mut results = Vec::new();

        invest_calc(&params, &mut results, &ReinvestPeriod::Months12, &DepositFrequency::Monthly);

        assert_eq!(results[0].deposits, 12_000.0);
        assert_eq!(results[1].deposits, 13_200.0);
    }

}


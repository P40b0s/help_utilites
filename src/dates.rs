use std::{borrow::Cow, fmt::Display, str::FromStr};

use chrono::{NaiveDateTime, Local, DateTime, NaiveDate, Datelike, NaiveTime, Timelike};
use logger::{error, backtrace};
use serde::{Deserialize, Serialize};
pub const FORMAT_SERIALIZE_DATE_TIME: &'static str = "%Y-%m-%dT%H:%M:%S";
///26-10-2022T13:23:52
pub const FORMAT_SERIALIZE_DATE_TIME_REVERSE: &'static str = "%d-%m-%YT%H:%M:%S";
pub const FORMAT_SERIALIZE_DATE_TIME_WS: &'static str = "%Y-%m-%d %H:%M:%S";
pub const FORMAT_DOT_DATE: &'static str = "%d.%m.%Y";
pub const FORMAT_DASH_DATE: &'static str = "%d-%m-%Y";
pub const FORMAT_FULL_DATE: &'static str = "%d %m %Y";


#[derive(Debug, Clone)]
/// Объект хранящий дату время, пока без оффсета
pub struct Date
{
    date : NaiveDateTime
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let date = String::deserialize(deserializer)?;
        let parsed = Date::parse(&date);
        if let Some(d) = parsed
        {
            Ok(d)
        }
        else
        {
            let err = format!("Ошибка входного формата данных - {}. Поддерживаются форматы: {}, {}, {}, {}", &date, FORMAT_DOT_DATE, FORMAT_SERIALIZE_DATE_TIME, FORMAT_SERIALIZE_DATE_TIME_REVERSE, FORMAT_SERIALIZE_DATE_TIME_WS);
            Err(serde::de::Error::custom(err))
        }
    }
}
impl<'a> Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.format(DateFormat::SerializeReverse))
    }
}

impl Date
{
    pub fn parse<'a, F: Into<Cow<'a, str>>>(date: F) -> Option<Self>
    {
        let date = date.into();
        if let Ok(dt) = NaiveDateTime::parse_from_str(&date, FORMAT_SERIALIZE_DATE_TIME)
        {
            Some(Date {date : dt})
        }
        else if let Ok(dt) = NaiveDateTime::parse_from_str(&date, FORMAT_SERIALIZE_DATE_TIME_REVERSE)
        {
            Some(Date{date: dt})
        }
        else if let Ok(dt) = NaiveDateTime::parse_from_str(&date, FORMAT_SERIALIZE_DATE_TIME_WS)
        {
            Some(Date{date: dt})
        }
        else if let Ok(dt) = NaiveDate::parse_from_str(&date, FORMAT_DOT_DATE)
        {
            let dt =  dt.and_hms_opt(0, 0, 0).unwrap();
            Some(Date{date: dt})
        }
        else if let Ok(dt) = NaiveDate::parse_from_str(&Self::locale_months_to_num(&date), FORMAT_FULL_DATE)
        {
            let dt = dt.and_hms_opt(0, 0, 0).unwrap();
            Some(Date{date: dt})
        }
        else 
        {
            error!("Ошибка входного формата данных - {}. Поддерживаются форматы: {}, {}, {}, {}", &date, FORMAT_DOT_DATE, FORMAT_SERIALIZE_DATE_TIME, FORMAT_SERIALIZE_DATE_TIME_REVERSE, FORMAT_SERIALIZE_DATE_TIME_WS);
            None
        }
    }
    pub fn new_date_time(day: u32, month: u32, year:u32, hour:u32, minute: u32, second: u32) -> Self
    {
        let time = NaiveTime::from_hms_opt(hour, minute, second).expect("Ошибка первода даты");
        let date = NaiveDate::from_ymd_opt(year as i32, month, day).expect("Ошибка первода даты");
        Self{date: NaiveDateTime::new(date, time)}
    }
    pub fn new_date(day: u32, month: u32, year:u32) -> Self
    {
        let time = NaiveTime::from_hms_opt(0, 0, 0).expect("Ошибка первода даты");
        let date = NaiveDate::from_ymd_opt(year as i32, month, day).expect("Ошибка первода даты");
        Self{date: NaiveDateTime::new(date, time)}
    }
    pub fn now() -> Self
    {
        let now = Self::from_date_time_to_naive_date_time(Local::now());
        Self{ date: now}
    }
    fn from_date_time_to_naive_date_time(value: DateTime<Local>) -> NaiveDateTime
    {
        let time = NaiveTime::from_hms_opt(value.hour(), value.minute(), value.second()).expect("Ошибка первода даты из формата DateTime<Local> в формат NaiveTime");
        let date = NaiveDate::from_ymd_opt(value.year(), value.month(), value.day()).expect("Ошибка первода даты из формата DateTime<Local> в формат NaiveDate");
        NaiveDateTime::new(date, time)
    }
    fn as_naive_datetime(&self) -> NaiveDateTime
    {
        self.date.clone()
    }
    fn locale_months_to_num(date: &str) -> String
    {
        date
        .replace("января", "1")
        .replace("февраля", "2")
        .replace("марта", "3")
        .replace("апреля", "4")
        .replace("мая", "5")
        .replace("июня", "6")
        .replace("июля", "7")
        .replace("августа", "8")
        .replace("сентября", "9")
        .replace("октября", "10")
        .replace("ноября", "11")
        .replace("декабря", "12")
    }

    fn num_to_locale_month(&self) -> String
    {
        match self.date.month()
        {
            1 => "января".to_owned(),
            2 => "февраля".to_owned(),
            3 => "марта".to_owned(),
            4 => "апреля".to_owned(),
            5 => "мая".to_owned(),
            6 => "июня".to_owned(),
            7 => "июля".to_owned(),
            8 => "августа".to_owned(),
            9 => "сентября".to_owned(),
            10 => "октября".to_owned(),
            11 => "ноября".to_owned(),
            12 => "декабря".to_owned(),
            m  => ["Месяца № ".to_owned(), m.to_string(), " не существует".to_owned()].concat()
        }
    }

    fn format(&self, format : DateFormat) -> String
    {
        match format
        {
            DateFormat::Serialize => self.date.format(FORMAT_SERIALIZE_DATE_TIME).to_string(),
            DateFormat::SerializeReverse => self.date.format(FORMAT_SERIALIZE_DATE_TIME_REVERSE).to_string(),
            DateFormat::OnlyDate => self.date.format(FORMAT_DASH_DATE).to_string(),
            DateFormat::DotDate => self.date.format(FORMAT_DOT_DATE).to_string(),
            DateFormat::FullDate => 
            {
                let day = self.date.day();
                let month = self.num_to_locale_month();
                let year = self.date.year();
                format!("{day:02} {month} {year}")
            }
        }  
    }
}

impl Display for Date
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        f.write_str(&self.format(DateFormat::SerializeReverse))
    }
}

pub enum DateFormat
{
    ///Формат сериализации данных %Y-%m-%dT%H:%M:%S
    Serialize,
     ///Формат сериализации данных %d-%m-%YT%H:%M:%S
    SerializeReverse,
    ///Формат даты %d-%m-%Y
    OnlyDate,
    ///Формат даты dd.MM.yyyy
    DotDate,
    ///Формат 25 октября 2015
    FullDate
}

// pub fn get_date(day: u32, month: u32, year: u32) -> String
// {
//     format!("{year}-{month:02}-{day:02}")
// }
// pub fn get_date_time(day: u32, month: u32, year: u32, hours: u32, minutes: u32, seconds: u32) -> String
// {
//     format!("{year}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}")
// }
// pub fn get_date_time_z(day: u32, month: u32, year: u32, hours: u32, minutes: u32, seconds: u32, z: u32) -> String
// {
//     format!("{year}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}+{z:02}:00")
// }

// fn from_date_time_to_naive_date_time(value: DateTime<Local>) -> NaiveDateTime
// {
//     let time = NaiveTime::from_hms_opt(value.hour(), value.minute(), value.second()).expect("Ошибка первода даты из формата DateTime<Local> в формат NaiveTime");
//     let date = NaiveDate::from_ymd_opt(value.year(), value.month(), value.day()).expect("Ошибка первода даты из формата DateTime<Local> в формат NaiveDate");
//     NaiveDateTime::new(date, time)
// }
// fn from_u32(day: u32, month: u32, year:u32, hour:u32, minute: u32, second: u32) -> Date
// {
//     let time = NaiveTime::from_hms_opt(hour, minute, second).expect("Ошибка первода даты");
//     let date = NaiveDate::from_ymd_opt(year as i32, month, day).expect("Ошибка первода даты");
//     NaiveDateTime::new(date, time)
// }


// fn to_serialized(date: NaiveDateTime) -> String
// {
//     date.format(FORMAT_SERIALIZE_DATE_TIME).to_string()
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaysProgress
{
    ///количество дней между начальной и конечной датой
    pub days: i64,
    ///количество оставшихся дней от сегодняшней даты
    pub days_left: i64,
    ///процент для прогрессбара 0-100% (количество оставшихся дней в процентах)
    pub progress: i64

}
impl Default for DaysProgress
{
    fn default() -> Self 
    {
        Self { days: 0, days_left: 0, progress: 100 }
    }
}
impl DaysProgress
{
    ///на вход принимаент начальную дату и конечную дату <br>
    /// 1-количество дней между начальной и конечной датой<br>
    /// 2-количество оставшихся дней от сегодняшней даты<br>
    /// 3-процент для прогрессбара 0-100% (количество оставшихся дней в процентах)
    pub fn days_diff(start_date: &str, end_date: &str) -> Option<Self>
    {
        let start_date = Date::parse(start_date)?;
        let end_date = Date::parse(end_date)?;
        let date_now = Date::now();
        let one_day = 86400; // секунд с сутках
        let end_start_timestramp_diff = end_date.as_naive_datetime().and_utc().timestamp() - start_date.as_naive_datetime().and_utc().timestamp();
        let diff_full_vacation = if end_start_timestramp_diff > 0
        {
            end_start_timestramp_diff / one_day
        }
        else
        {
            0
        };
        let end_now_timestramp_diff = end_date.as_naive_datetime().and_utc().timestamp() - date_now.as_naive_datetime().and_utc().timestamp();
        let diff_from_now = if end_now_timestramp_diff > 0
        {
            end_now_timestramp_diff / one_day
        }
        else
        {
            0
        };
        let process = (100.0f64 - ((diff_from_now as f64 / diff_full_vacation as f64) * 100.0f64)).floor() as i64;
        logger::info!("{} {}, {}%  {}",diff_full_vacation, diff_from_now, process, backtrace!());
        Some(Self { days: diff_full_vacation, days_left: diff_from_now, progress: process})
    }
}


//     const diffFromNow = ((end_date - date_now) / one_day) + 1;
   
//     return {
//         progress: Math.abs((100 - Math.round((diffFromNow / diffFullVacation) * 100))),
//         left: diffFromNow,
//         overall: diffFullVacation
//     };
// }

// type DateProgress = 
// {
//     /**Текущий процесс в процентах */
//     progress: number,
//     /**Количество в единицах сколько осталось */
//     left: number,
//     /**В единицах сколько между первой единицей и второй единицей */
//     overall: number
// }

fn floor()
{
    let r = 4/5;
    println!("{}",r);
}

#[cfg(test)]
mod test
{
    use logger::{StructLogger, debug};
    
    
    use super::
    {
        Date,
        DateFormat,
    };

    #[test]
    pub fn date_output() 
    {
        logger::StructLogger::initialize_logger();
        let date = Date::parse("26-10-2022T13:23:52").unwrap();
        debug!("Парсинг 26-10-2022T13:23:52 - {} ", date.format(DateFormat::DotDate));
        let date2 = Date::parse("26 октября 2020").unwrap();
        assert_eq!(date.format(DateFormat::FullDate), "26 октября 2022".to_owned());
        debug!("Парсинг 26 октября 2020 - {} ", date2.format(DateFormat::FullDate));
        assert_eq!(date.format(DateFormat::DotDate), "26.10.2022".to_owned());
        assert_eq!(date.format(DateFormat::Serialize), "2022-10-26T13:23:52".to_owned());
        assert_eq!(date.format(DateFormat::OnlyDate), "26-10-2022".to_owned());
        assert_eq!(date.format(DateFormat::SerializeReverse), "26-10-2022T13:23:52".to_owned());
        debug!("Вывод в формате DotDate: {}", date.format(DateFormat::DotDate));
        debug!("Вывод в формате Serialize: {}", date.format(DateFormat::Serialize));
        debug!("Вывод в формате OnlyDate: {}", date.format(DateFormat::OnlyDate));
        debug!("Вывод в формате SerializeReverse: {}", date.format(DateFormat::SerializeReverse));
        debug!("Вывод в формате FullDate: {}", date.format(DateFormat::FullDate));
        debug!("Тукущее время: {}", Date::now().to_string());
        debug!("Дата 12 12 2056: {}", Date::new_date(12, 12, 2056).to_string());
        
    }

    #[test]
    pub fn round() 
    {
        logger::StructLogger::initialize_logger();
        let start_date = "2024-04-24 23:59:00";
        let end_date = "2024-04-30 00:00:00";
        let dd = super::DaysProgress::days_diff(start_date, end_date).unwrap();
        debug!("{:?}", dd);
    }

}
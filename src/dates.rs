use std::{borrow::Cow, collections::BTreeSet, fmt::{Display, Write}, ops::{Add, Sub}, str::FromStr};

use chrono::{DateTime, Datelike, FixedOffset, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Timelike, Utc, Weekday};
use logger::{error, backtrace};
use serde::{Deserialize, Serialize};

use crate::error::Error;
pub const FORMAT_SERIALIZE_DATE_TIME: &'static str = "%Y-%m-%dT%H:%M:%S";
///26-10-2022T13:23:52
pub const FORMAT_SERIALIZE_DATE_TIME_REVERSE: &'static str = "%d-%m-%YT%H:%M:%S";
pub const FORMAT_SERIALIZE_MSSQL: &'static str = "%Y-%m-%d %H:%M:%S.%f";
pub const FORMAT_SERIALIZE_DATE_TIME_WS: &'static str = "%Y-%m-%d %H:%M:%S";
pub const FORMAT_DOT_DATE: &'static str = "%d.%m.%Y";
pub const FORMAT_DASH_DATE: &'static str = "%d-%m-%Y";
pub const FORMAT_FULL_DATE: &'static str = "%d %m %Y";
pub const FORMAT_JOIN_DATE: &'static str = "%Y%m%d";
///date today + parsed time
pub const FORMAT_TIME: &'static str = "%H:%M:%S";

#[derive(Debug, Clone)]
pub struct IncludeDates<'a>
{
    from: &'a Date,
    to: &'a Date,
    source_from: &'a Date,
    source_to: &'a Date
}

impl<'a> Display for IncludeDates<'a>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        let frmt = format!("Зафиксировано персечение временных отрезков {}@{} и {}@{}", self.source_from, self.source_to, self.from, self.to);
        f.write_str(&frmt)
    }
}
#[derive(Debug, Clone)]
/// Datetime object for parse and format date
pub struct Date(NaiveDateTime);
impl<'de> Deserialize<'de> for Date 
{
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

impl PartialEq for Date
{
    fn eq(&self, other: &Self) -> bool 
    {
        self.date_is_equalis(other)
    }
}
impl PartialOrd for Date
{
    fn gt(&self, other: &Self) -> bool 
    {
        self.as_naive_datetime() > other.as_naive_datetime()
    }
    fn ge(&self, other: &Self) -> bool 
    {
        self.date_is_equalis(other) || self.gt(other)
    }
    fn lt(&self, other: &Self) -> bool 
    {
        self.as_naive_datetime() < other.as_naive_datetime()
    }
    fn le(&self, other: &Self) -> bool 
    {
        self.date_is_equalis(other) || self.lt(other)
    }
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> 
    {
        Some(self.as_naive_datetime().cmp(&other.as_naive_datetime()))
    }
}
impl Eq for Date{}


impl Ord for Date
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering 
    {
        self.as_naive_datetime().cmp(&other.as_naive_datetime())
    }
}


impl<'a> Serialize for Date 
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.format(DateFormat::Serialize))
    }
}


impl From<NaiveDateTime> for Date
{
    fn from(value: NaiveDateTime) -> Self 
    {
        Self(value)
    }
}

impl Date
{
    /// Поддерживаемые форматы дат:  
    /// 26-10-2022T13:23:52  
    /// 2022-10-26T13:23:52  
    /// 2022-10-26 13:23:52  
    /// 26.10.2022  
    /// 26-10-2022  
    /// 26 ноября 2022
    /// 20240618
    pub fn parse<'a, F: Into<Cow<'a, str>>>(date: F) -> Option<Self>
    {
        let date = date.into();
        if let Ok(dt) = NaiveDateTime::parse_from_str(&date, FORMAT_SERIALIZE_DATE_TIME)
        {
            Some(Date(dt))
        }
        else if let Ok(dt) = NaiveDateTime::parse_from_str(&date, FORMAT_SERIALIZE_DATE_TIME_REVERSE)
        {
            Some(Date(dt))
        }
        else if let Ok(dt) = NaiveDateTime::parse_from_str(&date, FORMAT_SERIALIZE_DATE_TIME_WS)
        {
            Some(Date(dt))
        }
        else if let Ok(dt) = NaiveDateTime::parse_from_str(&date, FORMAT_SERIALIZE_MSSQL)
        {
            Some(Date(dt))
        }
        else if let Ok(dt) = NaiveDate::parse_from_str(&date, FORMAT_DOT_DATE)
        {
            let dt =  dt.and_hms_opt(0, 0, 0).unwrap();
            Some(Date(dt))
        }
        else if let Ok(dt) = NaiveDate::parse_from_str(&date, FORMAT_DASH_DATE)
        {
            let dt =  dt.and_hms_opt(0, 0, 0).unwrap();
            Some(Date(dt))
        }
        else if let Ok(dt) = NaiveDate::parse_from_str(&Self::locale_months_to_num(&date), FORMAT_FULL_DATE)
        {
            let dt = dt.and_hms_opt(0, 0, 0).unwrap();
            Some(Date(dt))
        }
        else if let Ok(dt) = NaiveDate::parse_from_str(&date, FORMAT_JOIN_DATE)
        {
            let dt =  dt.and_hms_opt(0, 0, 0).unwrap();
            Some(Date(dt))
        }
        else if let Ok(time) = NaiveTime::parse_from_str(&date, FORMAT_TIME)
        {
            let value = Local::now();
            let date = NaiveDate::from_ymd_opt(value.year(), value.month(), value.day()).expect("Ошибка первода даты из формата DateTime<Local> в формат NaiveDate");
            Some(Self(NaiveDateTime::new(date, time)))
        }
        else 
        {
            error!("Ошибка входного формата данных - {}. Поддерживаются форматы: {}, {}, {}, {}, {}, {}", &date, FORMAT_JOIN_DATE, FORMAT_DOT_DATE, FORMAT_SERIALIZE_DATE_TIME, FORMAT_SERIALIZE_DATE_TIME_REVERSE, FORMAT_SERIALIZE_DATE_TIME_WS, FORMAT_TIME);
            None
        }
    }
   
    pub fn new_time(hour:u32, minute: u32, second: u32) -> Self
    {
        let value = Local::now();
        let time = NaiveTime::from_hms_opt(hour, minute, second).expect("Ошибка первода даты");
        let date = NaiveDate::from_ymd_opt(value.year(), value.month(), value.day()).expect("Ошибка первода даты из формата DateTime<Local> в формат NaiveDate");
        Self(NaiveDateTime::new(date, time))
    }
    pub fn new_date(day: u32, month: u32, year:u32) -> Self
    {
        let time = NaiveTime::from_hms_opt(0, 0, 0).expect("Ошибка первода даты");
        let date = NaiveDate::from_ymd_opt(year as i32, month, day).expect("Ошибка первода даты");
        Self(NaiveDateTime::new(date, time))
    }
    pub fn now() -> Self
    {
        let now = Self::from_date_time_to_naive_date_time(Local::now());
        Self(now)
    }
    fn from_date_time_to_naive_date_time(value: DateTime<Local>) -> NaiveDateTime
    {
        let time = NaiveTime::from_hms_opt(value.hour(), value.minute(), value.second()).expect("Ошибка первода даты из формата DateTime<Local> в формат NaiveTime");
        let date = NaiveDate::from_ymd_opt(value.year(), value.month(), value.day()).expect("Ошибка первода даты из формата DateTime<Local> в формат NaiveDate");
        NaiveDateTime::new(date, time)
    }
    pub fn as_naive_datetime(&self) -> NaiveDateTime
    {
        self.0.clone()
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
        match self.0.month()
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

    pub fn format(&self, format : DateFormat) -> String
    {
        match format
        {
            DateFormat::Serialize => self.0.format(FORMAT_SERIALIZE_DATE_TIME).to_string(),
            DateFormat::SerializeReverse => self.0.format(FORMAT_SERIALIZE_DATE_TIME_REVERSE).to_string(),
            DateFormat::OnlyDate => self.0.format(FORMAT_DASH_DATE).to_string(),
            DateFormat::DotDate => self.0.format(FORMAT_DOT_DATE).to_string(),
            DateFormat::JoinDate => self.0.format(FORMAT_JOIN_DATE).to_string(),
            DateFormat::Time => self.0.format(FORMAT_TIME).to_string(),
            DateFormat::MsqSqlDate => 
            {
                let nanos =  self.0.and_utc().timestamp_subsec_nanos() % 1000;
                let formatted = format!(
                    "{}.{}",
                    self.0.format("%Y-%m-%d %H:%M:%S"),
                    nanos
                );
                formatted
            },
            DateFormat::FullDate => 
            {
                let day = self.0.day();
                let month = self.num_to_locale_month();
                let year = self.0.year();
                format!("{day:02} {month} {year}")
            }
        }  
    }
    pub fn add_minutes(self, minutes: i64) -> Self
    {
        let s = Self(self.0.checked_add_signed(TimeDelta::minutes(minutes)).unwrap());
        s
    }
    pub fn sub_minutes(self, minutes: i64) -> Self
    {
        let s = Self(self.0.checked_add_signed(TimeDelta::minutes(-minutes)).unwrap());
        s
    }
    pub fn add_months(self, months: u32) -> Option<Self>
    {
        let s = Self(self.0.checked_add_months(Months::new(months))?);
        Some(s)
    }
    pub fn add_seconds(self, seconds: i64) -> Self
    {
        let s = Self(self.0.checked_add_signed(TimeDelta::seconds(seconds)).unwrap());
        s
    }

    

    ///Если временные отрезки пересекаются, то вернется объект IncludeDates с первым попавшимся пересечением
    pub fn in_range<'a>(source: (&'a Date, &'a Date), range: &[(&'a Date, &'a Date)]) -> Option<IncludeDates<'a>>
    {
        for r in range
        {
            if !((source.0.0 < r.0.0 && source.1.0 < r.0.0)
            || (source.0.0 > r.1.0 && source.1.0 > r.1.0))
            {
                return Some(IncludeDates
                {
                    from: r.0,
                    to:  r.1,
                    source_from: source.0,
                    source_to: source.1
                })
            }
        }
        None
    }
    ///Если временные отрезки пересекаются, то вернется true
    /// `time_from` - в формате h m s
    pub fn time_in_range<'a>(source: &'a Date, time_from: (u32, u32, u32), time_to: (u32, u32, u32)) -> bool
    {
        let source_time = source.0.time();
        if time_from.0 > time_to.0
        {
            let range_from_time_1 = Self::new_time(time_from.0, time_from.1, time_from.2).0.time();
            let range_to_time_1 = Self::new_time(23, 59, 59).0.time();
            let range_from_time_2 = Self::new_time(0, 0, 0).0.time();
            let range_to_time_2 = Self::new_time(time_to.0, time_to.1, time_to.2).0.time();
            if (source_time > range_from_time_1) && (source_time < range_to_time_1) 
                || (source_time > range_from_time_2) && (source_time < range_to_time_2)
            {
                true
            }
            else 
            {
                false
            }
        }
        else 
        {
            let range_from_time = Self::new_time(time_from.0, time_from.1, time_from.2).0.time();
            let range_to_time = Self::new_time(time_to.0, time_to.1, time_to.2).0.time();
            if (source_time > range_from_time) && (source_time < range_to_time)
            {
                true
            }
            else 
            {
                false
            }
        }
    }
    pub fn time_in_hour<'a>(&self, in_hour: u32) -> bool
    {
        let source_time = self.0.time();
        source_time.hour() == in_hour
    }
    pub fn with_time(self, date_with_time: &Date) -> Self
    {
        let old_time = date_with_time.0.time();
        let new_date = Date::new_date(self.0.day(), self.0.month(), self.0.year() as u32)
        .add_minutes(((old_time.hour() * 60) + old_time.minute()) as i64);
        new_date
    }
    pub fn is_today(&self) -> bool
    {
        let today = Self::now();
        if today.0.date() == self.0.date()
        {
            return true
        }
        false
    }
    pub fn is_weekend(&self) -> bool
    {
        let today = Self::now();
        if today.0.weekday() == Weekday::Sat || today.0.weekday() == Weekday::Sun
        {
            return true
        }
        false
    }
    pub fn date_is_equalis(&self, other: &Date) -> bool
    {
        if other.0.date() == self.0.date()
        {
            return true
        }
        false
    }
    pub fn time_is_equalis(&self, other: &Date) -> bool
    {
        if other.0.time() == self.0.time()
        {
            return true
        }
        false
    }
    ///del all exists dates in `items` if exists in `to_remove`
    pub fn exclude(items: &mut Vec<Date>, to_remove: Vec<Date>, compare: DateFormat)
    {
        let to_remove: Vec<String> = BTreeSet::from_iter(to_remove).into_iter().map(|m| m.format(compare.clone())).collect();
        items.retain(|e| !to_remove.contains(&e.format(compare.clone())));
    }
    ///del all exists dates in source and add all added_dates
    pub fn union(items: &mut Vec<Date>, added_dates: Vec<Date>, compare: DateFormat)
    {
        
        let formated_dates: Vec<String> = BTreeSet::from_iter(&added_dates).into_iter().map(|m| m.format(compare.clone())).collect();
        items.retain(|e| !formated_dates.contains(&e.format(compare.clone())));
        items.extend(added_dates);
    }
    ///добавляет время до конца текущей даты например 2022-10-26T13:23:52 -> 2022-10-26T23:59:59 
    fn add_time_to_end_day(self) -> Self
    {
        if self.0.hour() == 0 && self.0.minute() == 0 && self.0.second() == 0
        {
            Self(self.0.with_hour(23).unwrap().with_minute(59).unwrap().with_second(59).unwrap())
        }
        else
        {
            self
        }
    }
    /// Высчитывает разницу между датами в днях
    pub fn diff(&self, end_date: Date) -> Diff
    {
        let start_date = self;
        let end_date = end_date.add_time_to_end_day();
        let date_now = Date::now();
        let one_day = 86400; // секунд с сутках
        //разница между двумя датами в днях
        let end_start_timestramp_diff = end_date.as_naive_datetime().and_utc().timestamp() - start_date.as_naive_datetime().and_utc().timestamp();
        let diff_full_vacation = if end_start_timestramp_diff > 0
        {
            end_start_timestramp_diff as f64 / one_day as f64
        }
        else
        {
            0.0
        };
        //разница между сегодняшней датой и конечной датой
        let end_now_timestramp_diff = end_date.as_naive_datetime().and_utc().timestamp() - date_now.as_naive_datetime().and_utc().timestamp();
        let diff_from_now = if end_now_timestramp_diff > 0
        {
            end_now_timestramp_diff as f64 / one_day as f64
        }
        else
        {
            0.0
        };
        let process = (100.0f64 - ((diff_from_now as f64 / diff_full_vacation as f64) * 100.0f64)).floor() as i64;
        logger::info!("{} {}, {}%  {}",Self::round(diff_full_vacation, 2), Self::round(diff_from_now, 2), process, backtrace!());
        Diff { days: Self::round(diff_full_vacation, 2), days_left: Self::round(diff_from_now,2), progress: process}
    }
    
    fn round(x: f64, decimals: u32) -> f64 
    {
        let y = 10i32.pow(decimals) as f64;
        (x * y).round() / y
    }
    // pub fn convert_system_time(dt: SystemTime) -> Option<String>
    // {
    //     let mut offset: OffsetDateTime = dt.into();
    //     let dur = Duration::hours(3);
    //     if let Ok(utc_offset_result) = UtcOffset::from_whole_seconds(dur.as_seconds_f32().round() as i32)
    //     {
    //         offset = offset.to_offset(utc_offset_result);
    //     }
    //     let dt_format = crate::SETTINGS.read().unwrap().get_date_time_format();
    //     let format = format_description::parse(
    //         &dt_format,
    //     ).ok()?;
    //     // let format = format_description::parse(
    //     //     "[year]-[month]-[day]T[hour]:[minute]:[second]",
    //     // ).ok()?;
    //     let off = offset.format(&format);
    //     match off
    //     {
    //         Ok(off) => {
    //             //println!("{}", off);
    //             return Some(off);
    //         },
    //         Err(e) => 
    //         {
    //             error!("Ошибка преобразования даты: {:?}, {}", dt, e.to_string());
    //             return None;
    //         }
    //     };
    // }

    pub fn from_system_time(dt: std::time::SystemTime) -> Self
    {
     
        let dt_now_local: chrono::DateTime<Local> = dt.into();
        let n: NaiveDateTime = dt_now_local.naive_local();
        Self(n)
    
    }
}

impl FromStr for Date
{
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        if let Some(d) = Date::parse(s)
        {
            Ok(d)
        }
        else 
        {
            Err(Error::DateParseError(s.to_string(), ["Поддерживаются форматы:", FORMAT_JOIN_DATE, FORMAT_DOT_DATE, FORMAT_SERIALIZE_DATE_TIME, FORMAT_SERIALIZE_DATE_TIME_REVERSE, FORMAT_SERIALIZE_DATE_TIME_WS, FORMAT_TIME, FORMAT_SERIALIZE_MSSQL].concat()))
        }
    }
}

impl Sub for &Date
{
    type Output = i64;
    fn sub(self, rhs: Self) -> Self::Output 
    {
        self.as_naive_datetime().and_utc().timestamp() - rhs.as_naive_datetime().and_utc().timestamp()
    }
}
impl Add for &Date
{
    type Output = i64;
    fn add(self, rhs: Self) -> Self::Output 
    {
        self.as_naive_datetime().and_utc().timestamp() + rhs.as_naive_datetime().and_utc().timestamp()
    }
}

impl Display for Date
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        f.write_str(&self.format(DateFormat::Serialize))
    }
}
#[derive(Clone)]
pub enum DateFormat
{
    /// 2022-10-26T13:23:52  
    Serialize,
    /// 26-10-2022T13:23:52  
    SerializeReverse,
    /// 26-10-2022  
    OnlyDate,
    /// 26.10.2022  
    DotDate,
    /// 25 октября 20122
    FullDate,
    /// 20240618
    JoinDate,
    ///2025-03-11 15:51:21.452
    MsqSqlDate,
    ///12:00:00
    Time
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diff
{
    ///количество дней между начальной и конечной датой
    pub days: f64,
    ///количество оставшихся дней от сегодняшней даты
    pub days_left: f64,
    ///процент для прогрессбара 0-100% (количество оставшихся дней в процентах)
    pub progress: i64
}
impl Default for Diff
{
    fn default() -> Self 
    {
        Self { days: 0.0, days_left: 0.0, progress: 100 }
    }
}

// fn floor()
// {
//     let r = 4/5;
//     println!("{}",r);
// }

#[cfg(test)]
mod test
{
    use logger::debug;
    use serde::{Deserialize, Serialize};
    
    
    use crate::exclude;

    use super::
    {
        Date,
        DateFormat,
    };

    #[test]
    pub fn date_output() 
    {
        let _ = logger::StructLogger::new_default();
        let date = Date::parse("26-10-2022T13:23:52").unwrap();
        debug!("Парсинг 26-10-2022T13:23:52 - {} ", date.format(DateFormat::DotDate));
        let date2 = Date::parse("26 октября 2020").unwrap();
        assert_eq!(date.format(DateFormat::FullDate), "26 октября 2022".to_owned());
        debug!("Парсинг 26 октября 2020 - {} ", date2.format(DateFormat::FullDate));
        assert_eq!(date.format(DateFormat::DotDate), "26.10.2022".to_owned());
        assert_eq!(date.format(DateFormat::Serialize), "2022-10-26T13:23:52".to_owned());
        assert_eq!(date.format(DateFormat::OnlyDate), "26-10-2022".to_owned());
        assert_eq!(date.format(DateFormat::SerializeReverse), "26-10-2022T13:23:52".to_owned());
        let date_ms = Date::parse("2022-10-26 13:23:52.412").unwrap();
        debug!("Парсинг в формате mssql: 2022-10-26 13:23:52.412 - {} ", date_ms.format(DateFormat::MsqSqlDate));
        assert_eq!(date_ms.format(DateFormat::MsqSqlDate), "2022-10-26 13:23:52.412".to_owned());
        debug!("Вывод в формате DotDate: {}", date.format(DateFormat::DotDate));
        debug!("Вывод в формате Serialize: {}", date.format(DateFormat::Serialize));
        debug!("Вывод в формате OnlyDate: {}", date.format(DateFormat::OnlyDate));
        debug!("Вывод в формате SerializeReverse: {}", date.format(DateFormat::SerializeReverse));
        debug!("Вывод в формате FullDate: {}", date.format(DateFormat::FullDate));
        debug!("Тукущее время: {}", Date::now().to_string());
        debug!("Дата 12 12 2056: {}", Date::new_date(12, 12, 2056).to_string());
        
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DP
    {
        ///количество дней между начальной и конечной датой
        pub days: f64,
        ///процент для прогрессбара 0-100% (количество оставшихся дней в процентах)
        pub date: Date
    }


    #[test]
    pub fn serialize_date() 
    {
        let _ = logger::StructLogger::new_default();
        let start_date = Date::parse("2024-04-24 08:50:00").unwrap();
        let d = DP {days: 6.6654, date: start_date};
        let s = serde_json::to_string(&d).unwrap();
        //start_date.serialize(s);
        debug!("{:?}", &s);
        let structure: DP = serde_json::from_str(&s).unwrap();
        debug!("{:?}", &structure);
    }

    #[test]
    pub fn round() 
    {
        let _ = logger::StructLogger::new_default();
        let start_date = Date::parse("2024-04-24 08:50:00").unwrap();
        let end_date = Date::parse("2024-04-30 08:59:00").unwrap();
        let dd = start_date.diff(end_date);
        debug!("{:?}", dd);
    }
    #[test]
    pub fn test_in_range() 
    {
        let _ = logger::StructLogger::new_default();
        let start_date = Date::parse("2024-04-30 11:50:00").unwrap();
        let end_date = Date::parse("2024-04-30 11:59:00").unwrap();

        let a1 = Date::parse("2024-04-30 07:50:00").unwrap();
        let a2 = Date::parse("2024-04-30 08:49:00").unwrap();
        let b1 = Date::parse("2024-04-30 09:51:00").unwrap();
        let b2 = Date::parse("2024-04-30 10:50:00").unwrap();
        let c1 = Date::parse("2024-04-30 11:50:00").unwrap();
        let c2 = Date::parse("2024-04-30 12:50:00").unwrap();
        let arr : Vec<(&Date, &Date)> = vec![
            (&a1, &a2),
            (&b1, &b2),
            (&c1, &c2),
        ];
        let res = super::Date::in_range((&start_date, &end_date), &arr);
        debug!("{}", res.unwrap());
    }

    #[test]
    pub fn test_time_in_range() 
    {
        let _ = logger::StructLogger::new_default();
        let start_date = Date::parse("2024-04-30 11:50:00").unwrap();
        let range = Date::time_in_range(&start_date, (11, 25, 0), (12,20,0));
        assert_eq!(range, true);
        let start_date = Date::parse("2024-04-30 00:50:00").unwrap();
        let range = Date::time_in_range(&start_date, (23, 0, 0), (6,0,0));
        assert_eq!(range, true);
    }
    #[test]
    pub fn test_tz() 
    {
        let _ = logger::StructLogger::new_default();
        let date = Date::parse("2024-04-30 11:50:00").unwrap();
        let date = date.add_minutes(3 * 60);
        debug!("{:?}", &date);
    }
    #[test]
    pub fn test_exclude() 
    {
        let _ = logger::StructLogger::new_default();
        let exclude_date = vec![Date::parse("2024-04-30 11:22:00").unwrap()];
        let mut dates = vec![Date::parse("2024-04-30 11:50:00").unwrap(), Date::parse("2024-05-30 11:59:00").unwrap()];
        Date::exclude(&mut dates, exclude_date, DateFormat::DotDate);
        debug!("{:?}", &dates);
    }

    #[test]
    pub fn test_with_time() 
    {
        let _ = logger::StructLogger::new_default();
        let date = Date::parse("2024-04-30 11:22:00").unwrap();
        let now = Date::now();
        let n = now.with_time(&date).add_months(1);
        
        debug!("{:?}", n);
    }

}
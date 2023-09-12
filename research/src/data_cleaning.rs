use crate::backtesting;

use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::error::Error;
use chrono::{DateTime, Utc, TimeZone};
use chrono::{Datelike, Timelike, Weekday};


// The purpose of this module is to clean up the data in the data/bin directory.
// The raw data contains some continuity errors, which must be fixed.
// The ultimate goal is to chunk the data into continuous segments of 1 week each.
// If a segment is not continuous, it should not be included in the final data set.

// The data is stored in the following format:
// 1. The first 8 bytes are the timestamp, in milliseconds since the epoch.
// 2. The next 4 bytes are the bid price, as a f32 (little endian).
// 3. The last 4 bytes are the ask price, as a f32 (little endian).

// Basic process for cleaning the data:
// 1. Iterate through the data until we find the first weekend.
// 2. Iterate through the data until we find the next weekend.
// 3. If the data between the two weekends is continuous, write it to a new file.
//    If we find a date later than the second weekend, we discard that week of data and return to step 1.
// 4. Repeat until we reach the end of the data.

// Important values:
// Markets close at 5pm EST on Friday, and reopen at 5pm EST on Sunday.
// This is 10pm UTC on Friday, and 10pm UTC on Sunday.

pub struct Week {
    pub data: Vec<[u8; 16]>,
    pub start: u64,
    pub end: u64,
}

pub fn prettify_dt(dt: chrono::DateTime<chrono::Utc>) -> String {
    return format!("{}, {} {} {} {:02}:{:02}:{:02}", dt.weekday(), dt.day(), dt.month(), dt.year(), dt.hour(), dt.minute(), dt.second());
}

pub fn millisecond_timestamp_to_datetime(ts_millis: i64) -> chrono::DateTime<Utc> {
    let nanos = ts_millis * 1_000_000;
    let dt = Utc.timestamp_nanos(nanos);
    dt
}

pub fn datetime_to_millisecond_timestamp(dt: DateTime<Utc>) -> i64 {
    let nanos = dt.timestamp_nanos();
    let millis = nanos / 1_000_000;
    millis
}

// Finds the next Friday at 10pm UTC.
pub fn find_next_weekend_start(date: DateTime<Utc>) -> DateTime<Utc> {
    let mut date = date;
    loop {
        if date.weekday() == Weekday::Fri && date.time() <= chrono::NaiveTime::from_hms_opt(22, 0, 0).unwrap() {
            // Return the date at precisely 10pm UTC.
            return date.with_hour(22).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();
        }

        date = date + chrono::Duration::hours(12);
    }
}

// Finds the next Sunday at 10pm UTC.
pub fn find_next_weekend_end(date: DateTime<Utc>) -> DateTime<Utc> {
    let mut date = date;
    loop {
        if date.weekday() == Weekday::Sun && date.time() <= chrono::NaiveTime::from_hms_opt(22, 0, 0).unwrap() {
            // Return the date at precisely 10pm UTC.
            return date.with_hour(22).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();
        }

        date = date + chrono::Duration::hours(12);
    }
}

pub fn seek_first_time_gt(file: &mut BufReader<File>, start: u64) -> Result<u64, Box<dyn Error>> {
    let mut buffer: [u8; 16] = [0; 16];
    loop {
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                let timestamp_bytes: [u8; 8] = buffer[0..8].try_into()?;
                let timestamp = u64::from_le_bytes(timestamp_bytes);
                if timestamp < start {
                    continue;
                }

                return Ok(timestamp);
            },
            Err(_) => return Err("Error reading file".into()),
        }
    }
}


// Takes in a path to a binary file containing data in the format described above, and a timestamp in milliseconds.
// Returns a tuple containing a vector of 16-byte chunks of data, and the timestamp of the last chunk of data.
// The vector of data will contain the next complete week of data, starting at the timestamp provided.
pub fn find_next_week(path: &str, start: u64) -> Result<(Vec<[u8; 16]>, u64), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut file = BufReader::new(file);
    let mut buffer: [u8; 16] = [0; 16];
    let mut week_buffer: Vec<[u8; 16]> = Vec::new();

    // Find the first timestamp that is greater than start.
    // For example, if a user says find next week starting at 0, we don't want to start at 0, we want to start at the first timestamp in the data.
    let true_start: u64 = seek_first_time_gt(&mut file, start)?;

    // Our week chunk starts when the market opens on Sunday.
    let week_start: i64 = datetime_to_millisecond_timestamp(
        find_next_weekend_end(
            millisecond_timestamp_to_datetime(true_start as i64)
        )
    );

    // Our week chunk ends when the market closes on Friday.
    let week_end: i64 = datetime_to_millisecond_timestamp(
        find_next_weekend_start(
            millisecond_timestamp_to_datetime(week_start as i64)
        )
    );

    // println!(
    //     "Finding data... week start: {}, week end: {}", 
    //     prettify_dt(millisecond_timestamp_to_datetime(week_start)),
    //     prettify_dt(millisecond_timestamp_to_datetime(week_end))
    // );

    // Iterate over our data, ignore any data from before week_start, record data to week_buffer, and break on the first timestamp after week_end.
    loop {
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                let timestamp_bytes: [u8; 8] = buffer[0..8].try_into()?;
                let timestamp = u64::from_le_bytes(timestamp_bytes);
                if timestamp < week_start as u64 {
                    continue;
                }

                if timestamp > week_end as u64 {
                    break;
                }

                week_buffer.push(buffer);
            },
            Err(_) => return Err("Error reading file".into()),
        }
    }
    
    Ok((week_buffer, week_end as u64))
}

pub fn chunk_data_into_weeks(path: &str) -> Result<Vec<Week>, Box<dyn Error>> {
    let mut weeks: Vec<Week> = Vec::new();
    let mut next_start = 0;
    let mut i = 0;
    loop {
        // println!("===== Week {} ({}) =====", i, currency_pair);
        let (week, last_timestamp) = {
            match find_next_week(&path, next_start) {
                Ok((week, last_timestamp)) => (week, last_timestamp),
                
                // Errors should not be used for control flow, but I'm choosing to do so here out of laziness.
                // This process is only run once, so it's not a big deal.
                // This error is expected to occur when we reach the end of the file.
                Err(e) => {
                    // println!("Error: {}", e);
                    break;
                },
            }
        };

        // Set next_start to the timestamp just after markets close on Friday.
        // The find_next_week function will then find the next week of data, starting on market open on Sunday.
        next_start = last_timestamp + 1;

        if week.len() == 0 {
            continue;
        }

        let (start_timestamp, _, _) = backtesting::parse_chunk(&week[0], true)?;
        let (end_timestamp, _, _) = backtesting::parse_chunk(&week[week.len() - 1], true)?;

        weeks.push(Week {
            data: week,
            start: start_timestamp,
            end: end_timestamp,
        });
        i += 1;
    }
    Ok(weeks)
}

fn ensure_dir_exists(path: &std::path::Path) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn write_week_to_file(week: &Week, path: &str) -> Result<(), Box<dyn Error>> {
    let path = std::path::Path::new(path);
    ensure_dir_exists(path.parent().unwrap())?;

    let mut file = File::create(path).unwrap();
    for chunk in &week.data {
        file.write_all(chunk)?;
    }
    Ok(())
}

pub fn write_weeks_to_file(weeks: &Vec<Week>, path: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;
    for week in weeks {
        for chunk in &week.data {
            file.write_all(chunk)?;
        }
    }
    Ok(())
}

pub fn calculate_average_time_between_ticks(week: &Week) -> Result<f64, Box<dyn Error>> {
    let mut total_time_between_ticks: f64 = 0.0;
    let mut total_ticks: f64 = 0.0;
    
    let mut last_timestamp: u64 = 0;

    for chunk in &week.data {
        let (timestamp, _, _) = backtesting::parse_chunk(chunk, true)?;
        if last_timestamp == 0 {
            last_timestamp = timestamp;
            continue;
        }

        let time_between_ticks = (timestamp - last_timestamp) as f64;
        total_time_between_ticks += time_between_ticks;
        total_ticks += 1.0;
        last_timestamp = timestamp;
    }

    let average_time_between_ticks = total_time_between_ticks / total_ticks;
    Ok(average_time_between_ticks)
}

pub fn calculate_max_time_between_ticks(week: &Week) -> Result<f64, Box<dyn Error>> {
    let mut max_time_between_ticks: f64 = 0.0;
    
    let mut last_timestamp: u64 = 0;

    for chunk in &week.data {
        let (timestamp, _, _) = backtesting::parse_chunk(chunk, true)?;
        if last_timestamp == 0 {
            last_timestamp = timestamp;
            continue;
        }

        let time_between_ticks = (timestamp - last_timestamp) as f64;
        if time_between_ticks > max_time_between_ticks {
            max_time_between_ticks = time_between_ticks;
        }
        last_timestamp = timestamp;
    }

    Ok(max_time_between_ticks)
}

fn generate_path(currency_pair: &str) -> String {
    format!("data/bin/{}-all.bin", currency_pair)
}

pub fn clean_data(currency_pairs: Vec<&str>) -> Result<(), Box<dyn Error>> {
    for currency_pair in currency_pairs {
        let path = generate_path(currency_pair);
        match chunk_data_into_weeks(&path) {
            Ok(weeks) => {
                println!("{} weeks found for {}.", weeks.len(), currency_pair);

                for (i, week) in weeks.iter().enumerate() {
                    let avg_time = calculate_average_time_between_ticks(week)?;
                    let max_time = calculate_max_time_between_ticks(week)?;

                    // If the max time between ticks is greater than 10 minutes, print it out.
                    if max_time > 600_000.0 {
                        println!("[Week {}] Max gap: {}s > 10m, discarding this week.", i, max_time / 1000.0);
                    }
                    else {
                        let path = format!("data/weekly/{}/week-{}.bin", currency_pair, i);
                        println!("[Week {}] Average time between ticks: {}s, max gap: {}s, saving to file {}", i, avg_time / 1000.0, max_time / 1000.0, path);
                        write_week_to_file(week, &path)?;
                    }
                }
            },
            Err(e) => println!("Error: {}", e),
        };
    }
    Ok(())
}
use std::collections::HashMap;
use std::time::Duration;

use clap::{Parser, Subcommand};
use chrono::prelude::*;
use chrono::Duration as OtherDuration;
use indicatif::ProgressBar;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search by city (camelCase)
    city { 
        name: String,
        ///The unit to check (f|c)
        #[arg(short, long,)]
        unit: Option<String>,
        ///The date to check (YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>
     },
    ///Search by latitude and longitude
    pos { 
        lat: f32, 
        long: f32,
        ///The unit to check (f|c)
        #[arg(short, long)]
        unit: Option<String>,
        ///The date to check (YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>
     }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));

    let cli = Cli::parse();

    let mut latitude: f32;
    let mut longitude: f32;
    let mut unit_t = "celsius";
    
    let now: DateTime<Local> = Local::now();
    let mut date_t = format!("{}", (now + OtherDuration::days(1)).date().format("%Y-%m-%d"));

    match &cli.command {
        Commands::city { name, unit, date } => {
            bar.set_message("Getting latitude and longitude");
            let resp = reqwest::get(format!("https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json", name))
            .await?
            .text()
            .await?;

            latitude = resp.replace(",", "").replace(":", "").split("\"").collect::<Vec<&str>>()[10].parse::<f32>().expect("Location Not Found");
            longitude = resp.replace(",", "").replace(":", "").split("\"").collect::<Vec<&str>>()[12].parse::<f32>().expect("Location Not Found");

            if unit.is_some() {
                unit_t = match unit.clone().unwrap().as_str() {
                    "f" | "fahrenheit" => "fahrenheit",
                    _ => "celsius"
                };
            }

            if date.is_some() {
                date_t = date.clone().unwrap();
            }
        },
        Commands::pos { lat, long , unit, date} => {
            latitude = *lat;
            longitude = *long;

            if unit.is_some() {
                unit_t = match unit.clone().unwrap().as_str() {
                    "f" | "fahrenheit" => "fahrenheit",
                    _ => "celsius"
                };
            }

            if date.is_some() {
                date_t = date.clone().unwrap();
            }

        }
    }
        bar.set_message("Checking Weather via Open Meteo");
        let resp = reqwest::get(format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=temperature_2m_max,temperature_2m_min,uv_index_max,precipitation_sum&temperature_unit={}&start_date={}&end_date={}", latitude, longitude, unit_t, date_t, date_t))
            .await?
            .text()
            .await?;

        let binding = resp.replace(",", "").replace(":", "").replace("[", "").replace("]", "").replace("{", "").replace("{", "").replace("}}", "");
        let data = binding.split("\"").collect::<Vec<&str>>();

        let unit = data[27];
        let max = data[48];
        let min = data[50];
        let uv_index_max = data[52];
        let precipitation_sum = data[54];

        bar.set_message("");
        bar.finish();

        println!("\n--- Forcast For {} ---\nMax Temp = {}{}\nMin Temp = {}{}\nUV Index = {}\nRain Chance = {}%\n", date_t, max, unit, min, unit, uv_index_max, precipitation_sum);
    
    //https://geocoding-api.open-meteo.com/v1/search?name=Berlin&count=1&language=en&format=json    
    
    Ok(())
    
}

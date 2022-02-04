use chrono::{Datelike, Duration};
use chrono::naive::NaiveDate;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use clap::Parser;

use pla2html::pla::parser::PlaParser;

#[macro_use]
extern crate horrorshow;
// use horrorshow::prelude::*;
use horrorshow::helper::doctype;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Input file name. Should be in .pla format
    #[clap(short)]
    input_file:String,

    /// Output file name
    #[clap(short)]
    output_file: String,
}

fn main() {
    let args = Args::parse();

    // Parse the input pla file
    let _pla_parser = match PlaParser::new(Path::new(&args.input_file)) {
        Ok(p) => p,
        Err(why) => panic!("Unable to parse {} due to {}", args.input_file, why),
    };

    // Create the main html page with the grid
    let start_date = NaiveDate::parse_from_str("2021-10-01", "%Y-%m-%d").unwrap();
    let end_date = NaiveDate::parse_from_str("2021-12-31", "%Y-%m-%d").unwrap();
    let duration = end_date - start_date;
    let duration_days = duration.num_days();
    let formatted_month_name = format!("{}", start_date.format("%B %Y"));
    let mut months: Vec<String> = vec![formatted_month_name];
    let mut month_num: u32 = start_date.month();
    let mut year = start_date.year();
    let mut days_in_month: Vec<i64> = vec![get_days_in_month(year, month_num)];

    let mut last_month_idx = 0;
    for i in 1..duration_days {
        let last_month_name = String::from(months.get(last_month_idx).unwrap());
        let month_name = format!("{}", (start_date + Duration::days(i)).format("%B %Y"));
        if month_name != last_month_name {
            month_num = month_num + 1;
            if month_num > 12 {
                month_num = 1;
                year = year + 1;
            }

            last_month_idx = last_month_idx + 1;
            months.push(String::from(&month_name));
            days_in_month.push(get_days_in_month(year, month_num));
        }
    }

    let stylings = r#"
        body {
            font-family: sans-serif;
            font-size: 10pt;
        }

        table {
            border-collapse: collapse;
            position: relative;
            border-spacing: 0;
        }

        .monthName {
            text-align: center;
            font-size: 32pt;
            font-family: sans-serif;
        }

        td.emptyCell {
            background-color: white !important;
            border: none !important;
        }

        td.beerTitle {
            font-size: 14pt;
            font-family: sans-serif;
            padding-right: 1rem;
            width: 200px;
        }

        .beerTitle-spacer {
            padding-top: 1rem;
            padding-bottom: 1rem;
        }

        td.day {
            border-spacing: 0;
        }

        td.dayOfMonth {
          border-bottom: 1px solid black;
        }

        td.lastDayOfMonth:not(:last-child) {
            border-right: 3px solid black
        }

        td:not(.monthName):not(.headerRow):nth-child(2n) {
            background-color: lightgray;
        }

        td.headerRow:nth-child(2n+1) {
            background-color: lightgray;
            position: relative;
            overflow: visible;
        }

        div.full-bubble {
          position: absolute;
          top: calc(50% - 13px);
          border-radius: 4px;
          border: 1px solid darkgray;
          background-color: lightgreen;
          padding: .25rem;
          z-index: 25;
        }

        div.spacer {
          width: 43px;
          padding: 0;
          margin: 0;
          text-align: center;
        }
    "#;

    let num_months = months.len();
    let process_length = 22;
    let actual = format!("{}", html! {
        : doctype::HTML;
        html {
            head {
                title : format_args!("{} - {}", months.get(0).unwrap(), months.get(months.len() - 1).unwrap());
                style: format_args!("{}", stylings);
            }
            body {
                table {
                    tr {
                        td(class="emptyCell") {}
                        td(class="emptyCell") {}
                        @ for i in 0..num_months {
                            td (class="monthName", colspan=format!("{}", days_in_month.get(i).unwrap())) {
                                : format_args!("{}", months.get(i).unwrap());
                            }
                        }
                    }

                    tr {
                        td(class="emptyCell") {}
                        td(class="emptyCell") {}

                        @ for i in 0..num_months {
                            @ for j in 1..(*days_in_month.get(i).unwrap() as u32) + 1 {
                                td(class = if j == (*days_in_month.get(i).unwrap() as u32) { "day lastDayOfMonth dayOfMonth" } else { "dayOfMonth" }) {
                                    div (class="spacer") {
                                        : format_args!("{}", j);
                                    }
                                }
                            }
                        }
                    }

                    tr {
                        td(class="beerTitle", colspan="2") {
                            div(class="beerTitle-spacer") {
                                : "Autumn's"
                            }
                        }

                        @ for i in 0..num_months {
                            @ for j in 1..(*days_in_month.get(i).unwrap() as u32) + 1 {
                                td(class="day headerRow") {
                                    @if i == 0 && j == 12 {
                                        div(class="full-bubble", style=format!("width: {}px", process_length * 45 - 6)) {
                                            : "Autumn's Early Arrival Blonde"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // Write to a file for output
    let path = Path::new(&args.output_file);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(actual.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
    // println!("{}", actual);
}

fn get_days_in_month(year: i32, month: u32) -> i64 {
    NaiveDate::from_ymd(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .signed_duration_since(NaiveDate::from_ymd(year, month, 1))
    .num_days()
}
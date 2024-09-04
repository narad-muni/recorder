use std::fs::File;
use std::io::stdin;
use std::io::{self, Read};
use std::time::Instant;

use chrono::{Local, TimeZone, Utc};

fn main() {
    let ds = format!("{:?}", Local::now().date_naive());

    println!("{ds}");
}

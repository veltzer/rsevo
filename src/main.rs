mod config;

use anyhow::Result;
use clap::Parser;
use config::Config;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rsevo", about = "GA-based school/uni timetabling")]
struct Cli {
    #[arg(short, long, default_value = "examples/school.yaml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = Config::from_yaml_file(&cli.config)?;

    println!("Loaded {}", cli.config.display());
    println!(
        "  {} days x {} periods = {} slots/week",
        cfg.schedule.days.len(),
        cfg.schedule.periods_per_day,
        cfg.total_slots()
    );
    println!(
        "  rooms={} teachers={} groups={} subjects={}",
        cfg.rooms.len(),
        cfg.teachers.len(),
        cfg.groups.len(),
        cfg.subjects.len()
    );
    println!(
        "  {} lesson types, {} total lesson instances to place",
        cfg.lessons.len(),
        cfg.total_lesson_instances()
    );

    let demand = cfg.total_lesson_instances();
    let supply_per_room = cfg.total_slots();
    let total_supply = supply_per_room * cfg.rooms.len() as u32;
    println!("  capacity check: demand={demand} room-slots-available={total_supply}");

    Ok(())
}

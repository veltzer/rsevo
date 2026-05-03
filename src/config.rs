use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub schedule: Schedule,
    pub rooms: Vec<Room>,
    pub teachers: Vec<Teacher>,
    pub groups: Vec<Group>,
    pub subjects: Vec<Subject>,
    pub lessons: Vec<Lesson>,
    #[serde(default)]
    pub preferences: Vec<Preference>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub days: Vec<String>,
    pub periods_per_day: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub capacity: u32,
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Teacher {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub unavailable: Vec<Slot>,
    pub max_periods_per_day: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subject {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub requires_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lesson {
    pub group: String,
    pub subject: String,
    pub teacher: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    pub day: String,
    pub period: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Preference {
    AvoidPeriod { period: u32, weight: i32 },
    SpreadSubject { subject: String, weight: i32 },
    TeacherCompactDay { weight: i32 },
}

impl Config {
    pub fn from_yaml_file(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading config file {}", path.display()))?;
        let cfg: Config = serde_yaml::from_str(&text)
            .with_context(|| format!("parsing YAML in {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        for lesson in &self.lessons {
            anyhow::ensure!(
                self.groups.iter().any(|g| g.id == lesson.group),
                "lesson references unknown group {}",
                lesson.group
            );
            anyhow::ensure!(
                self.subjects.iter().any(|s| s.id == lesson.subject),
                "lesson references unknown subject {}",
                lesson.subject
            );
            anyhow::ensure!(
                self.teachers.iter().any(|t| t.id == lesson.teacher),
                "lesson references unknown teacher {}",
                lesson.teacher
            );
        }
        Ok(())
    }

    pub fn total_lesson_instances(&self) -> u32 {
        self.lessons.iter().map(|l| l.count).sum()
    }

    pub fn total_slots(&self) -> u32 {
        self.schedule.days.len() as u32 * self.schedule.periods_per_day
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal, internally consistent config used as the basis for the tests.
    fn valid_yaml() -> &'static str {
        r#"
schedule:
  days: [Mon, Tue, Wed]
  periods_per_day: 4
rooms:
  - { id: R1, capacity: 30 }
teachers:
  - { id: T1, name: Smith, max_periods_per_day: 6 }
groups:
  - { id: G1, size: 20 }
subjects:
  - { id: S1, name: Math }
lessons:
  - { group: G1, subject: S1, teacher: T1, count: 3 }
  - { group: G1, subject: S1, teacher: T1, count: 2 }
"#
    }

    fn parse(yaml: &str) -> Result<Config> {
        let cfg: Config = serde_yaml::from_str(yaml)?;
        cfg.validate()?;
        Ok(cfg)
    }

    #[test]
    fn parses_a_valid_config() {
        let cfg = parse(valid_yaml()).expect("valid config should parse");
        assert_eq!(cfg.schedule.days.len(), 3);
        assert_eq!(cfg.rooms.len(), 1);
        assert_eq!(cfg.teachers.len(), 1);
        assert_eq!(cfg.lessons.len(), 2);
    }

    #[test]
    fn total_slots_is_days_times_periods() {
        let cfg = parse(valid_yaml()).unwrap();
        assert_eq!(cfg.total_slots(), 12);
    }

    #[test]
    fn total_lesson_instances_sums_counts() {
        let cfg = parse(valid_yaml()).unwrap();
        assert_eq!(cfg.total_lesson_instances(), 5);
    }

    #[test]
    fn preferences_default_to_empty_when_absent() {
        let cfg = parse(valid_yaml()).unwrap();
        assert!(cfg.preferences.is_empty());
    }

    #[test]
    fn validate_rejects_unknown_group() {
        let yaml = valid_yaml().replace("group: G1", "group: G_MISSING");
        let err = parse(&yaml).expect_err("unknown group must be rejected");
        assert!(
            err.to_string().contains("unknown group"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_rejects_unknown_subject() {
        let yaml = valid_yaml().replace("subject: S1", "subject: S_MISSING");
        let err = parse(&yaml).expect_err("unknown subject must be rejected");
        assert!(
            err.to_string().contains("unknown subject"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_rejects_unknown_teacher() {
        let yaml = valid_yaml().replace("teacher: T1", "teacher: T_MISSING");
        let err = parse(&yaml).expect_err("unknown teacher must be rejected");
        assert!(
            err.to_string().contains("unknown teacher"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_yaml_file_reports_missing_file() {
        let err = Config::from_yaml_file(Path::new("does/not/exist.yaml"))
            .expect_err("missing file must be an error");
        assert!(
            err.to_string().contains("reading config file"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn bundled_example_config_is_valid() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/school.yaml");
        let cfg = Config::from_yaml_file(&path).expect("bundled example must stay valid");
        assert_eq!(cfg.total_slots(), 40);
        assert!(cfg.total_lesson_instances() > 0);
    }
}

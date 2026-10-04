# TOFIX

Findings from a code scan on 2026-10-04.

## High

- `README.md:3-4` - says the tool "builds school and university timetables with a genetic algorithm", but `src/main.rs:15-45` only loads the YAML, prints counts and exits; no timetable is produced. Either implement the GA or state in README, `Cargo.toml:6` and `docs/src/introduction.md:3` that this is an early config-loader stage (and stop cutting releases, `Cargo.toml:3` is at 0.1.4, until it does something).

## Medium

- `Cargo.toml:11` - `genetic_algorithm` is declared (and compiled, `Cargo.lock:221`) but never used anywhere in `src/`; drop it until the solver lands, or wire it in.
- `src/config.rs:131-150` - `validate()` only checks that lessons reference known group/subject/teacher; it does not check that `teachers[].unavailable` slots name a day in `schedule.days` and a period within `1..=periods_per_day`, that `spread_subject` preferences reference a known subject, that ids are unique, or that preference weights are positive as `docs/src/constraints.md:61` requires. A typo like `day: Fir` is silently accepted; add these checks with tests.
- `src/main.rs:39-42` - the "capacity check" only prints demand vs room-slots and never fails when demand exceeds supply, while `docs/src/constraints.md:88-92` specifies feasibility pre-checks that abort with a clear error (feature+capacity room exists per lesson, teacher weekly load vs `max_periods_per_day x days`, availability vs load). Implement them in `Config::validate` and return an error.

## Low

- `src/main.rs:41` - `supply_per_room * cfg.rooms.len() as u32` (and `src/config.rs:157`) can overflow/truncate on large inputs and panics in debug builds; use `checked_mul` or `u64`.
- `src/config.rs:2` - `Serialize` is derived on every config type but nothing serializes them; drop it unless output of configs is planned.

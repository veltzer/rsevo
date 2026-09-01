# rsevo

Evolutionary timetabling engine: builds school and university timetables
with a genetic algorithm.

The problem (classes, teachers, rooms, slots and constraints) is described
in a YAML file; see `examples/school.yaml` for the format.

Full documentation lives in the `docs/` mdBook.

## Quick start

```bash
cargo run --release                                  # uses examples/school.yaml
cargo run --release -- --config path/to/problem.yaml
```

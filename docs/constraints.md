# rsevo — Constraints Specification

This document enumerates every constraint the timetabling engine must understand. It is the source of truth for the YAML schema, the validator, and the fitness function.

Status: **DRAFT** — open questions are marked `TBD`.

## Vocabulary

- **Slot** — a single (day, period) pair on the weekly grid.
- **Lesson instance** — one atomic thing to be placed on the timetable. A lesson type with `count: 5` produces 5 lesson instances.
- **Assignment** — a placement of a lesson instance into a (slot, room).
- **Solution** — a complete set of assignments, one per lesson instance.

## Constraint classification

Every constraint is one of:

- **Hard** — a solution that violates it is **invalid**. The engine must never return such a solution; the decoder/repair phase must guarantee feasibility.
- **Soft** — a solution that violates it is **suboptimal but legal**. Each violation contributes a weighted penalty to the fitness score. The GA optimizes the weighted sum.

A configuration whose hard constraints are mutually unsatisfiable is **infeasible** — the engine must detect this and report it, not silently produce a bad timetable.

---

## Hard constraints

### H1 — Single placement
Every lesson instance is placed exactly once. No instance is dropped; none is duplicated.

### H2 — Teacher non-overlap
A teacher is in at most one room in any given slot.

### H3 — Group non-overlap
A student group attends at most one lesson in any given slot.

### H4 — Room non-overlap
A room hosts at most one lesson in any given slot.

### H5 — Teacher availability
A lesson taught by teacher `T` cannot be placed in any slot listed in `T.unavailable`.

### H6 — Room feature requirements
If subject `S` declares `requires_features: [...]`, the room hosting any lesson of `S` must include all listed features.

### H7 — Room capacity
The room hosting a lesson for group `G` must have `capacity >= G.size`.

### H8 — Teacher daily load cap
For each (teacher, day) pair, the number of assigned lessons must not exceed `teacher.max_periods_per_day`.

### H9 — Block integrity *(only if blocks are added — see Open Questions)*
A multi-period lesson block occupies consecutive periods within the same day, in the same room, with the same teacher and group throughout.

---

## Soft constraints (preferences)

Each preference declared in YAML carries a positive `weight`. Fitness penalty for a solution = Σ over violations of `weight × violation_magnitude`. Lower is better.

### S1 — `avoid_period`
**Spec:** `{ kind: avoid_period, period: P, weight: W }`
**Penalty:** `W` per lesson placed in period `P`.
**Intent:** Discourage scheduling in unpopular slots (e.g. first period).

### S2 — `spread_subject`
**Spec:** `{ kind: spread_subject, subject: SUBJ, weight: W }`
**Penalty:** For each group, count days with ≥2 lessons of `SUBJ`. Penalty = `W × Σ (lessons_of_subj_on_day - 1)` over days with surplus.
**Intent:** Avoid bunching the same subject (e.g. three math classes in one day).

### S3 — `teacher_compact_day`
**Spec:** `{ kind: teacher_compact_day, weight: W }`
**Penalty:** For each (teacher, day), let `span = last_period - first_period + 1` and `taught = count of lessons that day`. Gaps = `span - taught`. Penalty = `W × Σ gaps`.
**Intent:** Minimize idle gaps in a teacher's day.

### S4 — `group_compact_day` *(proposed)*
Symmetric to S3 but for student groups. **TBD: include?**

### S5 — `prefer_room` *(proposed)*
Bias a subject or teacher toward a specific room when possible. **TBD: include?**

---

## Constraint interactions worth flagging

- **H6 + H7 combined** can render a configuration infeasible if (e.g.) the only room with `computers` has `capacity: 20` but a CS lesson is scheduled for a group of 30. The validator should pre-check: for every lesson, ≥1 room satisfies both feature and capacity requirements.
- **H8 + lesson counts** can be infeasible: if a teacher's total weekly load exceeds `max_periods_per_day × days`, no schedule exists.
- **H2 + H5** interact when a teacher's available hours are barely enough — the validator should compare each teacher's available slots against their assigned lesson count.

The validator should run these **feasibility pre-checks before the GA starts** and abort with a clear error rather than letting the GA spin forever on an unsolvable instance.

---

## Open questions

1. **Lesson blocks (H9).** Add a `duration` field to lessons so labs / double-periods are first-class? If yes, blocks must stay within a single day (no spanning lunch / end-of-day).
2. **Lunch / break periods.** Should the schedule grid support marking certain periods as non-teaching (lunch, assembly)? Currently every period is teachable.
3. **Group composition.** Real schools have shared electives where a "class" is actually a subset across groups. Out of scope for v1?
4. **Teacher preferences vs. availability.** Currently `unavailable` is hard. Do we also want a soft `prefers_not` list?
5. **Multi-room subjects.** PE might split a group across gym + field. Out of scope for v1?
6. **Room transition cost.** Penalize a group having back-to-back lessons in distant rooms? Probably v2.
7. **Fairness across teachers.** Should the fitness include a term that equalizes teacher load / soft-preference quality? Otherwise the GA may dump all the bad slots on one teacher.

Resolve each of these before we lock the YAML schema and start on the encoding.

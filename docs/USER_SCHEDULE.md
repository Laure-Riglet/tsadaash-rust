# Spec: User “Schedule Templates” (availability + capabilities + location constraints)

## Goal

Implement a domain module that models **recurring “default agendas”** (weekly templates) for a user, and can:

1. Expand a schedule template into concrete **time blocks** for a date range.
2. Determine whether a given `Task` can be scheduled in a given time block, based on:
    - Availability status: `Unavailable` / `BusyButFlexible` / `Available`
    - Capabilities: hands/eyes/ears/speech/cognitive, device access, mobility
    - Location constraints: derived from existing `Location` in the `user` codebase

3. Provide utilities to find eligible time windows for short tasks during `BusyButFlexible` periods.

### Non-goals (v1)

- Full iCal RRULE support
- Complex event duration modeling
- Timezone database / DST edge-case perfection beyond what `chrono` / `time` provides
- Persistency layer (repositories), unless already standard in the codebase

---

## Assumptions / Existing Types

- `User` exists and includes `locations: Vec<Option<Location>>`
- `Task` exists (domain task) — but it may not contain scheduling requirements yet.
- `Location` exists (already implemented)
- Use `chrono` (recommended) for date/time types (or `time` if that’s what the project uses). Choose one and keep consistent.

---

## Domain Module Layout

Create a new module under your domain (example path):

- `src/domain/entities/schedule/mod.rs`
- `src/domain/entities/schedule/types.rs`
- `src/domain/entities/schedule/template.rs`
- `src/domain/entities/schedule/expansion.rs`
- `src/domain/entities/schedule/matching.rs`

Re-export public API from `mod.rs`.

---

## Public API Requirements

### 1) Core Types

#### Availability

```rust
pub enum AvailabilityKind {
    Unavailable(UnavailableReason),
    BusyButFlexible,
    Available,
}

pub enum UnavailableReason {
    Sleep,
    Work,
    Appointment,
    Focus,
    Other(String),
}
```

##### **Rules**

- `Unavailable`: tasks are rejected by default.
- `Available`: tasks can be accepted if requirements are satisfied.
- `BusyButFlexible`: only “short & location-free” tasks may be accepted (see Matching Rules).

#### Capability modeling

Use ordinal “levels” for key constraints:

```rust
pub enum AvailabilityLevel {
    None,
    Limited,
    Full,
}
```

Device and mobility:

```rust
pub enum DeviceAccess {
    None,
    PhoneOnly,
    Computer,
}

pub enum Mobility {
    Stationary,
    InTransit,
    Driving,
}
```

Capability set:

```rust
pub struct CapabilitySet {
    pub hands: AvailabilityLevel,
    pub eyes: AvailabilityLevel,
    pub speech: AvailabilityLevel,
    pub cognitive: AvailabilityLevel,
    pub device: DeviceAccess,
    pub mobility: Mobility,
}
```

Provide at least:

- `CapabilitySet::free()` (max capabilities)
- `CapabilitySet::driving()` (hands/eyes none, cognitive limited/??, device none/phone??, mobility driving)
- `CapabilitySet::in_transit()` (hands/eyes limited, device phone, mobility in_transit)

(Exact presets can be tuned; tests should reflect agreed values.)

#### Location constraints

Do **not** duplicate `User.locations`. Use constraints that can match a `current_location: Option<&Location>`.

```rust
pub enum LocationConstraint {
    Any,
    MustBeKnown,
    MustBeUnknown,
    MustBeOneOf(Vec<Location>),
}
```

Comparison semantics:

- `Any` always ok
- `MustBeKnown`: `current_location.is_some()`
- `MustBeUnknown`: `current_location.is_none()`
- `MustBeOneOf(xs)`: `current_location` must be `Some(l)` and equals one in `xs`

> NOTE: requires `Location: Eq` or a deterministic equality rule. If `Location` is complex, define an equality key (e.g., `location.id()`).

---

### 2) Schedule Template Types (weekly recurring rules)

#### Weekday representation

Use `chrono::Weekday` (if using chrono). Otherwise define your own.

#### Time-of-day

Use `chrono::NaiveTime` (or `time::Time`) for local time-of-day.

#### Recurring rule

A rule represents a weekly repeating block, e.g. “Mon-Fri 09:00–12:00 Work”.

```rust
pub struct RecurringRule {
    pub days: Vec<Weekday>,            // must be non-empty
    pub start: NaiveTime,
    pub end: NaiveTime,                // can be < start to represent overnight
    pub availability: AvailabilityKind,
    pub capabilities: CapabilitySet,
    pub location_constraint: LocationConstraint,
    pub label: Option<String>,
    pub priority: i16,                 // higher overrides lower
}
```

##### **Overnight semantics**

- If `end <= start`, the rule spans midnight into the next day.

#### Template

```rust
pub struct ScheduleTemplate {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub timezone: String,              // store IANA name or project standard
    pub rules: Vec<RecurringRule>,
}
```

---

### 3) Expanded Time Blocks

Expansion output is concrete blocks with absolute times:

```rust
pub struct TimeBlock {
    pub start: DateTime<TzOrLocal>,
    pub end: DateTime<TzOrLocal>,
    pub availability: AvailabilityKind,
    pub capabilities: CapabilitySet,
    pub location_constraint: LocationConstraint,
    pub label: Option<String>,
    pub priority: i16,
}
```

Type parameter `TzOrLocal`:

- Use `chrono::DateTime<FixedOffset>` or `DateTime<Tz>` if using `chrono-tz`.
- If project doesn’t want TZ dependency, use local naive datetimes and treat them as “user local”.

#### **Invariants**

- `start < end`
- adjacent blocks may be mergeable (see merging rules)

---

## Expansion Engine Spec

### Function signature

```rust
pub fn expand_template(
    template: &ScheduleTemplate,
    range_start: DateTime<...>,
    range_end: DateTime<...>,
) -> Vec<TimeBlock>;
```

#### **Behavior**

1. Generate all rule occurrences that overlap `[range_start, range_end)`.
2. If overlaps exist, resolve conflicts using:
    - Higher `priority` wins for overlapping instants.
    - If same priority: deterministic tie-breaker:
        - Prefer `Unavailable` over `BusyButFlexible` over `Available` OR keep the first rule order.
        - Choose one and test it. Recommended: **prefer more restrictive**.

3. After resolution, **merge adjacent blocks** if all these fields are equal:
    - availability, capabilities, location_constraint, priority, label (optional: label can break merge)

4. Return sorted by start time.

### Conflict Resolution Algorithm (recommended)

Implement a “sweep-line” approach:

- Collect all boundary instants from all occurrences
- Sort unique boundaries
- For each segment between boundaries, choose winning rule (highest priority; then restrictive tie-break)
- Create a block for each segment and merge adjacent equal blocks

This avoids dealing with complex interval subtraction logic.

---

## Task Scheduling / Matching Spec

### Task requirements

You have two options:

#### **Option A (recommended):** Create a small adapter trait that extracts scheduling requirements from `Task` without changing the existing `Task` struct immediately

```rust
pub trait SchedulableTask {
    fn estimated_duration_minutes(&self) -> u32;
    fn requires_location(&self) -> bool; // “location-free” if false
    fn min_hands(&self) -> AvailabilityLevel;
    fn min_eyes(&self) -> AvailabilityLevel;
    fn min_speech(&self) -> AvailabilityLevel;
    fn min_cognitive(&self) -> AvailabilityLevel;
    fn min_device(&self) -> DeviceAccess;
    fn allowed_mobility(&self) -> Vec<Mobility>;
}
```

Then implement it for your `Task` (in an impl block in schedule module, if allowed), or provide a wrapper `TaskSchedulingProfile`.

#### **Option B:** Add a `TaskSchedulingProfile` struct and pass it explicitly to the matcher. This avoids touching `Task` at all

### Matching function

```rust
pub fn can_schedule_task_in_block(
    task: &impl SchedulableTask,
    block: &TimeBlock,
    current_location: Option<&Location>,
) -> bool;
```

### Matching Rules

#### 1) Availability gating

- If `block.availability == Unavailable(_)` → return false
- If `block.availability == Available` → check normal requirements (capabilities + location)
- If `block.availability == BusyButFlexible` → only allow “micro tasks”:
  - `task.estimated_duration_minutes() <= BUSY_FLEX_MAX_MINUTES` (config const, default 15)
  - `task.requires_location() == false`
  - `block.location_constraint` must accept unknown/any location:
    - `LocationConstraint::Any` OR `MustBeUnknown`

  - Additionally, enforce a “low friction” constraint:
    - `task.min_device() != DeviceAccess::Computer` (computer tasks not allowed)
    - and `task.min_hands() <= Limited` (no full manual tasks)
    - and `task.min_eyes() <= Limited` (no full visual attention tasks)

  - Then apply capabilities matching as usual.

> These are v1 defaults; keep them constants so they’re adjustable.

#### 2) Location constraint matching

- Apply `LocationConstraint` vs `current_location` as described above.
- If task requires location (`requires_location == true`), then `current_location` must be `Some(_)` (unless you decide requirements are separate from constraints).

#### 3) Capabilities matching

A block’s capability must be **>=** task’s minimum:

- `block.capabilities.hands >= task.min_hands()`
- same for eyes/speech/cognitive

Device matching:

- Define order: `None < PhoneOnly < Computer`
- `block.capabilities.device >= task.min_device()`

Mobility:

- if `task.allowed_mobility()` is non-empty:
  - must contain `block.capabilities.mobility`

- else default allow all

---

## Finding candidate slots (optional v1 but useful)

Provide:

```rust
pub fn find_candidate_slots(
    blocks: &[TimeBlock],
    task: &impl SchedulableTask,
    current_location: Option<&Location>,
) -> Vec<(DateTime<...>, DateTime<...>)>;
```

Behavior:

- Return subranges within blocks where `can_schedule_task_in_block` is true and block duration >= task duration.
- For v1, return the whole block as a candidate (UI can pick start). Later can slice.

---

## Tests (must-have)

Create tests under `src/domain/entities/schedule/tests.rs` or module tests.

### Test helpers

- Provide a small `FakeTask` implementing `SchedulableTask`
- Provide a simplified `Location` or use real `Location` if easy to construct.
- Use fixed dates (no `Local::now()`).

### 1) Module declaration / compilation test

- Ensure `schedule` module compiles and re-exports public types:
  - `ScheduleTemplate`, `RecurringRule`, `TimeBlock`, `AvailabilityKind`, etc.

### 2) LocationConstraint matching tests

- `Any` works with None and Some
- `MustBeKnown` rejects None
- `MustBeUnknown` rejects Some
- `MustBeOneOf` accepts only matching location

### 3) Capability matching tests

- Block with `hands=Limited` allows task requiring `None` and `Limited`, rejects `Full`
- Device ordering works (`PhoneOnly` rejects `Computer` requirement)

### 4) BusyButFlexible gating tests

- Task 10 min, location-free, low requirements → allowed in BusyButFlexible
- Task 20 min → rejected (if max 15)
- Task requiring location → rejected
- Task requiring computer → rejected
- Task requiring Full hands → rejected

### 5) Recurrence expansion tests (non-overnight)

Example:

- Rule: Mon-Fri 09:00–10:00 Available
- Expand range: a week including a Monday
- Expect 5 occurrences, correct datetimes, sorted

### 6) Overnight rule expansion test

- Rule: Daily 23:00–07:00 Unavailable(Sleep)
- Expand for 2 days
- Verify blocks cross midnight correctly and overlap range properly

### 7) Priority conflict resolution test

- Base rule: Mon 09:00–12:00 Available (priority 0)
- Override rule: Mon 10:00–11:00 Unavailable(Work) (priority 10)
- Expand Monday range
- Expected result segments:
  - 09–10 Available
  - 10–11 Unavailable
  - 11–12 Available

- Also test merging: if two consecutive segments have same attributes, they merge

### 8) Tie-breaker test (same priority overlap)

- Two rules same priority overlapping
- Ensure deterministic output (choose restrictive availability or first rule order)
- Whatever you pick, encode it in the test

---

## Implementation Notes / Engineering Constraints

- Keep schedule logic in the domain layer (no I/O).
- Avoid `pub` fields everywhere if you prefer invariants, but tests should access key data via getters or `#[derive]`.
- Avoid heavy dependencies unless already used.
- Ensure deterministic sorting and stable behavior.
- All time arithmetic must guarantee `start < end` for output blocks.

---

## Deliverables

1. New schedule domain module with:
    - types, template structs, matching logic, expansion logic

2. Test suite covering the items above
3. `mod.rs` exposing a clean public API:
    - `pub use template::ScheduleTemplate;`
    - `pub use template::RecurringRule;`
    - `pub use expansion::expand_template;`
    - `pub use matching::{can_schedule_task_in_block, find_candidate_slots, SchedulableTask};`

---

## Constants (v1 defaults)

- `BUSY_FLEX_MAX_MINUTES: u32 = 15`

Optionally:

- `BUSY_FLEX_MAX_HANDS = AvailabilityLevel::Limited`
- `BUSY_FLEX_MAX_EYES = AvailabilityLevel::Limited`
- `BUSY_FLEX_MAX_DEVICE = DeviceAccess::PhoneOnly`

---

If you paste this into your IDE agent, it should be able to implement end-to-end.

If you want, I can also generate a **minimal “FakeTask + expected tests skeleton”** that matches your project’s existing test framework (plain `#[test]`, `rstest`, etc.).

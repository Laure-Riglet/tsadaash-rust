# Application Layer

## C’est quoi exactement l’**Application Layer** ?

👉 **L’application layer est le chef d’orchestre.**
👉 Elle **ne décide pas des règles métier**.
👉 Elle **ne sait pas comment les données sont stockées**.
👉 Elle **coordonne**.

> Si le domain répond à _“est-ce que c’est possible ?”_
> l’application layer répond à _“quoi faire maintenant, dans quel ordre, avec quelles données”_.

---

### Règle d’or (à garder en tête)

> **Le domain ne connaît pas l’application.
> L’application connaît le domain.**

---

## Ce qu’on met DANS l’application layer

### 1️⃣ Des _Use Cases_ (ou Application Services)

C’est le cœur.

Un use case = **une intention utilisateur ou système**, pas une entité.

Exemples concrets pour Tsadaash :

- `CreateTask`
- `UpdateTaskDuration`
- `SuggestTimeSlotsForTask`
- `ActivateScheduleTemplate`
- `GetUserDailyAgenda`
- `RescheduleTask`

👉 Ce sont des **verbes**, pas des noms de concepts.

#### Exemple

```rust
pub struct SuggestTimeSlots {
    schedule_service: ScheduleService,
}

impl SuggestTimeSlots {
    pub fn execute(
        &self,
        user_id: UserId,
        task_id: TaskId,
        range: DateRange,
    ) -> Vec<TimeSlotSuggestion> {
        // orchestration only
    }
}
```

---

### 2️⃣ De l’orchestration (mais pas de logique métier)

L’application layer peut :

- charger des données (via repositories)
- appeler le domain
- combiner plusieurs appels
- gérer les erreurs
- appliquer des règles de flux

Mais **pas** :

- décider si une tâche est faisable
- calculer les overlaps
- matcher capabilities / contraintes

👉 Ça, c’est déjà dans ton domain.

---

### 3️⃣ Des DTOs (Data Transfer Objects)

Tu ne passes **pas** tes entités de domain directement partout.

L’application layer :

- adapte les données à l’usage
- simplifie pour l’UI / CLI / API

Exemple :

```rust
pub struct TimeSlotSuggestion {
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub score: u8,
    pub reason: String,
}
```

👉 Ce type **n’a rien à faire dans le domain**.

---

### 4️⃣ Les règles “techno-métier”

Ce sont les règles :

- pas vraiment business
- pas vraiment techniques
- mais liées au produit

Exemples :

- “On suggère max 5 créneaux”
- “On ne propose pas plus de 7 jours à l’avance”
- “On ignore les slots < 10 minutes”
- “On trie par préférence cognitive”

👉 L’application layer est le bon endroit.

---

### 5️⃣ La gestion de la config (et du temps “maintenant”)

Très important.

Le domain ne devrait **pas** faire :

- `now()`
- `env::var()`
- `timezone = ...`

L’application layer :

- récupère `now`
- injecte la config
- passe tout au domain

Exemple :

```rust
let now = clock.now();
let config = app_config.schedule;
let blocks = expand_template(template, now, now + 7.days());
```

---

## Ce qu’on NE met PAS dans l’application layer

❌ Des structs comme `Task`, `User`, `Location`
❌ Des règles du type “une tâche est faisable si…”
❌ Du SQL
❌ Du HTTP
❌ De la sérialisation JSON
❌ De la logique de persistance

---

## Architecture concrète (Rust)

Une structure typique :

```bash
src/
├── domain/
│   ├── task/
│   ├── schedule/
│   └── user/
│
├── application/
│   ├── mod.rs
│   ├── services/
│   │   ├── suggest_time_slots.rs
│   │   ├── create_task.rs
│   │   └── activate_schedule.rs
│   │
│   ├── dto/
│   │   └── time_slot_suggestion.rs
│   │
│   └── errors.rs
│
├── infrastructure/
│   ├── persistence/
│   ├── clock.rs
│   └── config.rs
│
├── main.rs
```

---

## Exemple complet : SuggestTimeSlots (simplifié)

```rust
pub struct SuggestTimeSlots {
    task_repo: TaskRepository,
    schedule_repo: ScheduleRepository,
    clock: Clock,
    config: ScheduleConfig,
}

impl SuggestTimeSlots {
    pub fn execute(&self, user_id: UserId, task_id: TaskId) -> Vec<TimeSlotSuggestion> {
        let task = self.task_repo.get(task_id);
        let template = self.schedule_repo.active_template(user_id);

        let now = self.clock.now();
        let blocks = expand_template(
            &template,
            now,
            now + Duration::days(7),
        );

        blocks
            .into_iter()
            .filter(|block| {
                can_schedule_task_in_block(
                    &task,
                    block,
                    self.task_repo.current_location(user_id),
                    &self.config,
                )
            })
            .map(TimeSlotSuggestion::from_block)
            .collect()
    }
}
```

👉 Aucune logique métier ici.
👉 Juste de la coordination.

---

## Une phrase-clé pour t’orienter

> **Si tu peux écrire un test du code sans instancier de repo, tu es trop bas.
> Si tu dois mocker tout le domain, tu es trop haut.**

---

## Indicateur que tu fais bien les choses

- ton application layer **grossit**
- ton domain **reste stable**
- tes tests de domain ne cassent jamais
- seuls les tests d’application changent quand le produit évolue

---

## En résumé

### Users

- RegisterUser
- UpdateUserSettings

### Schedule

- CreateScheduleTemplate
- UpsertRecurringRule
- SetActiveScheduleTemplate

### Tasks

- CreateTask
- UpdateTask
- CompleteOccurrenceRep

### Views

- GetDayOverview

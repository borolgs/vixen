use rusqlite::Connection;

pub const SCHEMA: &str = r#"
create table issues (
    id       integer primary key,
    title    text    not null,
    -- 0 backlog, 1 todo, 2 in progress, 3 done, 4 canceled
    status   integer not null default 1 check (status between 0 and 4),
    -- 0 none, 1 low, 2 medium, 3 high, 4 urgent
    priority integer not null default 0 check (priority between 0 and 4),
    estimate integer check (estimate between 0 and 99),
    due      text    check (due is date(due)),
    -- The delete that removed the row, so one undo restores all of it.
    trashed  integer
) strict;

create view issues_v as
    select id, title, status, priority, estimate, due,
           status < 3 as open,
           coalesce(due < date('now') and status < 3, 0) as overdue
    from issues
    where trashed is null;
"#;

const VERBS: [&str; 11] = [
    "Fix", "Speed up", "Document", "Redesign", "Test", "Remove", "Rename", "Cache", "Audit",
    "Simplify", "Ship",
];

const THINGS: [&str; 24] = [
    "the sign-in redirect",
    "keyset pagination",
    "the toast stack",
    "asset hashing",
    "the search debounce",
    "the drawer focus trap",
    "base path handling",
    "combobox keyboard navigation",
    "the CSV export",
    "empty states",
    "the error pages",
    "the bundler cache",
    "form validation",
    "the sort headers",
    "row selection",
    "dark mode tokens",
    "session expiry",
    "the onboarding checklist",
    "webhook retries",
    "the audit log",
    "invite emails",
    "rate limiting",
    "the settings page",
    "image uploads",
];

const STATUSES: [u8; 12] = [0, 0, 0, 1, 1, 1, 2, 2, 3, 3, 3, 4];
const PRIORITIES: [u8; 10] = [0, 0, 1, 1, 2, 2, 2, 3, 3, 4];
const ESTIMATES: [Option<u8>; 7] = [None, Some(1), Some(2), Some(3), Some(5), Some(8), Some(13)];

pub fn seed(conn: &mut Connection) -> rusqlite::Result<()> {
    // xorshift: the same backlog on every start.
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut roll = |sides: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % sides as u64) as usize
    };

    let tx = conn.transaction()?;
    {
        let mut insert = tx.prepare(
            "insert into issues (title, status, priority, estimate, due)
                 values (?1, ?2, ?3, ?4, date('now', ?5))",
        )?;

        for n in 0..VERBS.len() * THINGS.len() {
            let title = format!(
                "{} {}",
                VERBS[n % VERBS.len()],
                THINGS[n * 5 % THINGS.len()]
            );
            let due = (roll(4) > 0).then(|| format!("{:+} days", roll(60) as i64 - 20));

            insert.execute((
                title,
                STATUSES[roll(STATUSES.len())],
                PRIORITIES[roll(PRIORITIES.len())],
                ESTIMATES[roll(ESTIMATES.len())],
                due,
            ))?;
        }
    }
    tx.commit()
}

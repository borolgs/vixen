use rusqlite::{Connection, OptionalExtension, Row, ToSql, named_params, params};
use vixen::{After, Page};

use super::{
    list::IssueSearch,
    model::{Change, Issue, SortField, Stats, Status},
};

pub const PAGE_SIZE: usize = 30;

const COLUMNS: &str = "id, title, status, priority, estimate, due, open, overdue";

fn issue(row: &Row) -> rusqlite::Result<Issue> {
    Ok(Issue {
        id: row.get(0)?,
        title: row.get(1)?,
        status: row.get(2)?,
        priority: row.get(3)?,
        estimate: row.get(4)?,
        due: row.get(5)?,
        open: row.get(6)?,
        overdue: row.get(7)?,
    })
}

pub fn search_issues(
    conn: &Connection,
    search: &IssueSearch,
) -> anyhow::Result<Page<Issue, After<i64>>> {
    let pattern = search
        .q
        .trim()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let (op, dir) = search.dir.sql();

    // A page continues after the last row's (key, id) in the sort direction.
    // Created is the id alone, so its key is a constant.
    let (key, after_key) = match search.sort {
        SortField::Created => ("''", ":key"),
        SortField::Title => ("title collate nocase", ":key"),
        SortField::Status => ("status", "cast(:key as integer)"),
        SortField::Priority => ("priority", "cast(:key as integer)"),
        SortField::Estimate => ("coalesce(estimate, -1)", "cast(:key as integer)"),
        SortField::Due => ("coalesce(due, '')", ":key"),
    };

    let after = search.after.as_ref();

    let mut stmt = conn.prepare(&format!(
        "select {COLUMNS}, cast({key} as text) from issues_v
             where (:status is null or status = :status)
               and title like '%' || :q || '%' escape '\\'
               and (:id is null or ({key}, id) {op} ({after_key}, :id))
             order by {key} {dir}, id {dir}
             limit :limit"
    ))?;

    let rows = stmt
        .query_map(
            named_params! {
                ":status": search.status,
                ":q": pattern,
                ":key": after.map(|after| after.key.as_str()),
                ":id": after.map(|after| after.id),
                ":limit": PAGE_SIZE as i64 + 1,
            },
            |row| Ok((issue(row)?, row.get::<_, String>(8)?)),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let page = Page::from_rows(rows, PAGE_SIZE, |(issue, key)| After {
        id: issue.id,
        key: key.clone(),
    });

    Ok(page.map(|(issue, _)| issue))
}

pub fn issue_by_id(conn: &Connection, id: i64) -> anyhow::Result<Option<Issue>> {
    let issue = conn
        .query_row(
            &format!("select {COLUMNS} from issues_v where id = ?1"),
            (id,),
            issue,
        )
        .optional()?;

    Ok(issue)
}

pub fn issues_by_ids(conn: &Connection, ids: &[i64]) -> anyhow::Result<Vec<Issue>> {
    let mut stmt = conn.prepare(&format!(
        "select {COLUMNS} from issues_v where id in (select value from json_each(?1))"
    ))?;
    let issues = stmt
        .query_map((json(ids),), issue)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(issues)
}

pub fn stats(conn: &Connection) -> anyhow::Result<Stats> {
    let stats = conn.query_row(
        "select count(*) filter (where open),
                count(*) filter (where status = ?1),
                coalesce(sum(estimate) filter (where open), 0),
                count(*) filter (where overdue)
             from issues_v",
        (Status::InProgress,),
        |row| {
            Ok(Stats {
                open: row.get(0)?,
                in_progress: row.get(1)?,
                points: row.get(2)?,
                overdue: row.get(3)?,
            })
        },
    )?;

    Ok(stats)
}

pub fn insert_issue(conn: &Connection, title: &str, status: Status) -> anyhow::Result<i64> {
    conn.execute(
        "insert into issues (title, status) values (?1, ?2)",
        (title, status),
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn update_issue(conn: &Connection, id: i64, change: &Change) -> anyhow::Result<()> {
    let (column, value): (&str, &dyn ToSql) = match change {
        Change::Title(title) => ("title", title),
        Change::Status(status) => ("status", status),
        Change::Priority(priority) => ("priority", priority),
        Change::Estimate(estimate) => ("estimate", estimate),
        Change::Due(due) => ("due", due),
    };

    conn.execute(
        &format!("update issues set {column} = ?2 where id = ?1"),
        params![id, value],
    )?;

    Ok(())
}

pub fn move_issues(conn: &Connection, ids: &[i64], status: Status) -> anyhow::Result<()> {
    conn.execute(
        "update issues set status = ?2 where id in (select value from json_each(?1))",
        (json(ids), status),
    )?;

    Ok(())
}

/// Returns the batch to pass to [`restore_issues`].
pub fn trash_issues(conn: &Connection, ids: &[i64]) -> anyhow::Result<i64> {
    let batch = conn.query_row(
        "select coalesce(max(trashed), 0) + 1 from issues",
        (),
        |row| row.get(0),
    )?;

    conn.execute(
        "update issues set trashed = ?2
             where trashed is null and id in (select value from json_each(?1))",
        (json(ids), batch),
    )?;

    Ok(batch)
}

pub fn restore_issues(conn: &Connection, batch: i64) -> anyhow::Result<usize> {
    let restored = conn.execute(
        "update issues set trashed = null where trashed = ?1",
        (batch,),
    )?;

    Ok(restored)
}

fn json(ids: &[i64]) -> String {
    let ids: Vec<_> = ids.iter().map(i64::to_string).collect();

    format!("[{}]", ids.join(","))
}

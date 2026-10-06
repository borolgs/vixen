use rusqlite::{Connection, OptionalExtension, Row};
use vixen::Page;

use crate::models::{After, MaterialSort};

pub const PAGE_SIZE: usize = 20;

pub struct MaterialQuery {
    pub q: String,
    pub sort: MaterialSort,
    pub after: Option<After>,
}

pub struct Material {
    pub id: i64,
    pub name: String,
    pub care: String,
    pub products: i64,
}

pub struct MaterialInput {
    pub name: String,
    pub care: String,
}

const MATERIAL_COLUMNS: &str = "id, name, care,
    (select count(*) from product_materials pm where pm.material_id = materials.id) as products";

fn material(row: &Row) -> rusqlite::Result<Material> {
    Ok(Material {
        id: row.get(0)?,
        name: row.get(1)?,
        care: row.get(2)?,
        products: row.get(3)?,
    })
}

pub fn search_materials(
    conn: &Connection,
    query: &MaterialQuery,
) -> anyhow::Result<Page<Material, After>> {
    let pattern = query
        .q
        .trim()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");

    // Each sort is one ascending key, so a page continues after the last row's (key, id).
    let (key, after_key) = match query.sort {
        MaterialSort::Name => ("name collate nocase", "?2"),
        MaterialSort::MostUsed => ("-products", "cast(?2 as integer)"),
        MaterialSort::LeastUsed => ("products", "cast(?2 as integer)"),
    };

    let after = query.after.as_ref();

    let mut stmt = conn.prepare(&format!(
        "select id, name, care, products, cast({key} as text) from (
                 select {MATERIAL_COLUMNS} from materials
                 where name like '%' || ?1 || '%' escape '\\'
                    or care like '%' || ?1 || '%' escape '\\')
             where (?3 is null or ({key}, id) > ({after_key}, ?3))
             order by {key}, id
             limit ?4"
    ))?;

    let mut rows = stmt
        .query_map(
            (
                &pattern,
                after.map(|after| &after.key),
                after.map(|after| after.id),
                PAGE_SIZE as i64 + 1,
            ),
            |row| Ok((material(row)?, row.get::<_, String>(4)?)),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let next = if rows.len() > PAGE_SIZE {
        rows.truncate(PAGE_SIZE);
        rows.last().map(|(material, key)| After {
            id: material.id,
            key: key.clone(),
        })
    } else {
        None
    };

    Ok(Page {
        items: rows.into_iter().map(|(material, _)| material).collect(),
        next,
    })
}

pub fn material_options(
    conn: &Connection,
    q: &str,
    after: Option<&After>,
) -> anyhow::Result<Page<Material, After>> {
    let pattern = q
        .trim()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");

    let mut stmt = conn.prepare(&format!(
        "select {MATERIAL_COLUMNS} from materials
             where name like '%' || ?1 || '%' escape '\\'
               and (?3 is null or (name collate nocase, id) > (?2, ?3))
             order by name collate nocase, id
             limit ?4"
    ))?;

    let mut materials = stmt
        .query_map(
            (
                &pattern,
                after.map(|after| &after.key),
                after.map(|after| after.id),
                PAGE_SIZE as i64 + 1,
            ),
            material,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let next = if materials.len() > PAGE_SIZE {
        materials.truncate(PAGE_SIZE);
        materials.last().map(|material| After {
            id: material.id,
            key: material.name.clone(),
        })
    } else {
        None
    };

    Ok(Page {
        items: materials,
        next,
    })
}

pub fn material_by_id(conn: &Connection, id: i64) -> anyhow::Result<Option<Material>> {
    let material = conn
        .query_row(
            &format!("select {MATERIAL_COLUMNS} from materials where id = ?1"),
            (id,),
            material,
        )
        .optional()?;

    Ok(material)
}

/// Whether a material other than `except` already has `name`.
pub fn name_taken(conn: &Connection, name: &str, except: Option<i64>) -> anyhow::Result<bool> {
    let taken = conn
        .query_row(
            "select 1 from materials where name = ?1 collate nocase and id is not ?2",
            (name, except),
            |_| Ok(()),
        )
        .optional()?
        .is_some();

    Ok(taken)
}

pub fn insert_material(conn: &Connection, input: &MaterialInput) -> anyhow::Result<()> {
    conn.execute(
        "insert into materials (name, care) values (?1, ?2)",
        (&input.name, &input.care),
    )?;

    Ok(())
}

pub fn update_material(conn: &Connection, id: i64, input: &MaterialInput) -> anyhow::Result<()> {
    conn.execute(
        "update materials set name = ?2, care = ?3 where id = ?1",
        (id, &input.name, &input.care),
    )?;

    Ok(())
}

pub fn delete_material(conn: &Connection, id: i64) -> anyhow::Result<()> {
    conn.execute("delete from materials where id = ?1", (id,))?;

    Ok(())
}

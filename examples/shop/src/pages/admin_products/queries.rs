use rusqlite::{Connection, OptionalExtension, Row};
use vixen::Page;

use crate::models::{After, Category, ProductSort};

pub const PAGE_SIZE: usize = 10;

pub struct ProductQuery {
    pub category: Option<Category>,
    pub q: String,
    pub sort: ProductSort,
    pub after: Option<After>,
}

pub struct Product {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub category: Category,
    pub price_cents: i64,
    pub in_stock: bool,
    pub materials: String,
}

pub struct ProductInput {
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub category: Category,
    pub price_cents: i64,
    pub in_stock: bool,
}

// The inner ORDER BY makes group_concat deterministic.
const PRODUCT_COLUMNS: &str = "id, slug, name, tagline, category, price_cents, in_stock,
    coalesce((select group_concat(name, ', ') from (
        select m.name from product_materials pm
        join materials m on m.id = pm.material_id
        where pm.product_id = products.id
        order by m.name)), '')";

fn product(row: &Row) -> rusqlite::Result<Product> {
    Ok(Product {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        tagline: row.get(3)?,
        category: row.get(4)?,
        price_cents: row.get(5)?,
        in_stock: row.get(6)?,
        materials: row.get(7)?,
    })
}

pub fn search_products(
    conn: &Connection,
    query: &ProductQuery,
) -> anyhow::Result<Page<Product, After>> {
    let pattern = query
        .q
        .trim()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");

    // Each sort is one ascending key, so a page continues after the last row's (key, id).
    let (key, after_key) = match query.sort {
        ProductSort::Shelf => ("id", "cast(?3 as integer)"),
        ProductSort::PriceAsc => ("price_cents", "cast(?3 as integer)"),
        ProductSort::PriceDesc => ("-price_cents", "cast(?3 as integer)"),
        ProductSort::Name => ("name collate nocase", "?3"),
    };

    let after = query.after.as_ref();

    let mut stmt = conn.prepare(&format!(
        "select {PRODUCT_COLUMNS}, cast({key} as text) from products
             where (?1 is null or category = ?1)
               and (name like '%' || ?2 || '%' escape '\\'
                 or tagline like '%' || ?2 || '%' escape '\\'
                 or slug like '%' || ?2 || '%' escape '\\')
               and (?4 is null or ({key}, id) > ({after_key}, ?4))
             order by {key}, id
             limit ?5"
    ))?;

    let mut rows = stmt
        .query_map(
            (
                query.category,
                &pattern,
                after.map(|after| &after.key),
                after.map(|after| after.id),
                PAGE_SIZE as i64 + 1,
            ),
            |row| Ok((product(row)?, row.get::<_, String>(8)?)),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let next = if rows.len() > PAGE_SIZE {
        rows.truncate(PAGE_SIZE);
        rows.last().map(|(product, key)| After {
            id: product.id,
            key: key.clone(),
        })
    } else {
        None
    };

    Ok(Page {
        items: rows.into_iter().map(|(product, _)| product).collect(),
        next,
    })
}

pub fn product_by_id(conn: &Connection, id: i64) -> anyhow::Result<Option<Product>> {
    let product = conn
        .query_row(
            &format!("select {PRODUCT_COLUMNS} from products where id = ?1"),
            (id,),
            product,
        )
        .optional()?;

    Ok(product)
}

/// Whether a product other than `except` already uses `slug`.
pub fn slug_taken(conn: &Connection, slug: &str, except: Option<i64>) -> anyhow::Result<bool> {
    let taken = conn
        .query_row(
            "select 1 from products where slug = ?1 and id is not ?2",
            (slug, except),
            |_| Ok(()),
        )
        .optional()?
        .is_some();

    Ok(taken)
}

pub fn insert_product(conn: &Connection, input: &ProductInput) -> anyhow::Result<()> {
    conn.execute(
        "insert into products (slug, name, tagline, category, price_cents, in_stock)
             values (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &input.slug,
            &input.name,
            &input.tagline,
            input.category,
            input.price_cents,
            input.in_stock,
        ),
    )?;

    Ok(())
}

pub fn update_product(conn: &Connection, id: i64, input: &ProductInput) -> anyhow::Result<()> {
    conn.execute(
        "update products
             set slug = ?2, name = ?3, tagline = ?4, category = ?5, price_cents = ?6, in_stock = ?7
             where id = ?1",
        (
            id,
            &input.slug,
            &input.name,
            &input.tagline,
            input.category,
            input.price_cents,
            input.in_stock,
        ),
    )?;

    Ok(())
}

pub fn delete_product(conn: &Connection, id: i64) -> anyhow::Result<()> {
    conn.execute("delete from products where id = ?1", (id,))?;

    Ok(())
}

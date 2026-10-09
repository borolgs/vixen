use rusqlite::{Connection, OptionalExtension, Row};
use vixen::{After, Page};

use crate::models::{Category, ProductSort};

pub const PAGE_SIZE: usize = 8;

pub struct ProductQuery {
    pub category: Option<Category>,
    pub q: String,
    pub sort: ProductSort,
    pub after: Option<After<i64>>,
}

pub struct Product {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub category: Category,
    pub price_cents: i64,
    pub in_stock: bool,
}

pub struct Material {
    pub name: String,
    pub care: String,
}

const PRODUCT_COLUMNS: &str = "id, slug, name, tagline, category, price_cents, in_stock";

fn product(row: &Row) -> rusqlite::Result<Product> {
    Ok(Product {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        tagline: row.get(3)?,
        category: row.get(4)?,
        price_cents: row.get(5)?,
        in_stock: row.get(6)?,
    })
}

pub fn search_products(
    conn: &Connection,
    query: &ProductQuery,
) -> anyhow::Result<Page<Product, After<i64>>> {
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
                 or tagline like '%' || ?2 || '%' escape '\\')
               and (?4 is null or ({key}, id) > ({after_key}, ?4))
             order by {key}, id
             limit ?5"
    ))?;

    let rows = stmt
        .query_map(
            (
                query.category,
                &pattern,
                after.map(|after| &after.key),
                after.map(|after| after.id),
                PAGE_SIZE as i64 + 1,
            ),
            |row| Ok((product(row)?, row.get::<_, String>(7)?)),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let page = Page::from_rows(rows, PAGE_SIZE, |(product, key)| After {
        id: product.id,
        key: key.clone(),
    });

    Ok(page.map(|(product, _)| product))
}

pub fn product_by_slug(conn: &Connection, slug: &str) -> anyhow::Result<Option<Product>> {
    let product = conn
        .query_row(
            &format!("select {PRODUCT_COLUMNS} from products where slug = ?1"),
            (slug,),
            product,
        )
        .optional()?;

    Ok(product)
}

pub fn product_materials(conn: &Connection, product_id: i64) -> anyhow::Result<Vec<Material>> {
    let mut stmt = conn.prepare(
        "select name, care from materials
             where id in (select material_id from product_materials where product_id = ?1)
             order by name",
    )?;

    let materials = stmt
        .query_map((product_id,), |row| {
            Ok(Material {
                name: row.get(0)?,
                care: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(materials)
}

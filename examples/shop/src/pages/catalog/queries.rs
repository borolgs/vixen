use rusqlite::{Connection, OptionalExtension, Row};
use vixen::Page;

use crate::{
    models::{Category, ProductSort},
    pages::catalog::search::SearchCatalog,
};

pub const PAGE_SIZE: usize = 8;

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
    search: &SearchCatalog,
) -> anyhow::Result<Page<Product, u32>> {
    let pattern = search
        .q
        .trim()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");

    let order = match search.sort {
        ProductSort::Shelf => "id",
        ProductSort::PriceAsc => "price_cents, id",
        ProductSort::PriceDesc => "price_cents desc, id",
        ProductSort::Name => "name collate nocase, id",
    };

    let offset = search.after.unwrap_or(0);

    let mut stmt = conn.prepare(&format!(
        "select {PRODUCT_COLUMNS} from products
             where (?1 is null or category = ?1)
               and (name like '%' || ?2 || '%' escape '\\'
                 or tagline like '%' || ?2 || '%' escape '\\')
             order by {order}
             limit ?3 offset ?4"
    ))?;

    let mut items = stmt
        .query_map(
            (search.category, &pattern, PAGE_SIZE as i64 + 1, offset),
            product,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let next = if items.len() > PAGE_SIZE {
        items.truncate(PAGE_SIZE);
        Some(offset + PAGE_SIZE as u32)
    } else {
        None
    };

    Ok(Page { items, next })
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

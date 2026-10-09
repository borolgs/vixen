use axum::response::{IntoResponse, Response};
use strum::IntoEnumIterator;
use vixen::{
    After, HxPartialResponse, Page, fragment,
    maud::{Markup, html},
    partial,
    ui::basecoatui::Combobox,
};

use crate::{
    models::{Category, Selection},
    pages::{
        admin_materials::{Material, MaterialPickerId, material_options, material_picker},
        admin_products::{
            queries::{self, Product, ProductInput},
            routes::{CreateProduct, EditProductPath, NewProductPath, UpdateProduct},
            ui::{DRAWER, PRODUCTS, product_row},
        },
        shared::{TOASTER, dollars, parse_dollars},
    },
    state::ctx,
};

pub async fn new_product(_: NewProductPath) -> Response {
    let options = ctx().db.call(|conn| material_options(conn, "", None)).await;

    match options {
        Ok(options) => DRAWER
            .header(html! {
                h2 { "New product" }
                p { "It goes on the shelf as soon as you save." }
            })
            .content(create_product_form(&CreateProduct::blank(), options))
            .into_response(),
        Err(err) => {
            tracing::error!("new product: {err:#}");

            TOASTER
                .error("That didn't go through", "Try again in a moment.")
                .into_response()
        }
    }
}

pub async fn edit_product(EditProductPath { id }: EditProductPath) -> Response {
    let product = ctx()
        .db
        .call(move |conn| {
            let Some(product) = queries::product_by_id(conn, id)? else {
                return Ok(None);
            };
            let materials = queries::product_materials(conn, id)?;
            let options = material_options(conn, "", None)?;

            Ok(Some((product, materials, options)))
        })
        .await;

    match product {
        Ok(Some((product, materials, options))) => DRAWER
            .header(html! {
                h2 { "Edit " (product.name) }
                p { "Changes show up in the shop right away." }
            })
            .content(edit_product_form(
                &UpdateProduct::of(&product, materials),
                options,
            ))
            .into_response(),
        Ok(None) => DRAWER
            .header(html! {
                h2 { "Not found" }
                p { "Someone got here first." }
            })
            .content(html! {
                div.alert data-variant="destructive" {
                    h3 { "No such product" }
                    section { p { "It may have been deleted. Close this and refresh." } }
                }
            })
            .into_response(),
        Err(err) => {
            tracing::error!("edit product {id}: {err:#}");

            TOASTER
                .error("That didn't go through", "Try again in a moment.")
                .into_response()
        }
    }
}

pub async fn create_product(form: CreateProduct) -> HxPartialResponse {
    let input = match form.validate() {
        Ok(input) => input,
        Err(message) => return partial!(product_form_error(message)),
    };

    let created = ctx()
        .db
        .call(move |conn| {
            let tx = conn.transaction()?;

            if queries::slug_taken(&tx, &input.slug, None)? {
                return Ok(Err("That slug is taken."));
            }
            let id = queries::insert_product(&tx, &input)?;
            queries::replace_product_materials(&tx, id, &input.materials)?;

            tx.commit()?;

            Ok(Ok(()))
        })
        .await
        .unwrap_or_else(|err| {
            tracing::error!("create product: {err:#}");

            Err("Internal Server Error")
        });

    match created {
        Ok(()) => partial! {
            DRAWER.close(),
            PRODUCTS.refresh(),
            TOASTER.success("Added", "It is on the shelf now."),
        },
        Err(message) => partial! {
            product_form_error(message)
        },
    }
}

pub async fn update_product(form: UpdateProduct) -> HxPartialResponse {
    let (id, input) = match form.validate() {
        Ok(valid) => valid,
        Err(message) => {
            return partial! {
                product_form_error(message)
            };
        }
    };

    let updated = ctx()
        .db
        .call(move |conn| {
            let tx = conn.transaction()?;

            if queries::slug_taken(&tx, &input.slug, Some(id))? {
                return Ok(Err("That slug is taken."));
            }
            queries::update_product(&tx, id, &input)?;
            queries::replace_product_materials(&tx, id, &input.materials)?;

            let Some(product) = queries::product_by_id(&tx, id)? else {
                return Ok(Err("That product is gone."));
            };

            tx.commit()?;

            Ok(Ok(product))
        })
        .await
        .unwrap_or_else(|err| {
            tracing::error!("update product {id}: {err:#}");

            Err("Internal Server Error")
        });

    match updated {
        Ok(product) => partial! {
            DRAWER.close(),
            product_row(&product),
            TOASTER.success("Saved", "Changes are live in the shop."),
        },
        Err(message) => partial! {
            product_form_error(message)
        },
    }
}

impl CreateProduct {
    fn blank() -> Self {
        Self {
            slug: String::new(),
            name: String::new(),
            tagline: String::new(),
            price: String::new(),
            category: Category::Kitchen,
            materials: Selection::default(),
            in_stock: true,
        }
    }

    fn validate(self) -> Result<ProductInput, &'static str> {
        let name = self.name.trim().to_owned();
        let slug = self.slug.trim().to_owned();

        if name.is_empty() {
            return Err("A product needs a name.");
        }
        if slug.is_empty()
            || !slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("Slugs are lowercase letters, digits and dashes.");
        }
        let Some(price_cents) = parse_dollars(&self.price) else {
            return Err("The price should look like 42.00.");
        };

        let materials = self
            .materials
            .0
            .iter()
            .filter_map(|picked| picked.value.parse().ok())
            .collect();

        Ok(ProductInput {
            slug,
            name,
            tagline: self.tagline.trim().to_owned(),
            category: self.category,
            price_cents,
            in_stock: self.in_stock,
            materials,
        })
    }
}

impl UpdateProduct {
    fn of(product: &Product, materials: Selection) -> Self {
        Self {
            id: product.id,
            slug: product.slug.clone(),
            name: product.name.clone(),
            tagline: product.tagline.clone(),
            price: dollars(product.price_cents),
            category: Some(product.category),
            materials,
            in_stock: product.in_stock,
        }
    }

    fn validate(self) -> Result<(i64, ProductInput), &'static str> {
        let name = self.name.trim().to_owned();
        let slug = self.slug.trim().to_owned();

        if name.is_empty() {
            return Err("A product needs a name.");
        }
        if slug.is_empty()
            || !slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("Slugs are lowercase letters, digits and dashes.");
        }
        let Some(price_cents) = parse_dollars(&self.price) else {
            return Err("The price should look like 42.00.");
        };
        let Some(category) = self.category else {
            return Err("Pick a category.");
        };

        let materials = self
            .materials
            .0
            .iter()
            .filter_map(|picked| picked.value.parse().ok())
            .collect();

        Ok((
            self.id,
            ProductInput {
                slug,
                name,
                tagline: self.tagline.trim().to_owned(),
                category,
                price_cents,
                in_stock: self.in_stock,
                materials,
            },
        ))
    }
}

fn create_product_form(form: &CreateProduct, options: Page<Material, After<i64>>) -> Markup {
    html! {
        form class="fieldset gap-4" hx-action=(CreateProduct::action()) {
            div.field {
                label for="product-name" { "Name" }
                input id="product-name" type="text" name=(CreateProduct::FIELD.name)
                    value=(form.name) placeholder="Stovetop Kettle";
            }
            div.field {
                label for="product-slug" { "Slug" }
                input id="product-slug" type="text" name=(CreateProduct::FIELD.slug)
                    value=(form.slug) placeholder="kettle";
            }
            div.field {
                label for="product-tagline" { "Tagline" }
                textarea id="product-tagline" rows="3" name=(CreateProduct::FIELD.tagline)
                    placeholder="One sentence for the card." { (form.tagline) }
            }
            div.field {
                label for="product-price" { "Price" }
                input id="product-price" type="text" inputmode="decimal"
                    name=(CreateProduct::FIELD.price) value=(form.price) placeholder="42.00";
            }
            div.field {
                label for="product-category" { "Category" }
                select id="product-category" class="w-full" name=(CreateProduct::FIELD.category) {
                    @for category in Category::iter() {
                        option value=(category.as_ref()) selected[category == form.category] {
                            (category.label())
                        }
                    }
                }
            }
            div.field {
                label for=(MaterialPickerId) { "Materials" }
                (material_picker(CreateProduct::FIELD.materials, &form.materials, options))
            }
            div.field data-orientation="horizontal" {
                input id="product-in-stock" type="checkbox"
                    name=(CreateProduct::FIELD.in_stock) value="true" checked[form.in_stock];
                label for="product-in-stock" { "In stock" }
            }

            (product_form_error::slot())

            div class="flex gap-2" {
                button.btn type="submit" { "Save" }
                button.btn data-variant="outline" type="button"
                    onclick="this.closest('dialog').close()" { "Cancel" }
            }
        }
    }
}

fn edit_product_form(form: &UpdateProduct, options: Page<Material, After<i64>>) -> Markup {
    html! {
        form class="fieldset gap-4" hx-action=(UpdateProduct::action().id(form.id)) {
            div.field {
                label for="product-name" { "Name" }
                input id="product-name" type="text" name=(UpdateProduct::FIELD.name)
                    value=(form.name) placeholder="Stovetop Kettle";
            }
            div.field {
                label for="product-slug" { "Slug" }
                input id="product-slug" type="text" name=(UpdateProduct::FIELD.slug)
                    value=(form.slug) placeholder="kettle";
            }
            div.field {
                label for="product-tagline" { "Tagline" }
                textarea id="product-tagline" rows="3" name=(UpdateProduct::FIELD.tagline)
                    placeholder="One sentence for the card." { (form.tagline) }
            }
            div.field {
                label for="product-price" { "Price" }
                input id="product-price" type="text" inputmode="decimal"
                    name=(UpdateProduct::FIELD.price) value=(form.price) placeholder="42.00";
            }
            div.field {
                label for="product-category" { "Category" }
                (Combobox::new("product-category", UpdateProduct::FIELD.category)
                    .value(form.category.as_ref().map_or("", |category| category.as_ref()))
                    .class("w-full")
                    .options(html! {
                        @for category in Category::iter() {
                            div role="option" data-value=(category.as_ref()) { (category.label()) }
                        }
                    }))
            }
            div.field {
                label for=(MaterialPickerId) { "Materials" }
                (material_picker(UpdateProduct::FIELD.materials, &form.materials, options))
            }
            div.field data-orientation="horizontal" {
                input id="product-in-stock" type="checkbox"
                    name=(UpdateProduct::FIELD.in_stock) value="true" checked[form.in_stock];
                label for="product-in-stock" { "In stock" }
            }

            (product_form_error::slot())

            div class="flex gap-2" {
                button.btn type="submit" { "Save" }
                button.btn data-variant="outline" type="button"
                    onclick="this.closest('dialog').close()" { "Cancel" }
            }
        }
    }
}

#[fragment]
fn product_form_error(message: &str) -> Markup {
    html! {
        div.alert id=(Self) data-variant="destructive" {
            h3 { "That didn't go through" }
            section { p { (message) } }
        }
    }
}

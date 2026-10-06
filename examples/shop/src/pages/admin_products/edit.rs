use axum::response::{IntoResponse, Response};
use strum::IntoEnumIterator;
use vixen::{
    fragment,
    hx::HxResponseTrigger,
    maud::{Markup, html},
    partial,
    ui::basecoatui::Slots,
};

use crate::{
    models::Category,
    pages::{
        admin_products::{
            queries::{self, Product, ProductInput},
            routes::{CreateProduct, EditProductPath, NewProductPath, UpdateProduct},
            ui::{DRAWER, PRODUCTS, product_row},
        },
        shared::{TOASTER, dollars, parse_dollars},
    },
    state::ctx,
};

pub async fn new_product(_: NewProductPath) -> Slots {
    DRAWER
        .header(html! {
            h2 { "New product" }
            p { "It goes on the shelf as soon as you save." }
        })
        .content(create_product_form(&CreateProduct::blank()))
}

pub async fn edit_product(EditProductPath { id }: EditProductPath) -> Response {
    let product = ctx()
        .db
        .call(move |conn| queries::product_by_id(conn, id))
        .await;

    match product {
        Ok(Some(product)) => DRAWER
            .header(html! {
                h2 { "Edit " (product.name) }
                p { "Changes show up in the shop right away." }
            })
            .content(edit_product_form(&UpdateProduct::of(&product)))
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

pub async fn create_product(form: CreateProduct) -> Response {
    let input = match form.validate() {
        Ok(input) => input,
        Err(message) => return partial!(product_form_error(message)).into_response(),
    };

    let created = ctx()
        .db
        .call(move |conn| {
            let tx = conn.transaction()?;

            if queries::slug_taken(&tx, &input.slug, None)? {
                return Ok(Err("That slug is taken."));
            }
            queries::insert_product(&tx, &input)?;

            tx.commit()?;

            Ok(Ok(()))
        })
        .await
        .unwrap_or_else(|err| {
            tracing::error!("create product: {err:#}");

            Err("Internal Server Error")
        });

    match created {
        Ok(()) => (
            HxResponseTrigger::normal([DRAWER.close(), PRODUCTS.refresh()]),
            TOASTER.success("Added", "It is on the shelf now."),
        )
            .into_response(),
        Err(message) => partial!(product_form_error(message)).into_response(),
    }
}

pub async fn update_product(form: UpdateProduct) -> Response {
    let (id, input) = match form.validate() {
        Ok(valid) => valid,
        Err(message) => return partial!(product_form_error(message)).into_response(),
    };

    let updated = ctx()
        .db
        .call(move |conn| {
            let tx = conn.transaction()?;

            if queries::slug_taken(&tx, &input.slug, Some(id))? {
                return Ok(Err("That slug is taken."));
            }
            queries::update_product(&tx, id, &input)?;

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
        Ok(product) => (
            HxResponseTrigger::normal([DRAWER.close()]),
            partial!(
                product_row(&product),
                TOASTER.success("Saved", "Changes are live in the shop."),
            ),
        )
            .into_response(),
        Err(message) => partial!(product_form_error(message)).into_response(),
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

        Ok(ProductInput {
            slug,
            name,
            tagline: self.tagline.trim().to_owned(),
            category: self.category,
            price_cents,
            in_stock: self.in_stock,
        })
    }
}

impl UpdateProduct {
    fn of(product: &Product) -> Self {
        Self {
            id: product.id,
            slug: product.slug.clone(),
            name: product.name.clone(),
            tagline: product.tagline.clone(),
            price: dollars(product.price_cents),
            category: product.category,
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

        Ok((
            self.id,
            ProductInput {
                slug,
                name,
                tagline: self.tagline.trim().to_owned(),
                category: self.category,
                price_cents,
                in_stock: self.in_stock,
            },
        ))
    }
}

fn create_product_form(form: &CreateProduct) -> Markup {
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

fn edit_product_form(form: &UpdateProduct) -> Markup {
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
                select id="product-category" class="w-full" name=(UpdateProduct::FIELD.category) {
                    @for category in Category::iter() {
                        option value=(category.as_ref()) selected[category == form.category] {
                            (category.label())
                        }
                    }
                }
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

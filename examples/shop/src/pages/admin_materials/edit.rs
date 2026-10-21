use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use vixen::{
    HxPartialResponse, fragment,
    maud::{Markup, html},
    partial,
    ui::basecoatui::{Drawer, Slots},
};

use crate::{
    pages::{
        admin_materials::{
            queries::{self, Material, MaterialInput},
            routes::{CreateMaterial, EditMaterialPath, NewMaterialPath, UpdateMaterial},
            ui::{DRAWER, MATERIALS, material_row},
        },
        shared::TOASTER,
    },
    state::ctx,
};

pub async fn new_material(_: NewMaterialPath) -> Slots {
    DRAWER
        .header(html! {
            h2 { "New material" }
            p { "Products can pick it as soon as you save." }
        })
        .content(create_material_form(&CreateMaterial::blank()))
}

pub async fn edit_material(EditMaterialPath { id }: EditMaterialPath) -> Response {
    let material = ctx()
        .db
        .call(move |conn| queries::material_by_id(conn, id))
        .await;

    match material {
        Ok(Some(material)) => DRAWER
            .header(html! {
                h2 { "Edit " (material.name) }
                p { "Every product made of it follows." }
            })
            .content(edit_material_form(&UpdateMaterial::of(&material)))
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            DRAWER
                .header(html! {
                    h2 { "Not found" }
                    p { "Someone got here first." }
                })
                .content(html! {
                    div.alert data-variant="destructive" {
                        h3 { "No such material" }
                        section { p { "It may have been deleted. Close this and refresh." } }
                    }
                }),
        )
            .into_response(),
        Err(err) => {
            tracing::error!("edit material {id}: {err:#}");

            partial! {
                StatusCode::INTERNAL_SERVER_ERROR,
                TOASTER.error("That didn't go through", "Try again in a moment."),
            }
            .into_response()
        }
    }
}

pub async fn create_material(form: CreateMaterial) -> HxPartialResponse {
    let input = match form.validate() {
        Ok(input) => input,
        Err(message) => {
            return partial! {
                StatusCode::UNPROCESSABLE_ENTITY,
                material_form_error(message),
            };
        }
    };

    let created = ctx()
        .db
        .call(move |conn| {
            let tx = conn.transaction()?;

            if queries::name_taken(&tx, &input.name, None)? {
                return Ok(Err("That name is taken."));
            }
            queries::insert_material(&tx, &input)?;

            tx.commit()?;

            Ok(Ok(()))
        })
        .await;

    match created {
        Ok(Ok(())) => partial! {
            DRAWER.close(),
            MATERIALS.refresh(),
            TOASTER.success("Added", "Products can pick it now."),
        },
        Ok(Err(message)) => partial! {
            StatusCode::UNPROCESSABLE_ENTITY,
            material_form_error(message),
        },
        Err(err) => {
            tracing::error!("create material: {err:#}");

            partial! {
                StatusCode::INTERNAL_SERVER_ERROR,
                material_form_error("Internal Server Error"),
            }
        }
    }
}

pub async fn update_material(form: UpdateMaterial) -> HxPartialResponse {
    let (id, input) = match form.validate() {
        Ok(valid) => valid,
        Err(message) => {
            return partial! {
                StatusCode::UNPROCESSABLE_ENTITY,
                material_form_error(message),
            };
        }
    };

    let updated = ctx()
        .db
        .call(move |conn| {
            let tx = conn.transaction()?;

            if queries::name_taken(&tx, &input.name, Some(id))? {
                return Ok(Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "That name is taken.",
                )));
            }
            queries::update_material(&tx, id, &input)?;

            let Some(material) = queries::material_by_id(&tx, id)? else {
                return Ok(Err((StatusCode::NOT_FOUND, "That material is gone.")));
            };

            tx.commit()?;

            Ok(Ok(material))
        })
        .await;

    match updated {
        Ok(Ok(material)) => partial! {
            DRAWER.close(),
            material_row(&material),
            TOASTER.success("Saved", "Every product made of it follows."),
        },
        Ok(Err((status, message))) => partial! {
            status,
            material_form_error(message),
        },
        Err(err) => {
            tracing::error!("update material {id}: {err:#}");

            partial! {
                StatusCode::INTERNAL_SERVER_ERROR,
                material_form_error("Internal Server Error"),
            }
        }
    }
}

impl CreateMaterial {
    fn blank() -> Self {
        Self {
            name: String::new(),
            care: String::new(),
        }
    }

    fn validate(self) -> Result<MaterialInput, &'static str> {
        let name = self.name.trim().to_owned();

        if name.is_empty() {
            return Err("A material needs a name.");
        }

        Ok(MaterialInput {
            name,
            care: self.care.trim().to_owned(),
        })
    }
}

impl UpdateMaterial {
    fn of(material: &Material) -> Self {
        Self {
            id: material.id,
            name: material.name.clone(),
            care: material.care.clone(),
        }
    }

    fn validate(self) -> Result<(i64, MaterialInput), &'static str> {
        let name = self.name.trim().to_owned();

        if name.is_empty() {
            return Err("A material needs a name.");
        }

        Ok((
            self.id,
            MaterialInput {
                name,
                care: self.care.trim().to_owned(),
            },
        ))
    }
}

fn create_material_form(form: &CreateMaterial) -> Markup {
    html! {
        form class="fieldset gap-4" hx-action=(CreateMaterial::action()) {
            div.field {
                label for="material-name" { "Name" }
                input id="material-name" type="text" name=(CreateMaterial::FIELD.name)
                    value=(form.name) placeholder="Cast iron";
            }
            div.field {
                label for="material-care" { "Care" }
                textarea id="material-care" rows="2" name=(CreateMaterial::FIELD.care)
                    placeholder="Season with oil; never soak." { (form.care) }
            }

            (material_form_error::slot())

            div class="flex gap-2" {
                button.btn type="submit" { "Save" }
                button.btn data-variant="outline" type="button"
                    onclick=(Drawer::cancel()) { "Cancel" }
            }
        }
    }
}

fn edit_material_form(form: &UpdateMaterial) -> Markup {
    html! {
        form class="fieldset gap-4" hx-action=(UpdateMaterial::action().id(form.id)) {
            div.field {
                label for="material-name" { "Name" }
                input id="material-name" type="text" name=(UpdateMaterial::FIELD.name)
                    value=(form.name) placeholder="Cast iron";
            }
            div.field {
                label for="material-care" { "Care" }
                textarea id="material-care" rows="2" name=(UpdateMaterial::FIELD.care)
                    placeholder="Season with oil; never soak." { (form.care) }
            }

            (material_form_error::slot())

            div class="flex gap-2" {
                button.btn type="submit" { "Save" }
                button.btn data-variant="outline" type="button"
                    onclick=(Drawer::cancel()) { "Cancel" }
            }
        }
    }
}

#[fragment]
fn material_form_error(message: &str) -> Markup {
    html! {
        div.alert id=(Self) role="alert" data-variant="destructive" {
            h3 { "That didn't go through" }
            section { p { (message) } }
        }
    }
}

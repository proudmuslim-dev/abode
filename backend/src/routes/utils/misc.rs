// Needed because of the default attrs on FromForm
#![allow(clippy::needless_late_init)]

use image::{io::Reader as ImageReader, ImageError, ImageFormat, ImageOutputFormat};
use imagesize::ImageSize;
use rocket::{
    data::ToByteUnit,
    form::{DataField, Form, FromFormField, Strict, ValueField},
    http::ContentType,
};
use sanitizer::Sanitize;
use std::{io::Cursor, ops::Deref, str::FromStr};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Copy, Clone, Deserialize)]
pub struct UuidField(pub(crate) Uuid);

impl Deref for UuidField {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: UUID from Username
impl<'v> FromFormField<'v> for UuidField {
    fn from_value(field: ValueField<'v>) -> rocket::form::Result<'v, Self> {
        let val = field
            .value
            .chars()
            .filter(|c| !c.is_whitespace() && c.is_ascii())
            .collect::<String>();

        let id = Uuid::from_str(val.as_str()).map_err(|_| rocket::form::Error::validation("invalid uuid"))?;

        Ok(UuidField(id))
    }
}

#[derive(FromForm, Copy, Clone)]
pub struct PaginationFields {
    #[field(default = 1)]
    pub page: u32,
    #[field(default = 10)]
    pub per_page: u32,
}

impl PaginationFields {
    pub fn skip(&self) -> i64 {
        if self.page == 1 {
            0
        } else {
            let ret = (self.page - 1) * self.per_page;

            ret.into()
        }
    }
}

pub struct ImageField {
    pub(crate) width: usize,
    pub(crate) height: usize,
    bytes: Vec<u8>,
    /// Can only be PNG or JPEG
    format: ImageFormat,
}

impl ImageField {
    pub fn persist(&self) -> std::io::Result<String> {
        let mut path = {
            let id = Uuid::new_v4();
            let ext = self.format.extensions_str()[0];

            format!("assets/images/{id}.{ext}")
        };

        let img = image::load_from_memory_with_format(&self.bytes, self.format)
            .expect("Can't fail because it's only instantiated by the from_data fn");

        match img.save(path.as_str()) {
            Ok(_) => {
                path.insert(0, '/');

                Ok(path)
            },
            Err(ImageError::IoError(e)) => Err(e),
            _ => unimplemented!("There's really no reason that this should be reached, and if it is that means there's a bug in the FromFormField impl"),
        }
    }
}

#[async_trait]
impl<'r> FromFormField<'r> for ImageField {
    fn from_value(_: ValueField<'r>) -> rocket::form::Result<'r, Self> {
        Err(rocket::form::Error::validation("Missing Content-Type on image field!"))?
    }

    async fn from_data(field: DataField<'r, '_>) -> rocket::form::Result<'r, Self> {
        let internal_server_error = Err(rocket::form::Error::validation("Internal server error"));

        let limit = field.request.limits().get("image").unwrap_or(4.mebibytes());

        let req_ct = field.content_type;
        let (png_ct, jpeg_ct) = (ContentType::new("image", "png"), ContentType::new("image", "jpeg"));

        if req_ct != png_ct && req_ct != jpeg_ct {
            return Err(rocket::form::Error::validation("Not a PNG or JPEG image"))?;
        }

        let mut bytes = match field.data.open(limit).into_bytes().await {
            Ok(bytes) if bytes.is_complete() => bytes.into_inner(),
            Ok(_) => return Err((None, Some(limit)))?,
            // TODO: Figure out how to make this thing actually return what I want it to
            Err(_) => return internal_server_error?,
        };

        if let Ok(ImageSize { width, height }) = imagesize::blob_size(&bytes) {
            let format;

            if req_ct == jpeg_ct {
                let mut cursor = Cursor::new(bytes);
                let mut cursor_2 = Cursor::new(vec![]);

                let rotation = {
                    let exif_reader = exif::Reader::new();

                    match exif_reader.read_from_container(&mut cursor) {
                        Ok(exif) => match exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
                            Some(orientation) => match orientation.value.get_uint(0) {
                                Some(v @ 1..=8) => v,
                                _ => 0,
                            },
                            _ => 0,
                        },
                        _ => 0,
                    }
                };

                cursor.set_position(0);

                // Encoding jpeg again to erase exif
                let img = if let Ok(r) = ImageReader::new(cursor).with_guessed_format() {
                    if let Ok(i) = r.decode() {
                        i
                    } else {
                        return internal_server_error?;
                    }
                } else {
                    return internal_server_error?;
                };

                let result = match &rotation {
                    2 => img.fliph(),
                    3 => img.rotate180(),
                    4 => img.rotate180().fliph(),
                    5 => img.rotate90().fliph(),
                    6 => img.rotate90(),
                    7 => img.rotate270().fliph(),
                    8 => img.rotate270(),
                    _ => img,
                }
                .write_to(&mut cursor_2, ImageOutputFormat::Jpeg(100));

                if result.is_err() {
                    return internal_server_error?;
                }

                bytes = cursor_2.into_inner();
                format = ImageFormat::Jpeg;
            } else {
                format = ImageFormat::Png;
            }

            Ok(ImageField {
                width,
                height,
                bytes,
                format,
            })
        } else {
            Err(rocket::form::Error::validation("Bad image"))?
        }
    }
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    let mut username = username.to_owned();

    username.retain(|c| !c.is_whitespace() && c.is_ascii());

    // Don't waste time if it's a junk req
    if username.is_empty() {
        return Err(ValidationError::new("Invalid username"));
    }

    Ok(())
}

pub fn sanitize_and_validate<T>(form: Form<Strict<T>>) -> Option<T>
where
    T: Validate + Sanitize,
{
    let mut form = form.into_inner().into_inner();

    form.validate().ok()?;
    form.sanitize();

    Some(form)
}

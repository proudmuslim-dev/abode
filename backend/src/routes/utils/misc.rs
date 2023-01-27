// Needed because of the default attrs on FromForm
#![allow(clippy::needless_late_init)]

use image::{imageops::FilterType, io::Reader as ImageReader, ImageFormat, ImageOutputFormat};
use imagesize::ImageSize;
use rocket::{
    data::{FromData, Outcome, ToByteUnit},
    form::{Form, FromFormField, Strict, ValueField},
    fs::TempFile,
    http::{ContentType, Status},
    Data, Either, Request,
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

pub struct ImageGuard {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) bytes: Vec<u8>,
}

#[async_trait]
impl<'r> FromData<'r> for ImageGuard {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let (png_ct, jpeg_ct) = (ContentType::new("image", "png"), ContentType::new("image", "jpeg"));
        let req_ct = req.content_type();

        if req_ct != Some(&png_ct) && req_ct != Some(&jpeg_ct) {
            return Outcome::Forward(data);
        }

        let mut bytes = match data.open(4.mebibytes()).into_bytes().await {
            Ok(bytes) if bytes.is_complete() => bytes.into_inner(),
            Ok(_) => return Outcome::Failure((Status::PayloadTooLarge, ())),
            Err(_) => return Outcome::Failure((Status::InternalServerError, ())),
        };

        if let Ok(ImageSize { width, height }) = imagesize::blob_size(&bytes) {
            if req_ct == Some(&jpeg_ct) {
                let mut cursor = Cursor::new(bytes);
                let mut cursor_2 = Cursor::new(vec![]);
                let exif_reader = exif::Reader::new();

                let rotation = match exif_reader.read_from_container(&mut cursor) {
                    Ok(exif) => match exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
                        Some(orientation) => match orientation.value.get_uint(0) {
                            Some(v @ 1..=8) => v,
                            _ => 0,
                        },
                        _ => 0,
                    },
                    _ => 0,
                };

                cursor.set_position(0);

                // Encoding jpeg again to erase exif
                let img = if let Ok(r) = ImageReader::new(cursor).with_guessed_format() {
                    if let Ok(i) = r.decode() {
                        i
                    } else {
                        return Outcome::Failure((Status::InternalServerError, ()));
                    }
                } else {
                    return Outcome::Failure((Status::InternalServerError, ()));
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
                    return Outcome::Failure((Status::InternalServerError, ()));
                }

                bytes = cursor_2.into_inner();
            }

            Outcome::Success(ImageGuard { width, height, bytes })
        } else {
            Outcome::Failure((Status::BadRequest, ()))
        }
    }
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    let mut username = username.to_owned();

    username.retain(|c| !c.is_whitespace());

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

// TODO: Add images to db in route
pub fn handle_image(file: &TempFile<'_>) -> Result<String, Status> {
    match file {
        TempFile::File { path, content_type, .. } => {
            if content_type.is_none() {
                dbg!("No img type.");
                return Err(Status::BadRequest);
            }

            let id = Uuid::new_v4();

            let content_type = content_type.clone().expect("Won't fail, checked above");

            let format = if content_type.is_png() {
                ImageFormat::Png
            } else if content_type.is_jpeg() {
                ImageFormat::Jpeg
            } else {
                dbg!("Bad img type.");
                return Err(Status::BadRequest);
            };

            let mut reader = match path {
                Either::Left(temp) => ImageReader::open(temp),
                Either::Right(p_buf) => ImageReader::open(p_buf),
            }
            .map_err(|_| Status::InternalServerError)?;

            reader.set_format(format);

            let image = reader.decode().map_err(|_| {
                dbg!("Failed to decode");
                Status::InternalServerError
            })?;

            // 795x1025
            let new_image = image.resize(795, 1025, FilterType::Lanczos3);

            let path = format!("assets/images/{id}.jpeg");

            new_image
                .save_with_format(path.as_str(), ImageFormat::Jpeg)
                .map_err(|_| Status::InternalServerError)?;

            Ok(path.replace("assets/", ""))
        }
        TempFile::Buffered { .. } => Err(Status::BadRequest),
    }
}

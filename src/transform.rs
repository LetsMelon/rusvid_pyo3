use std::num::ParseIntError;

use rusvid_core::pixel::Pixel;

use crate::error::{CustomError, CustomResult};
use crate::read::RawCommand;
use crate::{Command, CustomImage, ImageFill};

fn transform_raw_pixel(raw: Vec<Result<u8, ParseIntError>>) -> CustomResult<Pixel> {
    const ALPHA_DEFAULT: u8 = 255;

    let get_value_from_raw = |index| -> CustomResult<u8> {
        match raw.get(index) {
            Some(value) => match value {
                Ok(value) => Ok(*value),
                Err(err) => {
                    // TODO WHY?!
                    let err: &ParseIntError = err;
                    return Err(CustomError::ParseNumberValue(err.clone()));
                }
            },
            None => Ok(ALPHA_DEFAULT),
        }
    };

    if !(raw.len() == 3 || raw.len() == 4) {
        return Err(CustomError::Parsing(format!(
            "A raw pixel must have 3 or 4 values, but got: {:?} with length {}.",
            raw,
            raw.len()
        )));
    }

    let a = get_value_from_raw(0)?;
    let r = get_value_from_raw(1)?;
    let g = get_value_from_raw(2)?;
    let b = get_value_from_raw(3)?;

    Ok(Pixel::new(r, g, b, a))
}

fn transform_coordinate(
    raw: (Result<u32, ParseIntError>, Result<u32, ParseIntError>),
) -> CustomResult<(u32, u32)> {
    match raw {
        (Ok(a), Ok(b)) => Ok((a, b)),
        (_, Err(err)) | (Err(err), _) => Err(CustomError::from(err)),
    }
}

pub fn transform_raw(
    raw_width: Result<u32, ParseIntError>,
    raw_height: Result<u32, ParseIntError>,
    raw_background: Vec<Result<u8, ParseIntError>>,
    raw_commands: Vec<RawCommand>,
) -> CustomResult<CustomImage> {
    let width = raw_width?;
    let height = raw_height?;

    let background = transform_raw_pixel(raw_background)?;

    let mut commands = Vec::with_capacity(raw_commands.len());
    for command in raw_commands {
        let command = match command {
            RawCommand::DrawPixel { position, color } => {
                let color = transform_raw_pixel(color)?;
                let position = transform_coordinate(position)?;

                Command::DrawPixel { position, color }
            }
            RawCommand::DrawRect {
                corner_position_1,
                corner_position_2,
                color,
            } => {
                let color = transform_raw_pixel(color)?;
                let corner_position_1 = transform_coordinate(corner_position_1)?;
                let corner_position_2 = transform_coordinate(corner_position_2)?;

                Command::DrawRect {
                    corner_position_1,
                    corner_position_2,
                    color,
                }
            }
        };
        commands.push(command);
    }

    Ok(CustomImage {
        width,
        height,
        data: ImageFill::Sparse((commands, background)),
    })
}

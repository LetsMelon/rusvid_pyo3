use std::fs::read_to_string;
use std::path::PathBuf;

use parser::parse_file;
use pyo3::prelude::*;
use rusvid_core::pixel::Pixel;
use rusvid_core::plane::Plane;

pub mod parser;

#[derive(Debug)]
pub enum ImageFill {
    Sparse((Vec<Command>, Pixel)),
}

#[derive(Debug)]
pub enum Command {
    Pixel {
        position: (u32, u32),
        color: Pixel,
    },
    Rect {
        corner_position_1: (u32, u32),
        corner_position_2: (u32, u32),
        color: Pixel,
    },
}

#[derive(Debug)]
#[pyclass]
pub struct CustomImage {
    pub width: u32,
    pub height: u32,

    pub data: ImageFill,
}

#[pymethods]
impl CustomImage {
    #[new]
    pub fn new(path: PathBuf) -> Self {
        let content = read_to_string(path).unwrap();

        parse_file(&content).unwrap().1
    }

    pub fn save(&self, path: PathBuf) {
        let mut plane = match &self.data {
            ImageFill::Sparse((_, color)) => {
                Plane::new_with_fill(self.width, self.height, *color).unwrap()
            }
        };

        match &self.data {
            ImageFill::Sparse((commands, _)) => {
                for command in commands {
                    match command {
                        Command::Pixel { position, color } => {
                            plane.put_pixel(position.0, position.1, *color).unwrap()
                        }
                        Command::Rect {
                            corner_position_1,
                            corner_position_2,
                            color,
                        } => {
                            for x in corner_position_1.0.min(corner_position_2.0)
                                ..=corner_position_1.0.max(corner_position_2.0)
                            {
                                for y in corner_position_1.1.min(corner_position_2.1)
                                    ..=corner_position_1.1.max(corner_position_2.1)
                                {
                                    plane.put_pixel(x, y, *color).unwrap();
                                }
                            }
                        }
                    };
                }
            }
        };

        plane.save_as_png(path).unwrap();
    }

    fn __repr__(&self) -> String {
        format!(
            "CustomImage(width: {}, height: {}, fill: {:?}",
            self.width, self.height, self.data
        )
    }

    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule]
#[pyo3(name = "python_ffi")]
fn my_extension(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<CustomImage>()?;

    m.add_function(wrap_pyfunction!(double, m)?)?;
    Ok(())
}

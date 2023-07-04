use std::num::ParseIntError;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, newline, space1};
use nom::character::streaming::multispace0;
use nom::combinator::map;
use nom::multi::{many1, separated_list0};
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

#[derive(Debug)]
pub enum RawCommand {
    DrawPixel {
        position: (Result<u32, ParseIntError>, Result<u32, ParseIntError>),
        color: Vec<Result<u8, ParseIntError>>,
    },
    DrawRect {
        corner_position_1: (Result<u32, ParseIntError>, Result<u32, ParseIntError>),
        corner_position_2: (Result<u32, ParseIntError>, Result<u32, ParseIntError>),
        color: Vec<Result<u8, ParseIntError>>,
    },
}

fn generic_delimited<'a, F: FnMut(&'a str) -> IResult<&'a str, T>, T>(
    fct: F,
    opening_bracket: char,
    closing_bracket: char,
) -> impl Fn(&'a str) -> IResult<&'a str, T> {
    // hack so that generic F don't have to have the bound 'Copy'
    let fct = std::rc::Rc::new(std::cell::RefCell::new(fct));

    move |input| {
        delimited(
            tag(opening_bracket.to_string().as_str()),
            |input| fct.borrow_mut()(input),
            tag(closing_bracket.to_string().as_str()),
        )(input)
    }
}

fn generic_bracket_content<'a, F: FnMut(&'a str) -> IResult<&'a str, T>, T>(
    fct: F,
) -> impl Fn(&'a str) -> IResult<&'a str, Vec<T>> {
    // hack so that generic F don't have to have the bound 'Copy'
    let fct = std::rc::Rc::new(std::cell::RefCell::new(fct));

    move |input| {
        separated_list0(tuple((multispace0, tag(","), multispace0)), |input| {
            fct.borrow_mut()(input)
        })(input)
    }
}

fn parse_pixel_values(input: &str) -> IResult<&str, Vec<Result<u8, ParseIntError>>> {
    let (input, raw_data) = generic_delimited(
        generic_bracket_content(map(digit1, |raw: &str| raw.parse())),
        '[',
        ']',
    )(input)?;

    Ok((input, raw_data))
}

fn parse_pixel_coordinates(
    input: &str,
) -> IResult<&str, (Result<u32, ParseIntError>, Result<u32, ParseIntError>)> {
    let (input, raw_values) = generic_delimited(
        generic_bracket_content(map(digit1, |raw: &str| raw.parse())),
        '(',
        ')',
    )(input)?;

    assert_eq!(raw_values.len(), 2);

    Ok((input, (raw_values[0].clone(), raw_values[1].clone())))
}

pub fn read_content(
    input: &str,
) -> IResult<
    &str,
    (
        Result<u32, ParseIntError>,
        Result<u32, ParseIntError>,
        Vec<RawCommand>,
        Vec<Result<u8, ParseIntError>>,
    ),
> {
    let (input, width) = preceded(
        tag("width"),
        preceded(space1, map(digit1, |raw: &str| raw.parse())),
    )(input)?;

    let (input, _) = many1(newline)(input)?;

    let (input, height) = preceded(
        tag("height"),
        preceded(space1, map(digit1, |raw: &str| raw.parse())),
    )(input)?;

    let (input, _) = many1(newline)(input)?;

    // TODO implement types
    // let (input, fill) = preceded(
    //     pair(tag("type"), space1),
    //     value(ImageFill::Sparse, tag("sparse")),
    // )(input)?;
    //
    // let (input, _) = newline(input)?;

    let (input, background) =
        preceded(tag("background"), preceded(space1, parse_pixel_values))(input)?;

    let (input, _) = many1(newline)(input)?;

    let (input, commands) = separated_list0(
        newline,
        alt((
            map(
                tuple((
                    tag("pixel"),
                    space1,
                    parse_pixel_coordinates,
                    space1,
                    parse_pixel_values,
                )),
                |(_, _, position, _, color): (
                    _,
                    _,
                    (Result<u32, ParseIntError>, Result<u32, ParseIntError>),
                    _,
                    Vec<Result<u8, ParseIntError>>,
                )| RawCommand::DrawPixel { position, color },
            ),
            map(
                tuple((
                    tag("rect"),
                    space1,
                    parse_pixel_coordinates,
                    space1,
                    parse_pixel_coordinates,
                    space1,
                    parse_pixel_values,
                )),
                |(_, _, corner_position_1, _, corner_position_2, _, color): (
                    _,
                    _,
                    (Result<u32, ParseIntError>, Result<u32, ParseIntError>),
                    _,
                    (Result<u32, ParseIntError>, Result<u32, ParseIntError>),
                    _,
                    Vec<Result<u8, ParseIntError>>,
                )| RawCommand::DrawRect {
                    corner_position_1,
                    corner_position_2,
                    color,
                },
            ),
        )),
    )(input)?;

    Ok((input, (width, height, commands, background)))
}

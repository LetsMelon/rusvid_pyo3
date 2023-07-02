use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, newline, space1};
use nom::character::streaming::multispace0;
use nom::combinator::map;
use nom::multi::{many1, separated_list0};
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

use crate::{Command, CustomImage, ImageFill};

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

pub fn parse_file(input: &str) -> IResult<&str, CustomImage> {
    let (input, width) = preceded(
        tag("width"),
        preceded(space1, map(digit1, |raw: &str| raw.parse().unwrap())),
    )(input)?;

    let (input, _) = newline(input)?;

    let (input, height) = preceded(
        tag("height"),
        preceded(space1, map(digit1, |raw: &str| raw.parse().unwrap())),
    )(input)?;

    let (input, _) = newline(input)?;

    // TODO implement types
    // let (input, fill) = preceded(
    //     pair(tag("type"), space1),
    //     value(ImageFill::Sparse, tag("sparse")),
    // )(input)?;
    //
    // let (input, _) = newline(input)?;

    let (input, background) = preceded(
        tag("background"),
        preceded(
            space1,
            generic_delimited(
                generic_bracket_content(map(digit1, |raw: &str| raw.parse().unwrap())),
                '[',
                ']',
            ),
        ),
    )(input)?;

    let (input, _) = many1(newline)(input)?;

    let (input, commands) = separated_list0(
        newline,
        alt((
            map(
                tuple((
                    tag("pixel"),
                    space1,
                    generic_delimited(
                        generic_bracket_content(map(digit1, |raw: &str| raw.parse().unwrap())),
                        '(',
                        ')',
                    ),
                    space1,
                    generic_delimited(
                        generic_bracket_content(map(digit1, |raw: &str| raw.parse().unwrap())),
                        '[',
                        ']',
                    ),
                )),
                |(_, _, coords, _, color): (&str, &str, Vec<usize>, &str, Vec<u8>)| {
                    Command::Pixel {
                        position: (coords[0], coords[1]),
                        color: [color[0], color[1], color[2], color[3]],
                    }
                },
            ),
            map(
                tuple((
                    tag("rect"),
                    space1,
                    generic_delimited(
                        generic_bracket_content(map(digit1, |raw: &str| raw.parse().unwrap())),
                        '(',
                        ')',
                    ),
                    space1,
                    generic_delimited(
                        generic_bracket_content(map(digit1, |raw: &str| raw.parse().unwrap())),
                        '(',
                        ')',
                    ),
                    space1,
                    generic_delimited(
                        generic_bracket_content(map(digit1, |raw: &str| raw.parse().unwrap())),
                        '[',
                        ']',
                    ),
                )),
                |(_, _, coords1, _, coords2, _, color): (
                    &str,
                    &str,
                    Vec<usize>,
                    &str,
                    Vec<usize>,
                    &str,
                    Vec<u8>,
                )| Command::Rect {
                    corner_position_1: (coords1[0], coords1[1]),
                    corner_position_2: (coords2[0], coords2[1]),
                    color: [color[0], color[1], color[2], color[3]],
                },
            ),
        )),
    )(input)?;

    Ok((
        input,
        CustomImage {
            width: width,
            height: height,
            data: ImageFill::Sparse((
                commands,
                [background[0], background[1], background[2], background[3]],
            )),
        },
    ))
}

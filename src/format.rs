use rt_format::{Format, FormatArgument, ParsedFormat, Specifier};
use crate::database::SearchResult;
use std::{
    fmt,
    collections::HashMap,
};

/// Format result based on template
pub fn format_result(result: &SearchResult, template: &str) -> Result<String, ()> {
    let named_options = formatting_options(result);
    let args = ParsedFormat::parse(template, &[], &named_options).unwrap();
    Ok(format!("{}", args))
}

type OutputOptions<'a> = HashMap<&'a str, FormatValue<'a>>;

fn formatting_options<'a>(result: &SearchResult<'a>) -> OutputOptions<'a> {
    result.attributes.iter()
        .filter_map(|(key, values)| Some((key.as_ref(), FormatValue(values.iter().next()?))))
        .collect()
}

#[derive(Debug)]
struct FormatValue<'a>(&'a String);

fn format_int(
    value: &FormatValue,
    f: &mut fmt::Formatter,
    format_function: &dyn Fn(&usize, &mut fmt::Formatter) -> fmt::Result
) -> fmt::Result {
    value.0.parse::<usize>()
        .or(Err(fmt::Error))
        .and_then(|number| format_function(&number, f))
}

impl<'a> FormatArgument for FormatValue<'a> {
    fn supports_format(&self, specifier: &Specifier) -> bool {
        match specifier.format {
            Format::Display | Format::Debug => true,
            _ => self.0.parse::<usize>().is_ok(),
        }
    }

    fn fmt_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.0, f)
    }

    fn fmt_debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }

    fn fmt_octal(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_int(self, f, &fmt::Octal::fmt)
    }

    fn fmt_lower_hex(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_int(self, f, &fmt::LowerHex::fmt)
    }

    fn fmt_upper_hex(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_int(self, f, &fmt::UpperHex::fmt)
    }

    fn fmt_binary(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_int(self, f, &fmt::Binary::fmt)
    }

    fn fmt_lower_exp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_int(self, f, &fmt::LowerExp::fmt)
    }

    fn fmt_upper_exp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_int(self, f, &fmt::UpperExp::fmt)
    }

}

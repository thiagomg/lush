use rustyline::highlight::Highlighter;
use rustyline::Helper;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::completion::Completer;
use std::borrow::Cow;
use colored::*;

#[derive(Default)]
pub struct LushHighlighter;

impl Highlighter for LushHighlighter {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Owned(prompt.green().bold().to_string())
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if line.contains("function") {
            Cow::Owned(line.replace("function", &"function".blue().to_string()))
        } else {
            Cow::Borrowed(line)
        }
    }

}

impl Helper for LushHighlighter {}
impl Hinter for LushHighlighter {
    type Hint = String;
}
impl Validator for LushHighlighter {
    fn validate_while_typing(&self) -> bool {
        true
    }

    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let _ = ctx;
        Ok(ValidationResult::Valid(None))
    }
    
}

impl Completer for LushHighlighter {
    type Candidate = String;
}


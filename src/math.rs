//! Math rendering via pulldown-latex.
//!
//! Converts LaTeX math expressions to MathML Core at build-time.

use pulldown_latex::config::DisplayMode;
use pulldown_latex::{Parser, RenderConfig, Storage, push_mathml};

/// Render a LaTeX math expression to MathML.
///
/// # Arguments
/// * `latex` - The LaTeX source string
/// * `display_mode` - `true` for block equations, `false` for inline
///
/// # Returns
/// The rendered MathML string, or an error message on failure.
pub fn render_math(latex: &str, display_mode: bool) -> Result<String, String> {
    let storage = Storage::new();
    let parser = Parser::new(latex, &storage);

    let config = RenderConfig {
        display_mode: if display_mode {
            DisplayMode::Block
        } else {
            DisplayMode::Inline
        },
        xml: true,
        ..Default::default()
    };

    let mut output = String::new();
    push_mathml(&mut output, parser, config).map_err(|e| e.to_string())?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_math() {
        let result = render_math("x^2", false).unwrap();
        assert!(result.contains("<math"), "expected <math element: {result}");
    }

    #[test]
    fn test_display_math() {
        let result = render_math(r"\sum_{i=1}^n i", true).unwrap();
        assert!(result.contains("<math"), "expected <math element: {result}");
        assert!(
            result.contains("display=\"block\""),
            "expected display=block: {result}"
        );
    }

    #[test]
    fn test_mathcal() {
        let result = render_math(r"\mathcal{V}(r)", false).unwrap();
        assert!(result.contains("<math"), "expected <math element: {result}");
    }

    #[test]
    fn test_invalid_latex_no_panic() {
        // Should not panic — returns Ok with best-effort or Err
        let result = render_math(r"\invalidcommand", false);
        assert!(result.is_ok() || result.is_err());
    }
}

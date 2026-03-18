//! Math rendering via latex2mathml.
//!
//! Converts LaTeX math expressions to MathML at build-time.

use latex2mathml::{DisplayStyle, latex_to_mathml};

/// Render a LaTeX math expression to MathML.
///
/// # Arguments
/// * `latex` - The LaTeX source string
/// * `display_mode` - `true` for block equations, `false` for inline
///
/// # Returns
/// The rendered MathML string, or an error message on failure.
pub fn render_math(latex: &str, display_mode: bool) -> Result<String, String> {
    let style = if display_mode {
        DisplayStyle::Block
    } else {
        DisplayStyle::Inline
    };

    latex_to_mathml(latex, style).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_math() {
        let result = render_math("x^2", false).unwrap();
        assert!(result.contains("<math"));
    }

    #[test]
    fn test_display_math() {
        let result = render_math(r"\sum_{i=1}^n i", true).unwrap();
        assert!(result.contains("<math"));
        assert!(result.contains("display=\"block\""));
    }

    #[test]
    fn test_invalid_latex_no_panic() {
        // Should not panic — returns Ok with best-effort or Err
        let result = render_math(r"\invalidcommand", false);
        assert!(result.is_ok() || result.is_err());
    }
}

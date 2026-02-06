//! Math rendering via katex-rs.
//!
//! Converts LaTeX math expressions to HTML at build-time.

use katex::{KatexContext, Settings, render_to_string};

/// Render a LaTeX math expression to HTML.
///
/// # Arguments
/// * `latex` - The LaTeX source string
/// * `display_mode` - `true` for block equations, `false` for inline
///
/// # Returns
/// The rendered HTML string, or an error message on failure.
pub fn render_math(latex: &str, display_mode: bool) -> Result<String, String> {
    let ctx = KatexContext::default();
    let settings = Settings::builder()
        .display_mode(display_mode)
        .throw_on_error(false)
        .build();

    render_to_string(&ctx, latex, &settings).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_math() {
        let result = render_math("x^2", false).unwrap();
        assert!(result.contains("<span"));
    }

    #[test]
    fn test_display_math() {
        let result = render_math(r"\sum_{i=1}^n i", true).unwrap();
        assert!(result.contains("<span"));
    }

    #[test]
    fn test_invalid_latex_no_panic() {
        // Should not panic, returns error or graceful fallback
        let result = render_math(r"\invalidcommand", false);
        // katex-rs with throw_on_error=false returns error markup
        assert!(result.is_ok() || result.is_err());
    }
}

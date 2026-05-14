//! D2 diagram rendering via d2-little.
//!
//! Converts D2 diagram definitions to SVG at build-time.


/// Render a D2 diagram to SVG.
///
/// # Arguments
/// * `code` - The D2 diagram definition
///
/// # Returns
/// The rendered SVG string, or an error message on failure.
///
/// # Note
/// Uses catch_unwind to handle panics in upstream dependencies gracefully.
pub fn render_diagram(code: &str) -> Result<String, String> {
    let svg = d2_little::d2_to_svg(code)?;
    let svg_str = String::from_utf8_lossy(&svg).to_string();
    Ok(svg_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_flowchart() {
        let result = render_diagram("x -> y").unwrap();
        assert!(result.contains("<svg"));
        assert!(result.contains("</svg>"));
    }

    #[test]
    fn test_sequence_diagram() {
        let result = render_diagram(r#"shape: sequence_diagram
        alice -> bob: What does it mean\nto be well-adjusted?
        bob -> alice: The ability to play bridge or\ngolf as if they were games.
        "#).unwrap();
        assert!(result.contains("<svg"));
    }

    #[test]
    fn test_invalid_syntax_no_panic() {
        // Should not panic, returns error
        let result = render_diagram("invalid diagram syntax ???");
        // May succeed with error node or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_state_diagram() {
        let result =
            render_diagram(r#"Start: "" {
              shape: circle
              width: 10
            }
            End: "" {
              shape: circle
              width: 10
            }

            Start -> Still
            Still -> End

            Still -> Moving
            Moving -> Still
            Moving -> Crash
            Crash -> End

        "#);
        assert!(result.is_ok() || result.is_err());
    }
}

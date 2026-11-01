use super::{SlideApp, Value, json};

/// Text put on the system clipboard when the copied objects (a rectangle, a picture, slides…)
/// carry no text. egui-winit drops Ctrl+V when the system clipboard is empty, so without it the
/// shortcut never reached the session and only the context menu could paste objects (#182). An
/// invisible word joiner, so a paste into another app adds nothing visible.
pub(crate) const OBJECTS_MARKER: &str = "\u{2060}";

/// What a copy/cut leaves on the system clipboard for the session's plain-text rendering `t`.
pub(crate) fn system_clipboard_text(t: &str) -> String {
    if t.is_empty() { OBJECTS_MARKER.to_string() } else { t.to_string() }
}

/// Whether clipboard HTML is exactly one `<img>` and no visible text, as a browser's "Copy Image"
/// leaves it. Rich text that merely contains an inline image, and anything with words, is not.
pub(crate) fn html_is_one_bare_image(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    if lower.matches("<img").count() != 1 {
        return false;
    }
    let mut in_tag = false;
    let mut text = String::new();
    for c in lower.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    text.replace("&nbsp;", " ").trim().is_empty()
}

impl SlideApp {
    /// Offer the copy or cut on the system clipboard.
    ///
    /// Slides go out as plain text. Text goes out as HTML plus the same plain words, so a paste
    /// back into DeckCraft still uses the internal clipboard. A textless shape leaves the
    /// invisible [`OBJECTS_MARKER`] rather than a picture: with only an image on the clipboard
    /// egui-winit drops Ctrl+V, and the shortcut could no longer paste the shape here (#182).
    pub(super) fn publish_clipboard(&mut self, ctx: &egui::Context, cut: bool) {
        let cmd = if cut { "edit.cut" } else { "edit.copy" };
        let Ok(v) = self.session.execute(cmd, &json!({})) else { return };
        let text = v.get("text").and_then(Value::as_str).unwrap_or("").to_string();
        let html = v.get("html").and_then(Value::as_str).unwrap_or("").to_string();
        if self.session.clipboard.slides.is_empty() && !html.is_empty() && self.services.clipboard_set_html.as_mut().is_some_and(|f| f(&html, &text))
        {
            return;
        }
        ctx.copy_text(system_clipboard_text(&text));
    }
}

#[cfg(test)]
mod tests {
    use super::html_is_one_bare_image;

    #[test]
    fn only_a_lone_image_counts() {
        assert!(html_is_one_bare_image(r#"<a href="https://example.org/s"><img src="https://example.org/o.png" alt="x"></a>"#));
        assert!(html_is_one_bare_image("<meta charset='utf-8'><!--StartFragment--><IMG SRC=a.png>&nbsp;<!--EndFragment-->"));
        assert!(!html_is_one_bare_image("<p>Look: <img src=a.png></p>"));
        assert!(!html_is_one_bare_image("<img src=a.png><img src=b.png>"));
        assert!(!html_is_one_bare_image("<p><b>Hi</b></p>"));
        assert!(!html_is_one_bare_image(""));
    }
}

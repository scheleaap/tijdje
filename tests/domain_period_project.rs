mod common;

use common::{PROJECT, period};
use tijdje::domain::PeriodType::Work;

#[test]
fn setting_the_project_keeps_its_text() {
    // Given
    let mut p = period(900, Some(1200), Work);

    // When
    p.set_project(" otto ");

    // Then
    assert_eq!(p.project.as_deref(), Some(" otto "));
}

#[test]
fn a_project_of_only_spaces_is_no_project() {
    for text in ["", "   "] {
        // Given
        let mut p = period(900, Some(1200), Work);
        assert_eq!(p.project.as_deref(), Some(PROJECT));

        // When
        p.set_project(text);

        // Then
        assert_eq!(p.project, None, "{text:?}");
    }
}

# 11: Visual styling in the style of Midnight Commander

**What to build:** The Tracking screen and popups look like Midnight Commander, and the right panel gets a calmer, clearer layout. Picks up the visual items from the feedback log. Colors (fields and popup buttons) will likely be revisited a few times until they're settled.

**Blocked by:** 07

**Status:** done

- [x] Panels: light gray text on blue, single-line borders, yellow column headers; the active panel's title is black on light gray
- [x] Left panel title `Days: <year of the selected day>`
- [x] Year separator shows the year centered, e.g. `═══ 2027 ═══`
- [x] Selected day: cyan bar with black text in the active left panel; in an inactive left panel no bar, the whole row in cyan text
- [x] Right panel fields and checkboxes: light gray boxes with black text (a checked box shows ✓); the focused field cyan, also while editing; Tab order follows the screen
- [x] Right panel layout: period warning ⚠️ left of From; checkboxes right after Project; Notes last, taking the remaining width
- [x] A period of 0h00m also gets the period warning ⚠️
- [x] An empty row between periods
- [x] An implied break is its own row: From/To/Duration as plain text, Break checked, no fields, not focusable
- [x] Popups: light gray background with black text, a black drop shadow, a short title in the frame in blue and the message inside (wrapped); solid buttons, black on white, the selected one black on cyan, the hotkey letter in blue

Spec: `.scratch/time-tracking/spec.md` ("Look" and "Right panel layout"); decisions Q127–Q134, adjusted by the user's feedback during implementation (no column lines, light gray fields, ✓ checkboxes, Tab order).

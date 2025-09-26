# canvas_calendar_split

**Problem:**
(As far as I can tell) Canvas doesn't provide a way to export Calendars for individual classes

**Solution:**
Tool for splitting a calendar exported from Canvas into 1 per class code

**Warning:**
- Probably doesn't work for all classes at ASU
- Probably doesn't work for other schools

### Usage

- Pass in path to original `.ics` file exported by Canvas as the only argument
- Creates `output_calendars` directory wherever you run the binary from
- Creates `output_calendars/<class_code>.ics` for each unique class code found
- Creates `output_calendars/no_associated_class.ics` for events without a class code

### Class Code
- **Class Code** for an event is pulled from the `SUMMARY` line inside of square brackets `[]`
- Ex: Class code here would be "2025FallA-X-CSE230-123-456"
```
BEGIN:VEVENT
CLASS:PUBLIC
SUMMARY: Syllabus Quiz [2025FallA-X-CSE230-123-456]
END:VEVENT
```
- Ex: Missing class code (inside `[]`) and no `SUMMARY` get put in `no_associated_class.ics`:
```
BEGIN:VEVENT
CLASS:PUBLIC
SUMMARY: Syllabus Quiz 2025FallA-X-CSE230-123-456
END:VEVENT

BEGIN:VEVENT
CLASS:PUBLIC
END:VEVENT
```

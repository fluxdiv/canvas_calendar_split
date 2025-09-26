use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use anyhow::Result;
use rustc_hash::FxHashMap;
use icalendar::{Calendar, CalendarComponent, Component, Property};
// Won't know alias at runtime unless I use a config

/// Wraps `FxHashMap<String, Vec<String>>` where
pub struct Classes {
    /// Header properties from original calendar
    header: Vec<Property>,
    /// `String` Keys are class code
    calendars: FxHashMap<String, Calendar>
}

/// Calendar for components from canvas without a class (self created etc)
pub static DEFAULT_CLASS: &str = "no_associated_class";

impl Classes {
    /// Creates new `Classes` 
    /// - Preinserts a default/no class calendar
    /// - Holds original calender header, used when 
    ///   inserting a new class
    pub fn new(cal_header: Vec<Property>) -> Self {
        let mut m: FxHashMap<String, Calendar> = FxHashMap::default();
        // Calendar for no-associated-class events
        let default_cal = Calendar::empty();
        // let header = get_new_header(DEFAULT_CLASS.to_string(), vec![]);
        // for p in header.into_iter() {
        //     default_cal.append_property(p);
        // };
        m.insert(DEFAULT_CLASS.to_string(), default_cal);
        Self {
            header: cal_header,
            calendars: m
        }
    }

    /// Insert calendar event into existing class or create if class doesnt exist yet
    pub fn insert_component(&mut self, class_code: String, component: CalendarComponent) {
        self.calendars.entry(class_code)
            .and_modify(|calendar| {
                calendar.push(component.clone());
            })
            .or_insert_with(|| {
                let mut new_cal = Calendar::new();
                new_cal.push(component);
                new_cal
            });
    }

    /// Create file for each class_code & write all of its components
    pub fn finalize(mut self) -> Result<()> {
        // Create outdir if needed
        let mut cwd = env::current_dir()?;
        cwd.push("output_calendars");
        if !cwd.exists() {
            let _ = fs::create_dir(&cwd)?;
        };

        // Create out calendar file for each class code
        for (class_code, calendar) in self.calendars.iter_mut() {
            if class_code.eq(DEFAULT_CLASS) {
                if calendar.is_empty() {
                    continue;
                }
            }
            let mut fp = cwd.clone();
            fp.push(class_code);
            let mut class_file = OpenOptions::new()
                .read(true).write(true).create_new(true)
                .open(&fp)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let header = get_new_header(class_code.clone(), self.header.clone());
            for p in header.into_iter() {
                calendar.append_property(p);
            }
            let cal_data = calendar.to_string();
            class_file.write_all(&cal_data.as_bytes())?;
            println!("Calendar created: {}", fp.display());
        }

        Ok(())
    }
}

fn main() -> Result<()> {

    let cal_to_split_path = env::args().nth(1)
        .ok_or_else(|| anyhow::anyhow!("Must provide path of calendar to split"))?;
    let cal_content = fs::read_to_string(&cal_to_split_path)?;
    let cal: Calendar = cal_content
        .parse().map_err(|e| anyhow::anyhow!("{}", e))?;

    // Properties are lines before first event
    let mut classes = Classes::new(cal.properties.clone());

    // Components are Events / Todos / Venues
    // Extract class code & insert into map
    for c in &cal.components {
        if let CalendarComponent::Event(event) = c {
            if let Some(code) = extract_class_code(event.get_summary()) {
                classes.insert_component(code, c.clone());
            } else {
                classes.insert_component(DEFAULT_CLASS.to_string(), c.clone());
            }
        } else {
            classes.insert_component(DEFAULT_CLASS.to_string(), c.clone());
        }
    }

    classes.finalize()?;

    Ok(())
}

/// Extract class code from a `SUMMARY` line
/// - Ex: `SUMMARY:Final Exam [2025FallC-X-CSE360-77646]`
/// would return `2025FallC-X-CSE360-77646`
/// - Returns None if not `SUMMARY` or no square brackets
fn extract_class_code(summary: Option<&str>) -> Option<String> {
    let sum = summary?;
    let start = sum.find('[')?;
    let end = sum.find(']')?;
    if end > start {
        return Some(sum[start + 1..end].to_string());
    }
    None
}

/// Get updated header for a class
fn get_new_header(class: String, header: Vec<Property>) -> Vec<Property> {
    let mut new_header: Vec<Property> = vec![];

    for p in header.iter() {
        let key = p.key();
        match key {
            "PRODID" => {
                new_header.push(
                    Property::new(
                        "PRODID", 
                        "-//github.com/fluxdiv/canvas_calendar_split//EN"
                    )
                );
            },
            "X-WR-CALNAME" => {
                new_header.push(
                    Property::new("X-WR-CALNAME", class.clone())
                );
            },
            "X-WR-CALDESC" => {
                let d = format!("Calendar events for {}", class);
                new_header.push(Property::new("X-WR-CALDESC", d));
            },
            _ => {
                new_header.push(p.clone());
            }
        }
    };
    new_header
}


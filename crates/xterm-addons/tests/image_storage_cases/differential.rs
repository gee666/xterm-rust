use crate::support::*;
use serde_json::{json, Value};
use std::rc::Rc;
use xterm_addons::image::storage::*;
use xterm_core::{buffer::AttributeData, event::EmitError};

fn n(value: f64) -> Value {
    Value::String(if value == 0.0 && value.is_sign_negative() {
        "-0".into()
    } else {
        number(value)
    })
}
fn parse(value: &Value) -> f64 {
    value
        .as_f64()
        .unwrap_or_else(|| value.as_str().unwrap().parse().unwrap())
}
fn index(value: &Value, key: &str) -> usize {
    usize::try_from(value[key].as_u64().unwrap()).unwrap()
}
fn resource(value: Option<ImageResource>) -> Value {
    value.map_or(Value::Null, |image| {
        json!([
            image.width().unwrap(),
            image.height().unwrap(),
            image.bitmap_close().unwrap().is_some()
        ])
    })
}
fn snapshot(f: &Fixture) -> Value {
    let active = f
        .terminal
        .with_state(|state| state.active_buffer())
        .unwrap();
    let buffer = active.borrow();
    let cells = (0..buffer.lines.len())
        .map(|row| {
            let line = buffer.lines.get(row).unwrap();
            let line = line.borrow();
            (0..line.len())
                .map(|col| {
                    let cell = line.get_cell(col).unwrap();
                    let payload = cell
                        .extended
                        .payload()
                        .and_then(|value| value.downcast::<ImageTileInfo>().ok());
                    json!([
                        cell.content,
                        cell.fg,
                        cell.bg,
                        cell.extended.ext(),
                        cell.extended.url_id(),
                        payload.map(|tile| json!([tile.image_id.get(), n(tile.tile_id.get())]))
                    ])
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let service = f
        .terminal
        .with_state(|state| state.buffer_service.clone())
        .unwrap();
    let kind = if Rc::ptr_eq(&active, &service.buffers.normal()) {
        "normal"
    } else {
        "alternate"
    };
    let images=f.storage.image_ids().into_iter().map(|id| {
        let spec=f.storage.image_spec(id).unwrap();let actual=spec.actual_resource();
        json!({"id":id,"original":resource(Some(spec.original.clone())),"actual":resource(actual.clone()),
          "same":actual.is_some_and(|actual|actual.same_identity(&spec.original)),
          "cell":[n(spec.original_cell_size.width),n(spec.original_cell_size.height)],
          "actual_cell":[n(spec.actual_cell_size.get().width),n(spec.actual_cell_size.get().height)],
          "count":n(spec.tile_count.get()),"buffer":match spec.buffer_type{ImageBufferType::Normal=>"normal",ImageBufferType::Alternate=>"alternate"},
          "layer":match spec.layer{ImageLayer::Top=>"top",ImageLayer::Bottom=>"bottom"},"z":n(spec.z_index),
          "marker":spec.marker.as_ref().map(|marker|marker.is_disposed())})
    }).collect::<Vec<_>>();
    let marker_count = buffer.markers.borrow().len();
    let events = std::mem::take(&mut *f.events.borrow_mut());
    json!({"cursor":[buffer.x,buffer.y,buffer.ybase,buffer.ydisp],"dimensions":[service.cols(),service.rows()],
      "buffer":kind,"marker_count":marker_count,"limit":n(f.storage.get_limit()),"usage":n(f.storage.get_usage().unwrap()),
      "images":images,"cells":cells,"events":events})
}
fn execute(f: &Fixture, step: &Value) -> Result<Value, StorageError> {
    match step["op"].as_str().unwrap() {
        "cursor" => {
            f.set_cursor(index(step, "x"), index(step, "y"));
        }
        "add" => {
            let width = u32::try_from(index(step, "w")).unwrap();
            let height = u32::try_from(index(step, "h")).unwrap();
            let image = if step["bitmap"].as_bool().unwrap_or(false) {
                bitmap("random", width, height, f.events.clone(), None)
            } else {
                raster(width, height)
            };
            let label = step["label"].as_str().unwrap_or("random");
            let image = if step["bitmap"].as_bool().unwrap_or(false) && label == "bitmap" {
                bitmap("bitmap", width, height, f.events.clone(), None)
            } else {
                image
            };
            let id = f.storage.add_image(
                image,
                AddImageOptions {
                    scrolling: step["scrolling"].as_bool().unwrap_or(true),
                    layer: if step["layer"] == "bottom" {
                        ImageLayer::Bottom
                    } else {
                        ImageLayer::Top
                    },
                    z_index: if step.get("z").is_some() {
                        parse(&step["z"])
                    } else {
                        0.0
                    },
                    cursor_position: if step["cursor"] == "iip" {
                        CursorPosition::Iip
                    } else {
                        CursorPosition::Vt340
                    },
                },
            )?;
            return Ok(json!(id));
        }
        "limit" => f.storage.set_limit(parse(&step["value"]))?,
        "delete" => f.storage.delete_image(step["id"].as_u64().unwrap())?,
        "reset" => f.storage.reset()?,
        "render" => f.storage.render(index(step, "start"), index(step, "end"))?,
        "resize" => f
            .terminal
            .resize(index(step, "cols"), index(step, "rows"))
            .map_err(StorageError::host)?,
        "wipe" => f.storage.wipe_alternate()?,
        "alt" | "normal" => {
            let service = f
                .terminal
                .with_state(|state| state.buffer_service.clone())
                .unwrap();
            if step["op"] == "alt" {
                service.buffers.activate_alt_buffer(None)?;
            } else {
                service.buffers.activate_normal_buffer()?;
            }
        }
        "placeholder" => f
            .storage
            .set_show_placeholder(step["value"].as_bool().unwrap()),
        "metrics" => f.renderer.size.set(CellSize {
            width: parse(&step["cell"][0]),
            height: parse(&step["cell"][1]),
        }),
        "actual" => f
            .storage
            .image_spec(step["id"].as_u64().unwrap())
            .unwrap()
            .replace_actual_resource(if step["none"].as_bool().unwrap_or(false) {
                None
            } else {
                Some(raster(
                    u32::try_from(index(step, "w")).unwrap(),
                    u32::try_from(index(step, "h")).unwrap(),
                ))
            }),
        "seed" => {
            let buffer = f
                .terminal
                .with_state(|state| state.active_buffer())
                .unwrap();
            let line = buffer.borrow().lines.get(index(step, "y")).unwrap();
            line.borrow_mut().set_cell_from_codepoint(
                index(step, "x"),
                u32::try_from(index(step, "code")).unwrap(),
                1,
                &AttributeData {
                    fg: step["fg"].as_u64().unwrap_or(0) as u32,
                    bg: step["bg"].as_u64().unwrap_or(0) as u32,
                    ..Default::default()
                },
            );
        }
        "get" => {
            return f
                .storage
                .get_image_at_buffer_cell(index(step, "x"), index(step, "y"))
                .map(resource)
        }
        "extract" => {
            return f
                .storage
                .extract_tile_at_buffer_cell(index(step, "x"), index(step, "y"))
                .map(resource)
        }
        other => panic!("unknown source operation {other}"),
    }
    Ok(Value::Null)
}

#[test]
fn actual_buffers_and_image_storage_match_source_snapshots() {
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/image-storage.json")).unwrap();
    let mut count = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let config = &case["config"];
        let f = Fixture::new(
            index(config, "cols"),
            index(config, "rows"),
            CellSize {
                width: parse(&config["cell"][0]),
                height: parse(&config["cell"][1]),
            },
        );
        let added = f.events.clone();
        f.storage.set_on_image_added(Some(Rc::new(move || {
            added.borrow_mut().push("added".into());
            Ok(())
        })));
        let deleted = f.events.clone();
        f.storage.set_on_image_deleted(Some(Rc::new(move |id| {
            deleted.borrow_mut().push(format!("deleted:{id}"));
            Ok(())
        })));
        let events = f.events.clone();
        let linefeed = f
            .terminal
            .on_line_feed()
            .subscribe(move |_| events.borrow_mut().push("linefeed".into()));
        let weak = f.storage.downgrade();
        let resized = f.terminal.on_resize().try_subscribe(move |event| {
            weak.upgrade()
                .unwrap()
                .viewport_resize(event.cols, event.rows)
                .map_err(EmitError::callback)
        });
        for (step_index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
            let result = execute(&f, step);
            if let Some(expected) = step.get("error") {
                let error = result.unwrap_err();
                let kind = if matches!(error, StorageError::InvalidLimit) {
                    "RangeError"
                } else {
                    "Error"
                };
                assert_eq!(
                    format!("{kind}:{error}"),
                    expected.as_str().unwrap(),
                    "{} step{step_index}",
                    case["name"]
                );
            } else {
                assert_eq!(
                    result.unwrap(),
                    step["result"],
                    "{} step{step_index}",
                    case["name"]
                );
            }
            assert_eq!(
                snapshot(&f),
                step["expected"],
                "{} step{step_index} op {}",
                case["name"],
                step["op"]
            );
            count += 1;
        }
        linefeed.dispose();
        resized.dispose();
    }
    assert_eq!(count, 563);
}

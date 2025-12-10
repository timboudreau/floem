use baseview::DropData;
use dpi::PhysicalPosition;
use event::{Event, FileDragEvent};
use peniko::kurbo::Size;
use std::path::PathBuf;

pub fn convert_window_event(evt: &baseview::WindowEvent) -> Event {
    match evt {
        baseview::WindowEvent::Resized(window_info) => {
            let p = window_info.physical_size();
            Event::WindowResized(Size {
                width: p.width as f64,
                height: p.height as f64,
            })
        }
        baseview::WindowEvent::Focused => Event::WindowGotFocus,
        baseview::WindowEvent::Unfocused => Event::WindowLostFocus,
        baseview::WindowEvent::WillClose => Event::WindowClosed,
    }
}

pub fn convert_drag_event(evt: &baseview::MouseEvent, scale_factor: f64) -> Option<Event> {
    Some(match evt {
        baseview::MouseEvent::DragEntered {
            position,
            modifiers: _,
            data,
        } => Event::FileDrag(FileDragEvent::DragEntered {
            paths: to_path_vec(data),
            position: PhysicalPosition {
                x: position.x,
                y: position.y,
            },
            scale_factor: scale_factor,
        }),
        baseview::MouseEvent::DragMoved {
            position,
            modifiers: _,
            data: _,
        } => Event::FileDrag(FileDragEvent::DragMoved {
            position: PhysicalPosition {
                x: position.x,
                y: position.y,
            },
            scale_factor: scale_factor,
        }),
        baseview::MouseEvent::DragLeft => todo!(),
        baseview::MouseEvent::DragDropped {
            position,
            modifiers: _,
            data,
        } => Event::FileDrag(FileDragEvent::DragEntered {
            paths: to_path_vec(data),
            position: PhysicalPosition {
                x: position.x,
                y: position.y,
            },
            scale_factor: scale_factor,
        }),
        _ => return None,
    })
}

fn to_path_vec(drop_data: &DropData) -> Vec<PathBuf> {
    match drop_data {
        DropData::Files(v) => v.clone(),
        DropData::None => vec![],
    }
}

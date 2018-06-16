use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;
use std::vec::IntoIter;

use save;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum UserMessage {
    Ok(String),
    Error(String),
    Fatal(String),
}

pub struct UiStatus {
    master_system: Box<Sms>,
    save_directory: Option<PathBuf>,
    recording_status: RecordingStatus<SmsState>,
    messages: Arc<RwLock<Vec<UserMessage>>>,
}

fn push_or_panic<T>(messages: &mut Arc<RwLock<Vec<T>>>, t: T) {
    match messages.write() {
        Ok(mut vec) => vec.push(t),
        Err(e) => panic!("Poisoned RwLock {}", e),
    }
}

fn do_in_thread<T, F>(mut messages: Arc<RwLock<Vec<T>>>, f: F)
where
    T: Send + Sync + 'static,
    F: FnOnce() -> Option<T> + Send + 'static,
{
    thread::spawn(move || {
        if let Some(t) = f() {
            push_or_panic(&mut messages, t);
        }
    });
}

impl UiStatus {
    pub fn master_system(&self) -> &Sms {
        self.master_system.deref()
    }

    pub fn master_system_mut(&mut self) -> &mut Sms {
        self.master_system.deref_mut()
    }

    pub fn master_system_own(self) -> Box<Sms> {
        self.master_system
    }

    pub fn messages(&mut self) -> IntoIter<UserMessage> {
        use std::mem::swap;

        match self.messages.write() {
            Ok(ref mut vec) => {
                let mut vec2 = Vec::new();
                swap(vec.deref_mut(), &mut vec2);
                vec2.into_iter()
            }
            Err(e) => panic!("Poisoned RwLock {}", e),
        }
    }

    pub fn save_state(&mut self, name: Option<&str>) {
        if let Some(mut path) = self.save_directory.clone() {
            let filename = generate_filename(name);
            let state = Sms::state(self.master_system.deref());
            do_in_thread(self.messages.clone(), move || {
                path.push(format!("{}.sms_state", filename));
                if let Err(e) = save::serialize_at(&path, &state) {
                    Some(UserMessage::Error(format!(
                        "Cannot save state to '{}': {}",
                        path.to_string_lossy(),
                        e
                    )))
                } else {
                    Some(UserMessage::Ok(format!(
                        "Saved state to '{}'",
                        path.to_string_lossy(),
                    )))
                }
            });
        } else {
            push_or_panic(
                &mut self.messages,
                UserMessage::Error("Cannot save state: No save directory specified".to_owned()),
            );
        }
    }

    pub fn begin_recording(&mut self) {
        let state = Sms::state(self.master_system.deref());
        self.recording_status.begin_recording(state);
        push_or_panic(
            &mut self.messages,
            UserMessage::Ok("Started recording".to_owned()),
        );
    }

    pub fn save_recording(&mut self, name: Option<&str>) {
        if let (Some(mut path), Some(recording)) = (
            self.save_directory.clone(),
            self.recording_status.recording(),
        ) {
            let filename = generate_filename(name);
            let recording2 = recording.clone();
            do_in_thread(self.messages.clone(), move || {
                path.push(format!("{}.sms_record", filename));
                if let Err(e) = save::serialize_at(&path, &recording2) {
                    Some(UserMessage::Error(format!(
                        "Cannot save recording to '{}': {}",
                        path.to_string_lossy(),
                        e
                    )))
                } else {
                    Some(UserMessage::Ok(format!(
                        "Saved recording to '{}'",
                        path.to_string_lossy(),
                    )))
                }
            });
            return;
        }
        push_or_panic(
            &mut self.messages,
            UserMessage::Error("Cannot save state: No save directory specified".to_owned()),
        )
    }

    pub fn end_recording(&mut self) {
        self.recording_status.end_recording();
        push_or_panic(
            &mut self.messages,
            UserMessage::Ok("Ended recording".to_owned()),
        );
    }
}

pub trait UiHelper {
    fn frame_update(
        &mut self,
        ui: &mut UiStatus,
    ) -> Result<Option<SmsPlayerInputState>, SmsEmulationError>;
}

pub struct Ui {
    status: UiStatus,
    helper: Box<UiHelper>,
}

impl Ui {
    pub fn new(
        master_system: Box<Sms>,
        helper: Box<UiHelper>,
        save_directory: Option<PathBuf>,
    ) -> Self {
        Ui {
            status: UiStatus {
                master_system,
                save_directory,
                recording_status: Default::default(),
                messages: Default::default(),
            },
            helper,
        }
    }

    pub fn master_system(&self) -> &Sms {
        self.status.master_system.deref()
    }

    pub fn master_system_mut(&mut self) -> &mut Sms {
        self.status.master_system.deref_mut()
    }

    pub fn master_system_own(self) -> Box<Sms> {
        self.status.master_system
    }

    pub fn run(&mut self) -> Result<(), SmsEmulationError> {
        self.status.master_system.resume()?;
        loop {
            match self.helper.frame_update(&mut self.status)? {
                None => return Ok(()),
                Some(player_input) => {
                    self.status.recording_status.update(player_input);
                    self.status.master_system.run_frame(player_input)?;
                }
            };
        }
    }
}

fn generate_filename(name: Option<&str>) -> String {
    use chrono::prelude::Local;
    match name {
        Some(s) => s.to_owned(),
        None => format!("{}", Local::now().format("%Y-%m-%d %H-%M-%S-%f")),
    }
}

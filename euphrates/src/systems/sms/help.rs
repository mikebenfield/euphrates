use super::*;

/// Contains a saved recording of gameplay, together with the initial state of
/// the Master System. This is what is written when gameplay is saved to a file.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Recording<H> {
    pub state: H,
    pub player_statuses: Vec<SmsPlayerInput>,
}

/// Internal type for UserInterface to record gameplay
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RecordingStatus<S>(Option<Box<Recording<S>>>);

impl<S> Default for RecordingStatus<S> {
    fn default() -> Self {
        RecordingStatus(None)
    }
}

impl<S> RecordingStatus<S> {
    /// Call this every frame, after reading player's status but before
    /// emulating the frame
    pub fn update(&mut self, player_status: SmsPlayerInput) {
        if let Some(ref mut recording) = self.0 {
            recording.player_statuses.push(player_status)
        }
    }

    pub fn begin_recording(&mut self, state: S) {
        self.0 = Some(Box::new(Recording {
            state,
            player_statuses: Vec::with_capacity(256),
        }))
    }

    pub fn end_recording(&mut self) {
        self.0 = None
    }

    pub fn recording(&self) -> Option<&Recording<S>> {
        match self.0 {
            None => None,
            Some(ref r) => Some(r),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PlaybackStatus(Vec<SmsPlayerInput>);

impl PlaybackStatus {
    pub fn from_recorded(player_statuses: &[SmsPlayerInput]) -> PlaybackStatus {
        let mut v = player_statuses.to_vec();
        v.reverse();
        PlaybackStatus(v)
    }

    pub fn pop(&mut self) -> Option<SmsPlayerInput> {
        self.0.pop()
    }

    pub fn end_playback(&mut self) {
        self.0 = Vec::new();
    }
}

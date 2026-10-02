use super::types::ToneNote;

/// Parsed RTTTL (Ring Tone Text Transfer Language) musical score.
#[derive(Clone, Debug, PartialEq)]
pub struct RtttlMelody {
    pub name: String,
    pub notes: Vec<ToneNote>,
}

impl RtttlMelody {
    /// Parses an RTTTL string into a melody with sequence of timed tone notes.
    pub fn parse(input: &str) -> Result<Self, String> {
        let parts: Vec<&str> = input.split(':').collect();
        if parts.len() < 3 {
            return Err("Invalid RTTTL format: expected 3 colon-delimited sections".to_string());
        }

        let name = parts[0].trim().to_string();
        let (def_duration, def_octave, bpm) = parse_defaults(parts[1])?;
        let notes = parse_melody_notes(parts[2], def_duration, def_octave, bpm)?;

        Ok(Self { name, notes })
    }

    /// Total duration of the melody in milliseconds.
    pub fn total_duration_ms(&self) -> u32 {
        self.notes
            .iter()
            .fold(0u32, |total, note| total.saturating_add(note.duration_ms))
    }
}

/// Helper parsing the default duration, octave, and tempo header section.
fn parse_defaults(section: &str) -> Result<(u32, u32, u32), String> {
    let mut duration = 4;
    let mut octave = 5;
    let mut bpm = 120;

    for item in section.split(',') {
        let pair: Vec<&str> = item.trim().split('=').collect();
        if pair.len() == 2 {
            let key = pair[0].trim().to_ascii_lowercase();
            let val = pair[1].trim();
            match key.as_str() {
                "d" => duration = val.parse().unwrap_or(4),
                "o" => octave = val.parse().unwrap_or(5),
                "b" => bpm = val.parse().unwrap_or(120),
                _ => {}
            }
        }
    }

    Ok((duration, octave, bpm))
}

/// Helper parsing comma-separated musical notes from the RTTTL score.
fn parse_melody_notes(
    notes_str: &str,
    def_dur: u32,
    def_oct: u32,
    bpm: u32,
) -> Result<Vec<ToneNote>, String> {
    let whole_note_ms = 240_000.0 / bpm as f32;
    let mut result = Vec::new();

    for raw_note in notes_str.split(',') {
        let trimmed = raw_note.trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            continue;
        }

        let mut chars = trimmed.chars().peekable();
        let mut dur_str = String::new();
        while let Some(&c) = chars.peek()
            && c.is_ascii_digit()
        {
            dur_str.push(c);
            chars.next();
        }
        let duration: u32 = if dur_str.is_empty() {
            def_dur
        } else {
            dur_str.parse().unwrap_or(def_dur)
        };

        let note_char = chars.next().unwrap_or('p');
        let mut is_sharp = false;
        if let Some(&c) = chars.peek()
            && (c == '#' || c == '_')
        {
            is_sharp = true;
            chars.next();
        }

        let mut is_dotted = false;
        if let Some(&c) = chars.peek()
            && c == '.'
        {
            is_dotted = true;
            chars.next();
        }

        let mut oct_val = def_oct;
        if let Some(&c) = chars.peek()
            && c.is_ascii_digit()
        {
            oct_val = c.to_digit(10).unwrap_or(def_oct);
            chars.next();
        }

        if let Some(&c) = chars.peek()
            && c == '.'
        {
            is_dotted = true;
        }

        let mut dur_ms = whole_note_ms / duration as f32;
        if is_dotted {
            dur_ms *= 1.5;
        }

        if note_char == 'p' {
            result.push(ToneNote::pause((dur_ms as u32).max(1)));
        } else {
            let freq = calculate_note_frequency(note_char, is_sharp, oct_val);
            result.push(ToneNote::note(freq, (dur_ms as u32).max(1)));
        }
    }

    Ok(result)
}

/// Calculates pitch frequency from musical note, accidental, and octave.
fn calculate_note_frequency(note: char, sharp: bool, octave: u32) -> f32 {
    let semitone = match note {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => 9,
    } + if sharp { 1 } else { 0 };

    let midi_number = (octave as i32 + 1) * 12 + semitone;
    440.0 * 2.0_f32.powf((midi_number - 69) as f32 / 12.0)
}

/// Sequencer state machine for playing through an RTTTL melody.
#[derive(Clone, Debug)]
pub struct RtttlPlayer {
    pub melody: RtttlMelody,
    pub current_index: usize,
    pub elapsed_note_ms: f32,
    pub is_playing: bool,
    pub repeat: bool,
}

impl RtttlPlayer {
    /// Creates a player initialized with the given melody.
    pub fn new(melody: RtttlMelody) -> Self {
        Self {
            melody,
            current_index: 0,
            elapsed_note_ms: 0.0,
            is_playing: false,
            repeat: false,
        }
    }

    /// Advances playback by delta milliseconds and returns the current frequency if active.
    pub fn advance_ms(&mut self, dt_ms: f32) -> Option<f32> {
        if !self.is_playing || self.melody.notes.is_empty() {
            return None;
        }

        self.elapsed_note_ms += dt_ms;
        let mut note = &self.melody.notes[self.current_index];

        loop {
            let note_duration = note.duration_ms.max(1) as f32;
            if self.elapsed_note_ms < note_duration {
                break;
            }
            self.elapsed_note_ms -= note_duration;
            self.current_index += 1;
            if self.current_index >= self.melody.notes.len() {
                if self.repeat {
                    self.current_index = 0;
                } else {
                    self.is_playing = false;
                    self.current_index = 0;
                    return None;
                }
            }
            note = &self.melody.notes[self.current_index];
        }

        if note.is_pause {
            None
        } else {
            Some(note.frequency)
        }
    }

    /// Starts or resumes melody playback.
    pub fn play(&mut self) {
        self.is_playing = true;
    }

    /// Pauses melody playback.
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// Stops playback and rewinds to the beginning.
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_index = 0;
        self.elapsed_note_ms = 0.0;
    }
}

/// Standard preset RTTTL ringtones.
pub fn get_preset_melodies() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "Nokia Tune",
            "Nokia:d=4,o=5,b=225:e6,d6,f#,g#,c#6,b,d,e,b,a,c#,e,a",
        ),
        (
            "Imperial March",
            "Imperial:d=4,o=4,b=120:e,e,e,8c,16g,e,8c,16g,2e,b,b,b,8c5,16g,8d#5,16g,2e",
        ),
        (
            "Super Mario",
            "Mario:d=4,o=5,b=100:16e6,16e6,32p,8e6,16c6,8e6,8g6,8p,8g,8p",
        ),
        (
            "Tetris",
            "Tetris:d=4,o=5,b=140:e,8b4,8c,8d,16e,16d,8c,8b4,a4,8a4,8c,e,8d,8c,b4,8b4,8c,d,e,c,a4,2a4",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtttl_parser_nokia() {
        let rtttl = "Nokia:d=4,o=5,b=225:e6,d6,f#,g#";
        let melody = RtttlMelody::parse(rtttl).unwrap();
        assert_eq!(melody.name, "Nokia");
        assert_eq!(melody.notes.len(), 4);
        assert!((melody.notes[0].frequency - 1318.51).abs() < 1.0);
    }

    #[test]
    fn test_rtttl_player_stepping() {
        let melody = RtttlMelody::parse("Test:d=4,o=4,b=120:a,p,c").unwrap();
        let mut player = RtttlPlayer::new(melody);
        player.play();

        let freq1 = player.advance_ms(10.0);
        assert!(freq1.is_some());
        assert!((freq1.unwrap() - 440.0).abs() < 1.0);

        player.advance_ms(500.0);
        let freq_pause = player.advance_ms(10.0);
        assert!(freq_pause.is_none());
    }
}

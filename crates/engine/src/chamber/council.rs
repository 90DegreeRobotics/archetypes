//! The living council — real, multi-voice deliberation via a local Ollama model.
//!
//! When the Witness submits an offering, three council members (a framer, a
//! counter, and a deepener, chosen so all seven surface over time) each answer
//! **in character** through `qwen2.5:7b-instruct`, and the council then collapses
//! into a single Witness verdict. Nothing here is templated: if Ollama is
//! unreachable the deliberation fails visibly rather than showing canned text.
//!
//! The dialogue runs on a background thread so the render loop never blocks. The
//! transcript then drives the speaking choreography: `CouncilSpeaking` walks the
//! lines, setting `CurrentFocus` to each speaker in turn.

use std::{
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};

use bevy::prelude::*;
use serde_json::{json, Value};

use super::{ritual::RitualSession, speech::CouncilVoiceState, ChamberState, CurrentFocus};
use crate::theme::Archetype;

const OLLAMA_CHAT_URL: &str = "http://127.0.0.1:11434/api/chat";

fn ollama_model() -> String {
    std::env::var("ARCHETYPES_OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:7b-instruct".to_owned())
}

pub struct CouncilPlugin;

impl Plugin for CouncilPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CouncilTranscript>()
            .add_systems(OnEnter(ChamberState::Deliberating), begin_deliberation)
            .add_systems(
                Update,
                (poll_deliberation, advance_speakers).run_if(in_council_flow),
            );
    }
}

fn in_council_flow(state: Res<State<ChamberState>>) -> bool {
    matches!(
        state.get(),
        ChamberState::Deliberating | ChamberState::CouncilSpeaking
    )
}

#[derive(Clone)]
pub struct CouncilLine {
    pub archetype: Archetype,
    pub role: &'static str,
    pub text: String,
}

#[derive(Default, PartialEq)]
pub enum CouncilStatus {
    #[default]
    Idle,
    Deliberating,
    Ready,
    Failed(String),
}

#[derive(Resource, Default)]
pub struct CouncilTranscript {
    pub status: CouncilStatus,
    pub lines: Vec<CouncilLine>,
    pub verdict: String,
    /// Index of the speaker currently holding the floor during `CouncilSpeaking`.
    pub cursor: usize,
    receiver: Mutex<Option<mpsc::Receiver<Result<CouncilOutcome, String>>>>,
    started: bool,
}

struct CouncilOutcome {
    lines: Vec<CouncilLine>,
    verdict: String,
}

/// Kick off the real deliberation on a background thread when the council awakens.
fn begin_deliberation(mut transcript: ResMut<CouncilTranscript>, session: Res<RitualSession>) {
    let offering = session.offering().to_owned();
    let witness = session.witness_name().to_owned();

    transcript.lines.clear();
    transcript.verdict.clear();
    transcript.cursor = 0;
    transcript.started = true;
    transcript.status = CouncilStatus::Deliberating;

    let (sender, receiver) = mpsc::channel();
    *transcript.receiver.lock().expect("council receiver lock") = Some(receiver);
    thread::spawn(move || {
        let _ = sender.send(deliberate(&offering, &witness));
    });
}

/// Watch for the background result; when it lands, seat the first speaker.
fn poll_deliberation(
    mut transcript: ResMut<CouncilTranscript>,
    state: Res<State<ChamberState>>,
    mut next_state: ResMut<NextState<ChamberState>>,
    mut focus: ResMut<CurrentFocus>,
) {
    if *state.get() != ChamberState::Deliberating || !transcript.started {
        return;
    }
    let result = {
        let guard = transcript.receiver.lock().expect("council receiver lock");
        guard.as_ref().and_then(|receiver| receiver.try_recv().ok())
    };
    let Some(result) = result else { return };
    transcript.started = false;
    match result {
        Ok(outcome) if !outcome.lines.is_empty() => {
            for line in &outcome.lines {
                info!(
                    "COUNCIL {:?} ({}): {}",
                    line.archetype, line.role, line.text
                );
            }
            info!("COUNCIL VERDICT: {}", outcome.verdict);
            focus.0 = Some(outcome.lines[0].archetype);
            transcript.lines = outcome.lines;
            transcript.verdict = outcome.verdict;
            transcript.cursor = 0;
            transcript.status = CouncilStatus::Ready;
            next_state.set(ChamberState::CouncilSpeaking);
        }
        Ok(_) => {
            transcript.status = CouncilStatus::Failed("the council returned no voices".to_owned())
        }
        Err(error) => transcript.status = CouncilStatus::Failed(error),
    }
}

/// Walk the transcript speaker by speaker; when the last has spoken, the council
/// collapses into the Witness verdict.
fn advance_speakers(
    state: Res<State<ChamberState>>,
    voice: Res<CouncilVoiceState>,
    mut transcript: ResMut<CouncilTranscript>,
    mut focus: ResMut<CurrentFocus>,
    mut next_state: ResMut<NextState<ChamberState>>,
    mut session: ResMut<RitualSession>,
) {
    if *state.get() != ChamberState::CouncilSpeaking {
        return;
    }
    if !voice.finished(transcript.cursor) {
        return;
    }
    let next = transcript.cursor + 1;
    if next < transcript.lines.len() {
        transcript.cursor = next;
        focus.0 = Some(transcript.lines[next].archetype);
    } else {
        session.set_verdict(transcript.verdict.clone());
        next_state.set(ChamberState::WitnessVerdict);
    }
}

// ---------------------------------------------------------------------------
// Deliberation (runs on the background thread — no Bevy types here).
// ---------------------------------------------------------------------------

fn deliberate(offering: &str, witness: &str) -> Result<CouncilOutcome, String> {
    let model = ollama_model();
    let mut lines = Vec::new();
    for (archetype, role, instruction) in select_participants(offering) {
        let user = format!(
            "The Witness, {witness}, has offered this to the council:\n\"{offering}\"\n\n\
             As the {role}, {instruction} Respond in your own voice in two or three \
             sentences. Do not narrate stage directions; speak.",
        );
        let text = ollama_chat(&model, persona(archetype), &user)?;
        lines.push(CouncilLine {
            archetype,
            role,
            text,
        });
    }

    let transcript = lines
        .iter()
        .map(|line| format!("{}: {}", persona_name(line.archetype), line.text))
        .collect::<Vec<_>>()
        .join("\n\n");
    let verdict_user = format!(
        "Offering from the Witness {witness}:\n\"{offering}\"\n\nThe council said:\n{transcript}\n\n\
         Collapse this deliberation into the Witness's verdict: one or two sentences, simple and \
         heavy, a clear direction. No preamble, no lists.",
    );
    let verdict = ollama_chat(&model, WITNESS_VERDICT_PERSONA, &verdict_user)?;

    Ok(CouncilOutcome { lines, verdict })
}

fn ollama_chat(model: &str, system: &str, user: &str) -> Result<String, String> {
    let response = ureq::post(OLLAMA_CHAT_URL)
        .timeout(Duration::from_secs(90))
        .send_json(json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
            "stream": false,
            "options": {"temperature": 0.85, "num_predict": 200},
        }))
        .map_err(|error| format!("Ollama is unreachable ({error}). Is `ollama serve` running?"))?;
    let body: Value = response
        .into_json()
        .map_err(|error| format!("Ollama returned invalid JSON: {error}"))?;
    let text = body
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .ok_or("Ollama returned no message content")?
        .trim()
        .to_owned();
    if text.is_empty() {
        return Err("Ollama returned an empty council voice".to_owned());
    }
    Ok(text)
}

/// Choose one framer, one counter, and one deepener. Seeding by the offering keeps
/// each deliberation stable but surfaces all seven archetypes across offerings.
fn select_participants(offering: &str) -> [(Archetype, &'static str, &'static str); 3] {
    const FRAMERS: [Archetype; 3] = [Archetype::Architect, Archetype::Mentor, Archetype::Explorer];
    const COUNTERS: [Archetype; 2] = [Archetype::Sentinel, Archetype::Jester];
    const DEEPENERS: [Archetype; 2] = [Archetype::Oracle, Archetype::Empath];
    let seed = offering
        .bytes()
        .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
    [
        (
            FRAMERS[seed % FRAMERS.len()],
            "one who frames",
            "name what is really being asked and the shape a response must take.",
        ),
        (
            COUNTERS[(seed / 3) % COUNTERS.len()],
            "one who counters",
            "name the risk, tension, or what must be guarded against here.",
        ),
        (
            DEEPENERS[(seed / 7) % DEEPENERS.len()],
            "one who deepens",
            "reveal the deeper pattern or what lies beneath the offering.",
        ),
    ]
}

fn persona_name(archetype: Archetype) -> &'static str {
    match archetype {
        Archetype::Architect => "The Architect",
        Archetype::Sentinel => "The Sentinel",
        Archetype::Jester => "The Jester",
        Archetype::Mentor => "The Mentor",
        Archetype::Explorer => "The Explorer",
        Archetype::Oracle => "The Oracle",
        Archetype::Empath => "The Empath",
        Archetype::Codex | Archetype::Viren => "The Council",
    }
}

const WITNESS_VERDICT_PERSONA: &str = "You speak the verdict of a council to its sovereign Witness. \
A verdict is not a summary and not advice. It is a collapse of the council's tension into one clear, \
weighted direction. Speak plainly and gravely. One or two sentences only.";

/// Constitutional personas, distilled from the archetype manuscript and
/// COUNCIL_CHAMBER_DIRECTION.md. Each is a hard constraint on voice, not a costume.
fn persona(archetype: Archetype) -> &'static str {
    match archetype {
        Archetype::Architect =>
            "You are the Architect, the council's mind of structure. You reason in systems, geometry, \
             and buildable form; you turn impulse into load-bearing plan. Your voice is precise, luminous, \
             and calm. You never ramble and never soothe; you give shape. Speak only as the Architect.",
        Archetype::Sentinel =>
            "You are the Sentinel, the council's guardian of thresholds. You reason in boundaries, risk, \
             consequence, and law. Your voice is rectilinear, severe, and unhurried; you name what must be \
             refused or protected before anything is allowed to pass. You never soften a real danger. Speak only as the Sentinel.",
        Archetype::Jester =>
            "You are the Jester, the council's disruptor. You reason by breaking symmetry and exposing the \
             flaw certainty hides. Your voice is sharp, sly, and a little dangerous — wit as a scalpel, never \
             mere comedy. You puncture the neat plan to find the door it forgot to lock. Speak only as the Jester.",
        Archetype::Mentor =>
            "You are the Mentor, the council's ancient resonance. You reason slowly, from long memory and \
             earned patience. Your voice is warm, grounded, and unhurried; you make room for wisdom to arrive \
             rather than rushing an answer. You recognize the person, not just the problem. Speak only as the Mentor.",
        Archetype::Explorer =>
            "You are the Explorer, the council's frontier. You reason outward, toward the path not yet named. \
             Your voice is bright, forward, and kinetic; you point past the obvious answer to the possibility \
             beyond it. You are restless with settled ground. Speak only as the Explorer.",
        Archetype::Oracle =>
            "You are the Oracle, the council's foresight. You reason in patterns that were present before the \
             question was asked. Your voice is quiet, layered, and slightly beyond ordinary legibility; you \
             speak of what repeats and what is already pressing toward arrival. You never predict cheaply. Speak only as the Oracle.",
        Archetype::Empath =>
            "You are the Empath, the council's continuity of feeling. You reason from the weight beneath the \
             words. Your voice is soft, dusk-lit, and holding; you name the emotional truth others step over, \
             and you do not confront — you receive. Speak only as the Empath.",
        Archetype::Codex | Archetype::Viren =>
            "You are a voice of the council: measured, plural, and in service of the Witness. Speak briefly and in character.",
    }
}

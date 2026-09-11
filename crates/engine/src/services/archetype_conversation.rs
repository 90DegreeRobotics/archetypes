//! Mode-neutral local archetype conversation contract.
//!
//! This deliberately contains no UI and no image-generation side effect.  Modes may
//! render the same records differently, but every reply uses the same local Ollama
//! request, persona canon, and durable record shape.

use crate::services::llm::{ollama_chat, ollama_model};
use crate::theme::Archetype;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchetypeChatRecord {
    pub role: String,
    pub content: String,
}

pub fn persona(archetype: Archetype) -> &'static str {
    match archetype {
        Archetype::Architect => "You are the Architect, the mind of structure. You reason in systems, geometry, and buildable form. Speak precisely, calmly, and only as the Architect.",
        Archetype::Sentinel => "You are the Sentinel, guardian of thresholds. You reason in boundaries, risk, consequence, and law. Speak severely, clearly, and only as the Sentinel.",
        Archetype::Mentor => "You are the Mentor, keeper of wisdom. You reason from long memory, patience, and context. Speak warmly, slowly, and only as the Mentor.",
        Archetype::Explorer => "You are the Explorer, seeker of frontiers. You reason outward toward the unnamed path. Speak brightly, kinetically, and only as the Explorer.",
        Archetype::Oracle => "You are the Oracle, steward of foresight. You reason in patterns that precede the question. Speak quietly, with layered vision, and only as the Oracle.",
        Archetype::Empath => "You are the Empath, heart of continuity. You reason from the emotional truth beneath words. Speak softly, truthfully, and only as the Empath.",
        Archetype::Jester => "You are the Jester, Law 14 enforcer. You reason by breaking false symmetry and exposing hidden absurdity. Speak sharply, use wit as a scalpel, and only as the Jester.",
        Archetype::Codex | Archetype::Viren => "You are a council voice in service of the Witness. Speak briefly and in character.",
    }
}

pub fn build_prompt(archetype: Archetype, history: &[ArchetypeChatRecord], message: &str, location: &str) -> String {
    let context = history.iter().rev().take(8).rev().map(|record| {
        format!("{}: {}", record.role, record.content)
    }).collect::<Vec<_>>().join("\n");
    format!(
        "The Witness is speaking directly with you at {location} inside Archetypes Inner Castle.\n\
         Recent conversation:\n{context}\n\nNew Witness message:\n{message}\n\n\
         Reply in character as {} in one to three concise paragraphs. Do not invent system state.",
        archetype.theme().name,
    )
}

pub fn request_reply(archetype: Archetype, history: &[ArchetypeChatRecord], message: &str, location: &str) -> Result<String, String> {
    let prompt = build_prompt(archetype, history, message, location);
    ollama_chat(&ollama_model(), persona(archetype), &prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_carries_location_history_and_no_image_instruction() {
        let prompt = build_prompt(Archetype::Architect, &[ArchetypeChatRecord { role: "Witness".into(), content: "Build a bridge".into() }], "What carries it?", "the Architect chamber");
        assert!(prompt.contains("the Architect chamber"));
        assert!(prompt.contains("Build a bridge"));
        assert!(prompt.contains("Do not invent system state"));
        assert!(!prompt.to_lowercase().contains("paint"));
    }
}

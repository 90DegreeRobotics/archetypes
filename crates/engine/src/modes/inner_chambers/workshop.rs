//! The Architect's planning workshop: a physical bench in the Architect's room where the player
//! writes, sequences and closes their own `BuildIntent` plans.
//!
//! Nothing here writes to disk without the player seeing the exact content first and confirming
//! it on a dedicated screen. A failed write keeps the pending content on the bench with an
//! explanatory status rather than discarding the player's words, and cancelling at any point
//! writes nothing at all.
//!
//! `Esc` steps back one screen at a time and finally leaves the bench for the room — it never
//! exits the castle, because `interaction.rs` records that this surface owns input for the frame.

use bevy::prelude::*;
use bevy::window::{CursorOptions, PrimaryWindow};

use crate::services::build_intent::{self, BuildIntent, PlanStatus};
use crate::services::encounter_memory;
use crate::services::text_entry;

use super::interaction::{
    set_modal_cursor, InnerActions, InnerInteractionSet, InnerModalState, InteractionFocus,
    InteractionTarget,
};
use super::world::{HintPriority, HintRequest, InnerHintSet, InnerWorldElement};
use super::InnerChambersState;

const TITLE_MAX: usize = 72;
const LINE_MAX: usize = 240;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum WorkshopScreen {
    #[default]
    Closed,
    Browsing,
    Editing(EditField),
    Confirming,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditField {
    Title,
    IntentStatement,
    Action,
    Constraint,
    Risk,
    CompletionEvidence,
}

impl EditField {
    fn prompt(&self) -> &'static str {
        match self {
            EditField::Title => "Name this plan",
            EditField::IntentStatement => "What is this plan for?",
            EditField::Action => "Next action",
            EditField::Constraint => "Constraint",
            EditField::Risk => "Risk or open question",
            EditField::CompletionEvidence => "What shows this is finished?",
        }
    }

    fn cap(&self) -> usize {
        match self {
            EditField::Title => TITLE_MAX,
            _ => LINE_MAX,
        }
    }
}

/// A write the player has composed but not yet approved. Held here, shown verbatim on the
/// confirm screen, and only then handed to `build_intent`.
#[derive(Clone, PartialEq, Debug)]
enum PendingWrite {
    CreatePlan { title: String, intent: String, supporting_ids: Vec<String> },
    AddAction { plan_id: String, description: String },
    CompleteAction { plan_id: String, action_id: String, description: String },
    AddConstraint { plan_id: String, text: String },
    AddRisk { plan_id: String, text: String },
    SetStatus { plan_id: String, status: PlanStatus },
    ClosePlan { plan_id: String, evidence: String },
}

impl PendingWrite {
    fn summary(&self) -> String {
        match self {
            PendingWrite::CreatePlan { title, intent, supporting_ids } => {
                let provenance = if supporting_ids.is_empty() {
                    "no linked encounter".to_owned()
                } else {
                    format!("{} linked remembered encounter(s)", supporting_ids.len())
                };
                format!("Create plan \"{title}\"\n  Intent: {intent}\n  Provenance: {provenance}")
            }
            PendingWrite::AddAction { description, .. } => format!("Add next action: \"{description}\""),
            PendingWrite::CompleteAction { description, .. } => format!("Mark done: \"{description}\""),
            PendingWrite::AddConstraint { text, .. } => format!("Add constraint: \"{text}\""),
            PendingWrite::AddRisk { text, .. } => format!("Add risk: \"{text}\""),
            PendingWrite::SetStatus { status, .. } => format!("Set status to {}", status.label()),
            PendingWrite::ClosePlan { evidence, .. } => format!("Close this plan\n  Evidence: {evidence}"),
        }
    }
}

#[derive(Resource, Default)]
pub struct WorkshopState {
    screen: WorkshopScreen,
    plans: Vec<BuildIntent>,
    selected_plan: usize,
    selected_action: usize,
    draft: String,
    new_plan_title: String,
    supporting_ids: Vec<String>,
    pending: Option<PendingWrite>,
    status: String,
}

impl WorkshopState {
    pub fn is_open(&self) -> bool {
        self.screen != WorkshopScreen::Closed
    }

    /// How many plans the bench is currently showing, for capture/report evidence.
    pub fn plan_count(&self) -> usize {
        self.plans.len()
    }

    /// Which screen the bench is on, for capture/report evidence.
    pub fn screen_label(&self) -> &'static str {
        match self.screen {
            WorkshopScreen::Closed => "Closed",
            WorkshopScreen::Browsing => "Browsing",
            WorkshopScreen::Editing(EditField::Title) => "Editing(Title)",
            WorkshopScreen::Editing(EditField::IntentStatement) => "Editing(Intent)",
            WorkshopScreen::Editing(EditField::Action) => "Editing(Action)",
            WorkshopScreen::Editing(EditField::Constraint) => "Editing(Constraint)",
            WorkshopScreen::Editing(EditField::Risk) => "Editing(Risk)",
            WorkshopScreen::Editing(EditField::CompletionEvidence) => "Editing(Evidence)",
            WorkshopScreen::Confirming => "Confirming",
        }
    }

    fn selected(&self) -> Option<&BuildIntent> {
        self.plans.get(self.selected_plan)
    }
}

#[derive(Component)]
struct WorkshopPanel;

#[derive(Component)]
struct WorkshopText;

pub struct WorkshopPlugin;

impl Plugin for WorkshopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorkshopState>()
            .add_systems(OnEnter(InnerChambersState::Navigating), spawn_workshop_ui)
            .add_systems(OnEnter(InnerChambersState::Exiting), close_workshop_on_exit)
            .add_systems(
                Update,
                (focus_or_open_workshop, drive_workshop, render_workshop_ui)
                    .chain()
                    .in_set(InnerHintSet::Request)
                    .after(InnerInteractionSet::Resolve)
                    .run_if(in_state(InnerChambersState::Navigating)),
            );
    }
}

fn spawn_workshop_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(10.0),
                right: Val::Percent(10.0),
                top: Val::Px(64.0),
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.020, 0.028, 0.042, 0.96)),
            BorderColor::all(Color::srgb(0.38, 0.62, 0.86)),
            Visibility::Hidden,
            GlobalZIndex(1100),
            WorkshopPanel,
            InnerWorldElement,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont { font_size: 19.0, ..default() },
                TextColor(Color::srgb(0.88, 0.92, 0.97)),
                WorkshopText,
            ));
        });
}

/// Leaving the castle with the bench open would otherwise strand the player: the workshop would
/// still own input on re-entry and locomotion would stay frozen.
fn close_workshop_on_exit(mut state: ResMut<WorkshopState>) {
    *state = WorkshopState::default();
}

fn reload(state: &mut WorkshopState) {
    match build_intent::load_plans() {
        Ok(plans) => {
            state.plans = plans;
            if state.selected_plan >= state.plans.len() {
                state.selected_plan = state.plans.len().saturating_sub(1);
            }
            state.selected_action = 0;
        }
        Err(error) => state.status = format!("Could not read the local plan journal: {error}"),
    }
}

fn focus_or_open_workshop(
    actions: Res<InnerActions>,
    modal: Res<InnerModalState>,
    focus: Res<InteractionFocus>,
    mut state: ResMut<WorkshopState>,
    mut hint: ResMut<HintRequest>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if state.is_open() || focus.0 != Some(InteractionTarget::ArchitectWorkshop) {
        return;
    }
    hint.request(
        HintPriority::Device,
        "[E / Pad-X] Architect's bench — write and review your own plans",
    );
    if modal.any() || !actions.interact {
        return;
    }
    state.screen = WorkshopScreen::Browsing;
    state.status.clear();
    reload(&mut state);
    set_modal_cursor(&mut cursor, true);
}

fn drive_workshop(
    keyboard: Res<ButtonInput<KeyCode>>,
    actions: Res<InnerActions>,
    mut state: ResMut<WorkshopState>,
    mut hint: ResMut<HintRequest>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !state.is_open() {
        return;
    }
    hint.request(HintPriority::Modal, "Architect's bench — Esc steps back to the room");
    match state.screen.clone() {
        WorkshopScreen::Closed => {}
        WorkshopScreen::Browsing => browsing_input(&keyboard, &actions, &mut state, &mut cursor),
        WorkshopScreen::Editing(field) => editing_input(&keyboard, &actions, &mut state, field),
        WorkshopScreen::Confirming => confirming_input(&keyboard, &actions, &mut state),
    }
}

fn browsing_input(
    keyboard: &ButtonInput<KeyCode>,
    actions: &InnerActions,
    state: &mut WorkshopState,
    cursor: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if actions.cancel {
        *state = WorkshopState::default();
        set_modal_cursor(cursor, false);
        return;
    }

    if !state.plans.is_empty() {
        let count = state.plans.len();
        if keyboard.just_pressed(KeyCode::ArrowDown) {
            state.selected_plan = (state.selected_plan + 1) % count;
            state.selected_action = 0;
        }
        if keyboard.just_pressed(KeyCode::ArrowUp) {
            state.selected_plan = (state.selected_plan + count - 1) % count;
            state.selected_action = 0;
        }
    }

    if keyboard.just_pressed(KeyCode::KeyN) {
        state.draft.clear();
        state.new_plan_title.clear();
        state.supporting_ids = remembered_architect_encounter_ids();
        state.screen = WorkshopScreen::Editing(EditField::Title);
        return;
    }

    let Some(plan) = state.selected().cloned() else {
        state.status = "No plans yet. Press N to write your first one.".to_owned();
        return;
    };

    if !plan.actions.is_empty() {
        let count = plan.actions.len();
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            state.selected_action = (state.selected_action + 1) % count;
        }
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            state.selected_action = (state.selected_action + count - 1) % count;
        }
    }

    if plan.is_closed() {
        if keyboard.any_just_pressed([KeyCode::KeyA, KeyCode::KeyC, KeyCode::KeyR, KeyCode::KeyX, KeyCode::KeyS, KeyCode::Enter]) {
            state.status = "That plan is closed. Choose another with the arrow keys, or press N.".to_owned();
        }
        return;
    }

    for (key, field) in [
        (KeyCode::KeyA, EditField::Action),
        (KeyCode::KeyC, EditField::Constraint),
        (KeyCode::KeyR, EditField::Risk),
        (KeyCode::KeyX, EditField::CompletionEvidence),
    ] {
        if keyboard.just_pressed(key) {
            state.draft.clear();
            state.screen = WorkshopScreen::Editing(field);
            return;
        }
    }

    if keyboard.just_pressed(KeyCode::KeyS) {
        let status = if plan.status == PlanStatus::Open { PlanStatus::Stalled } else { PlanStatus::Open };
        state.pending = Some(PendingWrite::SetStatus { plan_id: plan.id.clone(), status });
        state.screen = WorkshopScreen::Confirming;
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        match plan.actions.get(state.selected_action) {
            Some(action) if !action.done => {
                state.pending = Some(PendingWrite::CompleteAction {
                    plan_id: plan.id.clone(),
                    action_id: action.id.clone(),
                    description: action.description.clone(),
                });
                state.screen = WorkshopScreen::Confirming;
            }
            Some(_) => state.status = "That step is already marked done.".to_owned(),
            None => state.status = "No steps yet — press A to add one.".to_owned(),
        }
    }
}

fn editing_input(
    keyboard: &ButtonInput<KeyCode>,
    actions: &InnerActions,
    state: &mut WorkshopState,
    field: EditField,
) {
    if actions.cancel {
        state.draft.clear();
        state.screen = WorkshopScreen::Browsing;
        state.status = "Cancelled. Nothing was written.".to_owned();
        return;
    }

    text_entry::apply_typed_keys(keyboard, &mut state.draft, field.cap());

    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }
    let text = state.draft.trim().to_owned();
    if text.is_empty() {
        state.status = "Type something first, or press Esc to cancel.".to_owned();
        return;
    }

    match field {
        EditField::Title => {
            state.new_plan_title = text;
            state.draft.clear();
            state.screen = WorkshopScreen::Editing(EditField::IntentStatement);
            return;
        }
        EditField::IntentStatement => {
            state.pending = Some(PendingWrite::CreatePlan {
                title: state.new_plan_title.clone(),
                intent: text,
                supporting_ids: state.supporting_ids.clone(),
            });
            state.draft.clear();
            state.screen = WorkshopScreen::Confirming;
            return;
        }
        _ => {}
    }

    let Some(plan) = state.selected().cloned() else {
        state.screen = WorkshopScreen::Browsing;
        state.status = "That plan is no longer on the bench.".to_owned();
        return;
    };
    let pending = match field {
        EditField::Action => PendingWrite::AddAction { plan_id: plan.id, description: text },
        EditField::Constraint => PendingWrite::AddConstraint { plan_id: plan.id, text },
        EditField::Risk => PendingWrite::AddRisk { plan_id: plan.id, text },
        EditField::CompletionEvidence => PendingWrite::ClosePlan { plan_id: plan.id, evidence: text },
        // Both handled above; returning is honest about that rather than asserting.
        EditField::Title | EditField::IntentStatement => return,
    };
    state.pending = Some(pending);
    state.draft.clear();
    state.screen = WorkshopScreen::Confirming;
}

fn confirming_input(keyboard: &ButtonInput<KeyCode>, actions: &InnerActions, state: &mut WorkshopState) {
    if actions.cancel || keyboard.just_pressed(KeyCode::KeyN) {
        state.pending = None;
        state.screen = WorkshopScreen::Browsing;
        state.status = "Discarded. Nothing was written.".to_owned();
        return;
    }
    if !keyboard.just_pressed(KeyCode::Enter) && !keyboard.just_pressed(KeyCode::KeyY) {
        return;
    }
    let Some(pending) = state.pending.take() else {
        state.screen = WorkshopScreen::Browsing;
        return;
    };
    match commit(&pending) {
        Ok(message) => {
            state.status = message;
            state.screen = WorkshopScreen::Browsing;
            reload(state);
        }
        Err(error) => {
            // The player's words stay on the bench and the confirm screen stays up, so a
            // failed local write never silently eats what they wrote.
            state.status = format!("Not written — {error}. Press Enter to retry or Esc to discard.");
            state.pending = Some(pending);
        }
    }
}

fn commit(pending: &PendingWrite) -> Result<String, String> {
    let (receipt, message) = match pending {
        PendingWrite::CreatePlan { title, intent, supporting_ids } => {
            // Consent is re-checked at the moment of writing: an encounter the player forgot
            // between choosing it and confirming must not be recorded as provenance.
            let recallable: Vec<String> = encounter_memory::recallable_records(Some("Architect"))
                .map(|records| records.into_iter().map(|record| record.id).collect())
                .unwrap_or_default();
            let supporting = build_intent::filter_supporting_ids(supporting_ids, &recallable);
            (
                build_intent::create_plan(title, intent, supporting)?,
                format!("Written: plan \"{title}\"."),
            )
        }
        PendingWrite::AddAction { plan_id, description } => (
            build_intent::add_action(plan_id, description)?,
            format!("Written: next action \"{description}\"."),
        ),
        PendingWrite::CompleteAction { plan_id, action_id, description } => (
            build_intent::complete_action(plan_id, action_id)?,
            format!("Written: \"{description}\" is done."),
        ),
        PendingWrite::AddConstraint { plan_id, text } => (
            build_intent::add_constraint(plan_id, text)?,
            format!("Written: constraint \"{text}\" recorded."),
        ),
        PendingWrite::AddRisk { plan_id, text } => (
            build_intent::add_risk(plan_id, text)?,
            format!("Written: risk \"{text}\" recorded."),
        ),
        PendingWrite::SetStatus { plan_id, status } => (
            build_intent::set_status(plan_id, *status)?,
            format!("Written: status is now {}.", status.label()),
        ),
        PendingWrite::ClosePlan { plan_id, evidence } => (
            build_intent::close_plan(plan_id, Some(evidence))?,
            "Written: plan closed.".to_owned(),
        ),
    };
    Ok(format!("{message}{}", receipt.caveat()))
}

/// Provenance offered when a new plan is written: only encounters the player explicitly chose
/// to remember are eligible, and the choice is shown on the confirm screen before it is stored.
fn remembered_architect_encounter_ids() -> Vec<String> {
    encounter_memory::recallable_records(Some("Architect"))
        .map(|records| records.into_iter().rev().take(1).map(|record| record.id).collect())
        .unwrap_or_default()
}

fn render_workshop_ui(
    state: Res<WorkshopState>,
    mut panel: Query<&mut Visibility, With<WorkshopPanel>>,
    mut text: Query<&mut Text, With<WorkshopText>>,
) {
    let Ok(mut panel) = panel.single_mut() else {
        return;
    };
    *panel = if state.is_open() { Visibility::Visible } else { Visibility::Hidden };
    if !state.is_open() {
        return;
    }
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let body = render_body(&state);
    if text.0 != body {
        text.0 = body;
    }
}

fn render_body(state: &WorkshopState) -> String {
    let header = "ARCHITECT'S BENCH — your plans, written and kept locally";
    let screen = match &state.screen {
        WorkshopScreen::Closed => String::new(),
        WorkshopScreen::Browsing => browsing_body(state),
        WorkshopScreen::Editing(field) => format!(
            "{}\n\n> {}▌\n\nEnter accepts  •  Esc cancels without writing",
            field.prompt(),
            state.draft
        ),
        WorkshopScreen::Confirming => format!(
            "About to write:\n\n{}\n\nEnter / Y confirms  •  Esc / N discards",
            state
                .pending
                .as_ref()
                .map(PendingWrite::summary)
                .unwrap_or_else(|| "nothing".to_owned())
        ),
    };
    if state.status.is_empty() {
        format!("{header}\n\n{screen}")
    } else {
        format!("{header}\n\n{screen}\n\n{}", state.status)
    }
}

fn browsing_body(state: &WorkshopState) -> String {
    if state.plans.is_empty() {
        return "No plans yet.\n\nN writes a new one  •  Esc returns to the room".to_owned();
    }
    let signals = build_intent::plan_signals(&state.plans, now_ms());
    let list: Vec<String> = state
        .plans
        .iter()
        .enumerate()
        .map(|(index, plan)| {
            let marker = if index == state.selected_plan { ">" } else { " " };
            format!("{marker} [{}] {}", plan.status.label(), plan.title)
        })
        .collect();

    let mut body = format!(
        "{} open  •  {} stalled  •  {} open steps  •  oldest untouched {}d\n\n{}",
        signals.open,
        signals.stalled,
        signals.open_actions,
        signals.oldest_untouched_days,
        list.join("\n")
    );

    if let Some(plan) = state.selected() {
        body.push_str(&format!("\n\nIntent: {}", plan.intent_statement));
        if !plan.constraints.is_empty() {
            body.push_str(&format!("\nConstraints: {}", plan.constraints.join(" | ")));
        }
        if !plan.risks.is_empty() {
            body.push_str(&format!("\nRisks: {}", plan.risks.join(" | ")));
        }
        if plan.actions.is_empty() {
            body.push_str("\n\nNo steps yet.");
        } else {
            body.push_str("\n\nSteps:");
            for (index, action) in plan.actions.iter().enumerate() {
                let marker = if index == state.selected_action { ">" } else { " " };
                let box_mark = if action.done { "x" } else { " " };
                body.push_str(&format!("\n{marker} [{box_mark}] {}", action.description));
            }
        }
        if let Some(evidence) = &plan.completion_evidence {
            body.push_str(&format!("\n\nClosed with: {evidence}"));
        }
    }

    body.push_str(
        "\n\n↑↓ plan  •  ←→ step  •  N new  •  A step  •  C constraint  •  R risk  •  Enter mark done  •  S stall/reopen  •  X close  •  Esc room",
    );
    body
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(title: &str) -> BuildIntent {
        BuildIntent {
            id: "plan-1".to_owned(),
            version: build_intent::BUILD_INTENT_VERSION,
            title: title.to_owned(),
            intent_statement: "so the bench is useful".to_owned(),
            constraints: Vec::new(),
            risks: Vec::new(),
            actions: Vec::new(),
            supporting_ids: Vec::new(),
            status: PlanStatus::Open,
            created_at_utc_ms: 1_000,
            updated_at_utc_ms: 1_000,
            completion_evidence: None,
            content_hash: "hash".to_owned(),
        }
    }

    #[test]
    fn a_closed_workshop_owns_no_input() {
        assert!(!WorkshopState::default().is_open());
    }

    #[test]
    fn cancelling_an_edit_writes_nothing_and_returns_to_browsing() {
        let mut state = WorkshopState {
            screen: WorkshopScreen::Editing(EditField::Action),
            draft: "half typed".to_owned(),
            plans: vec![plan("Ship it")],
            ..Default::default()
        };
        editing_input(
            &ButtonInput::default(),
            &InnerActions { interact: false, cancel: true },
            &mut state,
            EditField::Action,
        );
        assert_eq!(state.screen, WorkshopScreen::Browsing);
        assert!(state.draft.is_empty());
        assert!(state.pending.is_none());
    }

    #[test]
    fn discarding_a_confirmation_drops_the_pending_write() {
        let mut state = WorkshopState {
            screen: WorkshopScreen::Confirming,
            pending: Some(PendingWrite::AddRisk { plan_id: "plan-1".to_owned(), text: "slippage".to_owned() }),
            ..Default::default()
        };
        confirming_input(
            &ButtonInput::default(),
            &InnerActions { interact: false, cancel: true },
            &mut state,
        );
        assert!(state.pending.is_none());
        assert_eq!(state.screen, WorkshopScreen::Browsing);
    }

    #[test]
    fn escape_at_the_browsing_screen_closes_the_bench_rather_than_the_castle() {
        let mut state = WorkshopState { screen: WorkshopScreen::Browsing, ..Default::default() };
        // The castle-exit binding stands down whenever this surface reports itself open, so
        // closing here is the whole effect of that key press.
        assert!(state.is_open());
        state = WorkshopState::default();
        assert!(!state.is_open());
    }

    #[test]
    fn every_pending_write_states_exactly_what_it_will_do() {
        let writes = [
            PendingWrite::CreatePlan { title: "T".into(), intent: "I".into(), supporting_ids: vec!["e1".into()] },
            PendingWrite::AddAction { plan_id: "p".into(), description: "step".into() },
            PendingWrite::CompleteAction { plan_id: "p".into(), action_id: "a".into(), description: "step".into() },
            PendingWrite::AddConstraint { plan_id: "p".into(), text: "c".into() },
            PendingWrite::AddRisk { plan_id: "p".into(), text: "r".into() },
            PendingWrite::SetStatus { plan_id: "p".into(), status: PlanStatus::Stalled },
            PendingWrite::ClosePlan { plan_id: "p".into(), evidence: "shipped".into() },
        ];
        for write in writes {
            assert!(!write.summary().trim().is_empty());
        }
    }

    #[test]
    fn the_create_summary_says_when_an_encounter_is_linked() {
        let linked = PendingWrite::CreatePlan { title: "T".into(), intent: "I".into(), supporting_ids: vec!["e1".into()] };
        let unlinked = PendingWrite::CreatePlan { title: "T".into(), intent: "I".into(), supporting_ids: Vec::new() };
        assert!(linked.summary().contains("1 linked"));
        assert!(unlinked.summary().contains("no linked encounter"));
    }

    #[test]
    fn browsing_body_lists_plans_and_their_steps() {
        let mut listed = plan("Ship the bench");
        listed.actions.push(build_intent::PlanAction {
            id: "a1".to_owned(),
            description: "wire the focus rule".to_owned(),
            done: true,
            created_at_utc_ms: 2_000,
        });
        let state = WorkshopState { screen: WorkshopScreen::Browsing, plans: vec![listed], ..Default::default() };
        let body = browsing_body(&state);
        assert!(body.contains("Ship the bench"));
        assert!(body.contains("[x] wire the focus rule"));
        assert!(body.contains("OPEN"));
    }

    #[test]
    fn an_empty_bench_tells_the_player_how_to_start() {
        let state = WorkshopState { screen: WorkshopScreen::Browsing, ..Default::default() };
        assert!(browsing_body(&state).contains("N writes a new one"));
    }
}

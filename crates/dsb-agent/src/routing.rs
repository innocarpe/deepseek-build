//! Flash / Pro model routing (spec 20).

use dsb_provider_deepseek::{MODEL_PRO, ModelId, ReasoningEffort, ThinkingMode};

/// Session preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Preset {
    /// Product default: Flash.
    #[default]
    Flash,
    /// Flash for tools; Pro for escalations.
    Balanced,
    /// Sticky Pro.
    Max,
}

impl Preset {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "flash" => Some(Self::Flash),
            "balanced" => Some(Self::Balanced),
            "max" => Some(Self::Max),
            _ => None,
        }
    }
}

/// One-shot or sticky model override for a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnModelOverride {
    /// One-shot Pro then return to default unless sticky max.
    ProOnce,
    /// Force Flash for this turn (beats router).
    ForceFlash,
    /// Force Pro for this turn (same as sticky if preset max).
    ForcePro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteSource {
    Default,
    StickyPreset,
    UserOverride,
    AutoRouter,
    FallbackUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDecision {
    pub model: ModelId,
    pub wire_model: String,
    pub thinking: ThinkingMode,
    pub effort: ReasoningEffort,
    pub source: RouteSource,
    pub escalate_reason: Option<String>,
    pub warning: Option<String>,
}

impl RouteDecision {
    pub fn visibility_line(&self) -> String {
        let thinking = match self.thinking.type_ {
            dsb_provider_deepseek::ThinkingType::Enabled => "on",
            dsb_provider_deepseek::ThinkingType::Disabled => "off",
        };
        let effort = match self.effort {
            ReasoningEffort::Low => "low",
            ReasoningEffort::High => "high",
            ReasoningEffort::Max => "max",
        };
        let mut s = format!(
            "model={} thinking={thinking} effort={effort}",
            self.wire_model
        );
        if let Some(r) = &self.escalate_reason {
            s.push_str(&format!(" escalate_reason={r}"));
        }
        if let Some(w) = &self.warning {
            s.push_str(&format!(" warning={w}"));
        }
        s
    }
}

#[derive(Debug, Clone)]
pub struct ModelRouter {
    preset: Preset,
    /// Pending one-shot Pro after `/pro`.
    pro_once: bool,
    /// User forced flash for next turn.
    force_flash_once: bool,
    /// Automatic router enabled (optional M1).
    auto_router: bool,
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new(Preset::Flash)
    }
}

impl ModelRouter {
    pub fn new(preset: Preset) -> Self {
        Self {
            preset,
            pro_once: false,
            force_flash_once: false,
            auto_router: true,
        }
    }

    pub fn preset(&self) -> Preset {
        self.preset
    }

    pub fn set_preset(&mut self, preset: Preset) {
        self.preset = preset;
        // Clear one-shots when sticky preset changes.
        self.pro_once = false;
        self.force_flash_once = false;
    }

    pub fn request_pro_once(&mut self) {
        self.pro_once = true;
        self.force_flash_once = false;
    }

    pub fn request_force_flash(&mut self) {
        self.force_flash_once = true;
        self.pro_once = false;
    }

    pub fn set_auto_router(&mut self, enabled: bool) {
        self.auto_router = enabled;
    }

    /// Select model for this turn. Consumes one-shot overrides.
    ///
    /// Precedence: explicit user override > sticky preset > auto router > default Flash.
    pub fn route_turn(&mut self, user_text: &str) -> RouteDecision {
        // User force flash
        if self.force_flash_once {
            self.force_flash_once = false;
            return Self::decision(ModelId::Flash, RouteSource::UserOverride, None, None);
        }

        // One-shot pro
        if self.pro_once {
            self.pro_once = false;
            return Self::decision(
                ModelId::Pro,
                RouteSource::UserOverride,
                Some("user_pro_once".into()),
                None,
            );
        }

        // Sticky preset
        match self.preset {
            Preset::Max => {
                return Self::decision(ModelId::Pro, RouteSource::StickyPreset, None, None);
            }
            Preset::Flash => {
                // fall through to auto router / default
            }
            Preset::Balanced => {
                // default flash unless auto escalates
            }
        }

        if self.auto_router
            && let Some(reason) = auto_escalate_reason(user_text)
        {
            // Only escalate if user did not force flash (already handled).
            return Self::decision(ModelId::Pro, RouteSource::AutoRouter, Some(reason), None);
        }

        let source = if self.preset == Preset::Flash {
            RouteSource::Default
        } else {
            RouteSource::StickyPreset
        };
        Self::decision(ModelId::Flash, source, None, None)
    }

    /// Fallback when Pro is unavailable (404/unsupported).
    pub fn fallback_flash(warning: impl Into<String>) -> RouteDecision {
        Self::decision(
            ModelId::Flash,
            RouteSource::FallbackUnavailable,
            None,
            Some(warning.into()),
        )
    }

    fn decision(
        model: ModelId,
        source: RouteSource,
        escalate_reason: Option<String>,
        warning: Option<String>,
    ) -> RouteDecision {
        let (thinking, effort) = match model {
            ModelId::Pro => (ThinkingMode::enabled(), ReasoningEffort::High),
            ModelId::Flash => (ThinkingMode::enabled(), ReasoningEffort::High),
            ModelId::Other(_) => (ThinkingMode::enabled(), ReasoningEffort::High),
        };
        // Preset max uses effort max — handled by caller for sticky; keep High default for flash/pro one-shot.
        let effort = if matches!(source, RouteSource::StickyPreset) && model.as_wire() == MODEL_PRO
        {
            ReasoningEffort::Max
        } else {
            effort
        };
        RouteDecision {
            wire_model: model.as_wire().to_string(),
            model,
            thinking,
            effort,
            source,
            escalate_reason,
            warning,
        }
    }

    /// Apply sticky max effort when preset is Max.
    pub fn route_turn_for_preset(&mut self, user_text: &str) -> RouteDecision {
        let mut d = self.route_turn(user_text);
        if self.preset == Preset::Max && d.model.as_wire() == MODEL_PRO {
            d.effort = ReasoningEffort::Max;
        }
        d
    }
}

fn auto_escalate_reason(user_text: &str) -> Option<String> {
    let lower = user_text.to_ascii_lowercase();
    const KEYWORDS: &[&str] = &[
        "architecture",
        "architect",
        "system design",
        "design the system",
        "refactor the whole",
        "migration plan",
        "deep review",
    ];
    for kw in KEYWORDS {
        if lower.contains(kw) {
            return Some(format!("keyword:{kw}"));
        }
    }
    None
}

/// Parse slash-style routing commands from a user line.
/// Returns (cleaned_user_text, optional override applied to router).
pub fn apply_routing_command(
    router: &mut ModelRouter,
    line: &str,
) -> (String, Option<&'static str>) {
    let trimmed = line.trim();
    if trimmed == "/pro" || trimmed.starts_with("/pro ") {
        router.request_pro_once();
        let rest = trimmed.trim_start_matches("/pro").trim().to_string();
        return (rest, Some("pro_once"));
    }
    if trimmed == "/flash" || trimmed.starts_with("/flash ") {
        router.request_force_flash();
        let rest = trimmed.trim_start_matches("/flash").trim().to_string();
        return (rest, Some("force_flash"));
    }
    if let Some(rest) = trimmed.strip_prefix("/preset ") {
        let name = rest.split_whitespace().next().unwrap_or("");
        if let Some(p) = Preset::parse(name) {
            router.set_preset(p);
            let after = rest[name.len()..].trim().to_string();
            return (after, Some("preset"));
        }
    }
    // /model is informational in the REPL (CLI prints visibility each turn).
    if trimmed == "/model" || trimmed.starts_with("/model ") {
        let rest = trimmed.trim_start_matches("/model").trim().to_string();
        return (rest, Some("model_status"));
    }
    (line.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsb_provider_deepseek::MODEL_FLASH;

    #[test]
    fn default_is_flash() {
        let mut r = ModelRouter::new(Preset::Flash);
        r.set_auto_router(false);
        let d = r.route_turn("hello");
        assert_eq!(d.wire_model, MODEL_FLASH);
        assert_eq!(d.source, RouteSource::Default);
    }

    #[test]
    fn pro_oneshot_then_flash() {
        let mut r = ModelRouter::new(Preset::Flash);
        r.set_auto_router(false);
        r.request_pro_once();
        let d1 = r.route_turn("x");
        assert_eq!(d1.wire_model, MODEL_PRO);
        let d2 = r.route_turn("y");
        assert_eq!(d2.wire_model, MODEL_FLASH);
    }

    #[test]
    fn preset_max_sticky() {
        let mut r = ModelRouter::new(Preset::Max);
        r.set_auto_router(false);
        let d1 = r.route_turn("a");
        let d2 = r.route_turn("b");
        assert_eq!(d1.wire_model, MODEL_PRO);
        assert_eq!(d2.wire_model, MODEL_PRO);
        assert_eq!(d1.effort, ReasoningEffort::Max);
    }

    #[test]
    fn user_beats_router() {
        let mut r = ModelRouter::new(Preset::Flash);
        r.set_auto_router(true);
        r.request_force_flash();
        let d = r.route_turn("please redesign the architecture");
        assert_eq!(d.wire_model, MODEL_FLASH);
        assert_eq!(d.source, RouteSource::UserOverride);
    }

    #[test]
    fn auto_router_escalates_architecture() {
        let mut r = ModelRouter::new(Preset::Flash);
        r.set_auto_router(true);
        let d = r.route_turn("help me with the architecture of this service");
        assert_eq!(d.wire_model, MODEL_PRO);
        assert_eq!(d.source, RouteSource::AutoRouter);
        assert!(d.escalate_reason.as_ref().unwrap().contains("architecture"));
    }

    #[test]
    fn pro_unavailable_fallback() {
        let d = ModelRouter::fallback_flash("pro 404");
        assert_eq!(d.wire_model, MODEL_FLASH);
        assert_eq!(d.source, RouteSource::FallbackUnavailable);
        assert!(d.warning.is_some());
    }

    #[test]
    fn visibility_includes_wire_model() {
        let mut r = ModelRouter::default();
        r.set_auto_router(false);
        let d = r.route_turn("hi");
        let line = d.visibility_line();
        assert!(line.contains(MODEL_FLASH));
        assert!(line.contains("thinking="));
        assert!(line.contains("effort="));
    }

    #[test]
    fn slash_pro_command() {
        let mut r = ModelRouter::default();
        let (text, cmd) = apply_routing_command(&mut r, "/pro explain this");
        assert_eq!(cmd, Some("pro_once"));
        assert_eq!(text, "explain this");
        let d = r.route_turn(&text);
        assert_eq!(d.wire_model, MODEL_PRO);
    }
}

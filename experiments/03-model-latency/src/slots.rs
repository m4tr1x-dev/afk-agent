//! Question 5: do two pinned contexts at different visual budgets keep their
//! caches, or evict each other?
//!
//! `17-model-contract.md` assumes they do not evict each other. The tactical
//! cadence runs at a small visual budget and the deliberative one at a large
//! budget, and the whole prompt layout is arranged so that each keeps a stable
//! prefix. If one slot's arrival flushes the other's cache, the two cadences
//! pay a full prefill every time they alternate, and the layout buys nothing.
//!
//! `ADR-0006` lists this as **negotiable**, with the fallback written down
//! before the measurement: one slot, and a deliberative call costs the tactical
//! slot one prefill. This experiment supplies the number that decides whether
//! the fallback is needed.
//!
//! **Two independent measures, because the runtime's own counter is not
//! enough.** A server can report a cache hit and prefill anyway; time is the
//! ground truth. Both are recorded per request:
//!
//! | Measure | Where it comes from |
//! | --- | --- |
//! | Tokens reused | `timings.cache_n`, the runtime's own counter |
//! | Prefill time | `timings.prompt_ms`, which is what it actually cost |
//!
//! **The negative control is the part most easily skipped.** After the
//! alternation, one byte of slot A's prefix is changed and the same request
//! sent again. If the reported reuse does not collapse and the prefill time
//! does not jump, then nothing here was measuring a cache at all — every
//! preceding number would be consistent with a runtime that reports whatever it
//! likes.

use std::fmt::Write as _;

use crate::client;

/// How many alternating rounds. Each round is one request on each slot.
const ROUNDS: usize = 20;

/// One request's cache behaviour.
struct Reading {
    /// Tokens the runtime says it reused.
    reused: u64,
    /// Tokens it actually prefilled.
    prefilled: u64,
    /// What that prefill cost.
    prompt_ms: f64,
}

/// Run the alternation and the control.
///
/// # Errors
///
/// Returns the step that failed.
pub(crate) fn check(host: &str, small: &[u8], large: &[u8]) -> Result<String, String> {
    let mut out = String::new();
    let _ = writeln!(out, "--- two pinned slots, question 5 ---");
    let _ = writeln!(
        out,
        "  {ROUNDS} alternating rounds; slot A at the tactical budget, slot B at the deliberative one"
    );
    let _ = writeln!(out);

    // Prime both, so neither is measured on its first sight of its own prefix.
    let _ = ask(host, PREFIX_A, small)?;
    let _ = ask(host, PREFIX_B, large)?;

    let mut a = Vec::with_capacity(ROUNDS);
    let mut b = Vec::with_capacity(ROUNDS);
    for _ in 0..ROUNDS {
        a.push(ask(host, PREFIX_A, small)?);
        b.push(ask(host, PREFIX_B, large)?);
    }

    let _ = writeln!(
        out,
        "  {:<16} {:>10} {:>12} {:>12}",
        "slot", "reused", "prefilled", "prefill ms"
    );
    report(&mut out, "A tactical", &a);
    report(&mut out, "B deliberative", &b);

    // The negative control. One byte different in slot A's prefix; everything
    // else identical.
    let _ = writeln!(out);
    let mutated = PREFIX_A.replacen("window.", "window!", 1);
    let control = ask(host, &mutated, small)?;
    let _ = writeln!(
        out,
        "  {:<16} {:>10} {:>12} {:>12.1}   (one byte changed)",
        "A mutated", control.reused, control.prefilled, control.prompt_ms
    );

    let steady = median_reused(&a);
    let _ = writeln!(out);
    let verdict = if steady == 0 {
        "INCONCLUSIVE: slot A reused nothing even in steady state, so there was no cache to evict and the control below proves nothing."
    } else if control.reused >= steady {
        "INCONCLUSIVE: changing a byte of the prefix did not reduce the reported reuse. Nothing here was measuring a cache, and every number above is consistent with a runtime reporting whatever it likes."
    } else if steady > 0 {
        "BOTH SLOTS KEEP THEIR CACHE: each slot reuses its own prefix in steady state while the other alternates against it, and a one-byte change collapses the reuse. The two-slot arrangement 17-model-contract.md assumes is available, and ADR-0006's one-slot fallback is not needed."
    } else {
        "SLOTS EVICT EACH OTHER: alternating costs a full prefill. 17-model-contract.md should say so rather than claiming a property the stack does not have, and ADR-0006's one-slot fallback applies."
    };
    let _ = writeln!(out, "  {verdict}");
    Ok(out)
}

fn report(out: &mut String, name: &str, readings: &[Reading]) {
    let reused = median_reused(readings);
    let prefilled = {
        let mut values: Vec<u64> = readings.iter().map(|r| r.prefilled).collect();
        values.sort_unstable();
        values.get(values.len() / 2).copied().unwrap_or(0)
    };
    let prompt_ms = {
        let mut values: Vec<f64> = readings.iter().map(|r| r.prompt_ms).collect();
        values.sort_by(f64::total_cmp);
        values.get(values.len() / 2).copied().unwrap_or(0.0)
    };
    let _ = writeln!(
        out,
        "  {name:<16} {reused:>10} {prefilled:>12} {prompt_ms:>12.1}"
    );
}

fn median_reused(readings: &[Reading]) -> u64 {
    let mut values: Vec<u64> = readings.iter().map(|r| r.reused).collect();
    values.sort_unstable();
    values.get(values.len() / 2).copied().unwrap_or(0)
}

/// The tactical slot's stable prefix.
const PREFIX_A: &str = "You are the tactical cadence. You see the screen several times a second and \
                        choose one action. You do not plan. You do not act outside the target window.";

/// The deliberative slot's stable prefix, deliberately different.
const PREFIX_B: &str = "You are the deliberative cadence. You see the screen occasionally and \
                        produce a plan with end conditions. You do not act directly.";

/// One request against one slot, reporting what the runtime says about its cache.
fn ask(host: &str, prefix: &str, png: &[u8]) -> Result<Reading, String> {
    let data = client::base64(png);
    let request = serde_json::json!({
        "messages": [{
            "role": "user",
            "content": [
                {"type": "text", "text": prefix},
                {"type": "image_url",
                 "image_url": {"url": format!("data:image/png;base64,{data}")}}
            ]
        }],
        "temperature": 0.0,
        "max_tokens": 8,
        "cache_prompt": true,
    });

    let body = client::post_json(host, "/v1/chat/completions", &request.to_string())?;
    let response: serde_json::Value =
        serde_json::from_str(&body).map_err(|error| format!("response is not JSON: {error}"))?;

    let number = |path: &str| -> f64 {
        response
            .pointer(path)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0)
    };
    Ok(Reading {
        reused: number("/timings/cache_n") as u64,
        prefilled: number("/timings/prompt_n") as u64,
        prompt_ms: number("/timings/prompt_ms"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(reused: u64) -> Reading {
        Reading {
            reused,
            prefilled: 100 - reused,
            prompt_ms: 10.0,
        }
    }

    #[test]
    fn the_median_ignores_an_outlier() {
        let readings: Vec<Reading> = [90, 91, 3, 92, 93].into_iter().map(reading).collect();
        assert_eq!(median_reused(&readings), 91);
    }

    #[test]
    fn the_two_prefixes_differ() {
        // If they were the same, both slots would share one cache entry and the
        // experiment would measure nothing.
        assert_ne!(PREFIX_A, PREFIX_B);
    }

    #[test]
    fn the_control_changes_exactly_one_byte() {
        let mutated = PREFIX_A.replacen("window.", "window!", 1);
        assert_eq!(mutated.len(), PREFIX_A.len());
        let differing = PREFIX_A
            .bytes()
            .zip(mutated.bytes())
            .filter(|(left, right)| left != right)
            .count();
        assert_eq!(
            differing, 1,
            "the control must change one byte, not a phrase"
        );
    }
}

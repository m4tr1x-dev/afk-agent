---
title: Sensor expressions
description: "The grammar for the predicates that postconditions and stuck detection are written in."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [reference, grounding]
requirements: []
decisions: []
generated: false
---

# Sensor expressions

A sensor is a named predicate over a region of the screen, evaluated on the graphics device every reflex tick without a model.

Sensors are what make checking affordable.
A postcondition written as an expression costs a fraction of a millisecond to evaluate, which is what allows every action to be verified rather than a sample of them.

This page is the grammar.
The design reasoning is in [grounding and verification](../spec/18-grounding-and-verification.md).

## Shape

```text
<name> = <expression>
```

Names are lowercase with underscores, unique within a session.

## Primitives

Three, and deliberately no more.

### Region fraction

The proportion of pixels in a region matching a colour range.

```text
health = region_fraction([40, 900, 300, 24], hue=[0, 10], saturation>0.5)
```

The region is `[x, y, width, height]` in capture-space pixels.
Hue is a range; saturation and value are comparisons.
The result is between zero and one.

For health bars, progress bars, cooldown fills — anything whose state is how much of a fixed area is a particular colour.

### Region integer

A number read from a region by text recognition.

```text
carrot_count = region_integer([1700, 60, 120, 40])
currency     = region_integer([1700, 60, 120, 40], digits=6)
```

Yields an integer, or undefined when recognition fails or produces no digits.

**Undefined is not zero.**
A counter that could not be read is not a counter reading zero, and treating it as one is how an agent concludes it has lost all its currency because a tooltip covered the display for a frame.

### Template presence

Whether a stored template matches above a threshold.

```text
in_menu   = template_present("pause_panel", 0.9)
shop_open = template_present("shop_header", 0.85)
```

Templates come from the session's atlas — elements the agent interacted with and confirmed.
A template name that does not exist in the atlas makes the expression undefined rather than false.

## Operators

| Operator | Applies to |
| --- | --- |
| `>` `<` `>=` `<=` `==` `!=` | Numbers |
| `and` `or` `not` | Booleans |
| `@name` | The previous tick's value of `name` |

The previous-value reference is what makes change detectable:

```text
progressed = carrot_count > @carrot_count
bleeding   = health < @health
```

## Undefined

Any primitive can be undefined, and it propagates.

An expression that evaluates to undefined yields **inconclusive** rather than false, which is how the third verification value in [ADR-0036](../decisions/0036-three-valued-verification.md) arises in practice.

This is the single most important property of the language.
Without it, a sensor that could not be read this tick looks exactly like a sensor reporting failure, and the agent thrashes through its escalation ladder on situations that were never wrong.

## What the language deliberately cannot do

No loops, no function definitions, no arbitrary arithmetic, no access to anything but the current frame and the previous tick's sensor values.

Every expression evaluates in constant time and cannot fail in a way that blocks a tick.
A sensor language that could be slow would be a sensor language that could break the reflex loop, and the reflex loop is where every safety interlock lives.

## Declaration and lifetime

Sensors are declared by the deliberative cadence through a skill, and live for the session.

They are discarded when the session ends, like everything else the agent learns (`CON-006`).

## Using them as postconditions

`FR-LOOP-004` requires every subgoal to carry its postcondition as an expression rather than as prose:

```text
subgoal: "find the currency activity"
post:    carrot_count > @carrot_count
```

"I have found the activity" is not a postcondition.
It needs a model call to evaluate, which means the agent can only afford to check occasionally, which means it spends most of its time not knowing whether it is succeeding.

## Worked example

Collecting a resource in an unfamiliar game:

```text
carrot_count = region_integer([1700, 60, 120, 40])
health       = region_fraction([40, 900, 300, 24], hue=[0, 10], saturation>0.5)
in_menu      = template_present("pause_panel", 0.9)

progressed   = carrot_count > @carrot_count
in_danger    = health < 0.3
blocked      = in_menu and not progressed
```

`progressed` is the postcondition for the collection subgoal.
`in_danger` triggers a deliberative pass.
`blocked` is a stuck signal: a menu is open and the counter is not moving, which means the agent is looking at something it did not intend to open.

## Open questions

1. Whether regions should be expressible relative to the window rather than in absolute pixels. Absolute breaks when the window resizes; relative needs a resolution the expression does not have.
2. Whether a sensor should be able to reference another sensor's current value rather than only its previous one. Useful, and it introduces evaluation order.
3. How a sensor declared against one scene class behaves when the scene changes. Currently it keeps evaluating, which is frequently meaningless and occasionally exactly right.

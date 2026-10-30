For NIM, keep them separate:

- **Salience:** how much the content deserves attention.
- **Strength:** how resistant the memory is to forgetting.
- **Confidence:** how reliable you believe it is.

Your existing field descriptions already follow this distinction.

**Salience**

Calculate it when artifacts and annotations arrive. Start with normalized signals in `[0, 1]`:

```text
salience =
    0.30 × importance
  + 0.25 × novelty
  + 0.20 × self_relevance
  + 0.15 × emotional_intensity
  + 0.10 × threat
```

Those weights are proposed defaults, not established scientific constants. Your annotation labels already cover these inputs, but **annotation confidence measures certainty about a label, not its intensity**. You need a separate magnitude or a consistently scored content assessment.

For memories containing multiple artifacts, use the strongest artifact salience initially. Summing would make large memories automatically important.

**Strength**

I’d represent strength as a **retention half-life in days**, rather than another arbitrary `[0, 1]` score:

```text
initial_strength = 1 + 6 × salience
retention = 2 ^ (-days_since_reinforcement / strength)
```

After a useful recall:

```text
spacing = clamp(days_since_reinforcement / strength, 0, 1)
strength = min(max_strength, strength × (1 + 0.5 × usefulness × spacing))
```

This gives spaced, useful recalls more reinforcement than repeated immediate retrievals. The coefficients are tunable engineering choices.

Crucially, **appearing in search results shouldn’t automatically count as useful recall**. Otherwise frequently retrieved memories reinforce themselves regardless of whether they helped. Your similarity score measures relevance, not usefulness.

Keep strength stable between reinforcement events and calculate retention as needed. That avoids periodically rewriting every memory just to model decay.

Separating importance, relevance, and recency also has precedent in agent memory retrieval systems. [Generative Agents](https://arxiv.org/abs/2304.03442)
